//! Event parsing — the boundary that turns OSC/eval strings into typed `Event`s.

use crate::orbit::OrbitParamId;
use crate::superpan::SpeakerSet;
use crate::types::{
    ChorusType, DelayType, DistortMode, FlangerMode, FoldMode, GenericSlot, LfoShape, ReverbType,
    Source, SubWave, SyncMode, VinylType, midi2freq,
};
use crate::voice::{ModChain, ParamId};

/// A named-param value for an arf patch: a static write to its control lane,
/// or a modulation chain riding the same per-sample machinery as fixed params.
#[derive(Clone, Debug)]
pub enum PatchParamValue {
    Value(f32),
    Chain(ModChain),
}

#[derive(Clone, Default, Debug)]
pub struct Event {
    pub cmd: Option<String>,

    // Timing (sample-accurate)
    pub tick: Option<u64>,
    pub delta: Option<i64>,
    pub gate: Option<f32>,

    // Voice control
    pub voice: Option<usize>,
    pub reset: Option<bool>,
    pub orbit: Option<usize>,

    // Inline parameter modulation
    pub mods: Vec<(ParamId, ModChain)>,
    // Params that arrived as static values; statics displace any active
    // ModChain on the same param when applied to a sounding voice.
    pub static_ids: Vec<ParamId>,
    // Named params for the voice's arf source patch (`p:name/value` keys),
    // still by name: only dispatch can resolve a name against the installed
    // program's lane map. Unresolvable names are silently ignored there.
    pub patch_params: Vec<(String, PatchParamValue)>,
    // Same pair for orbit FX params (sticky on the target orbit).
    pub orbit_mods: Vec<(OrbitParamId, ModChain)>,
    pub orbit_static_ids: Vec<OrbitParamId>,

    // Pitch
    pub freq: Option<f32>,
    pub detune: Option<f32>,
    pub speed: Option<f32>,
    // Portamento time in seconds (sticky on the voice)
    pub glide: Option<f32>,
    // Time stretch
    pub stretch: Option<f32>,

    // Fit sample playback into a target duration (seconds)
    pub fit: Option<f32>,

    // Source
    pub sound: Option<String>,
    pub pw: Option<f32>,
    pub spread: Option<f32>,
    pub size: Option<u16>,
    pub warp: Option<f32>,
    pub mirror: Option<f32>,
    pub harmonics: Option<f32>,
    pub timbre: Option<f32>,
    pub morph: Option<f32>,
    pub n: Option<String>,
    pub cut: Option<usize>,
    pub begin: Option<f32>,
    pub end: Option<f32>,
    pub slice: Option<f32>,
    pub pick: Option<f32>,
    pub bank: Option<String>,
    pub wave: Option<f32>,
    pub sub: Option<f32>,
    pub sub_oct: Option<u8>,
    pub sub_wave: Option<SubWave>,
    pub sync_ratio: Option<f32>,
    pub sync_phase: Option<f32>,
    pub sync_mode: Option<SyncMode>,
    pub scan: Option<f32>,
    pub wtlen: Option<u32>,
    // Web sample (WASM only - set by JavaScript)
    pub file_pcm: Option<usize>,
    pub file_frames: Option<usize>,
    pub file_channels: Option<u8>,
    pub file_freq: Option<f32>,

    // Gain
    pub gain: Option<f32>,
    pub postgain: Option<f32>,
    pub velocity: Option<f32>,
    pub pan: Option<f32>,

    // Gain envelope
    pub envdelay: Option<f32>,
    pub attack: Option<f32>,
    pub hold: Option<f32>,
    pub decay: Option<f32>,
    pub sustain: Option<f32>,
    pub release: Option<f32>,

    // Filters
    pub lpf: Option<f32>,
    pub lpq: Option<f32>,
    pub hpf: Option<f32>,
    pub hpq: Option<f32>,
    pub bpf: Option<f32>,
    pub bpq: Option<f32>,

    // Steep SVF filters (cascaded 4-pole)
    pub slpf: Option<f32>,
    pub slpq: Option<f32>,
    pub shpf: Option<f32>,
    pub shpq: Option<f32>,
    pub sbpf: Option<f32>,
    pub sbpq: Option<f32>,

    // Ladder filter
    pub llpf: Option<f32>,
    pub llpq: Option<f32>,
    pub lhpf: Option<f32>,
    pub lhpq: Option<f32>,
    pub lbpf: Option<f32>,
    pub lbpq: Option<f32>,

    // Vibrato
    pub vib: Option<f32>,
    pub vibmod: Option<f32>,
    pub vibshape: Option<LfoShape>,

    // FM synthesis
    pub fm: Option<f32>,
    pub fmh: Option<f32>,
    pub fmshape: Option<LfoShape>,
    pub fm2: Option<f32>,
    pub fm2h: Option<f32>,
    pub fmpivot: Option<f32>,
    pub fmfb: Option<f32>,
    pub fmloop: Option<f32>,

    // AM
    pub am: Option<f32>,
    pub amdepth: Option<f32>,
    pub amshape: Option<LfoShape>,

    // Ring mod
    pub rm: Option<f32>,
    pub rmdepth: Option<f32>,
    pub rmshape: Option<LfoShape>,

    // Phaser
    pub phaser: Option<f32>,
    pub phaserdepth: Option<f32>,
    pub phasersweep: Option<f32>,
    pub phasercenter: Option<f32>,

    // Flanger
    pub flanger: Option<f32>,
    pub flangerdepth: Option<f32>,
    pub flangerfeedback: Option<f32>,
    pub flangermode: Option<FlangerMode>,
    pub fshift: Option<f32>,
    pub pshift: Option<f32>,
    pub pshiftwin: Option<f32>,
    pub wah: Option<f32>,
    pub wahpeak: Option<f32>,
    pub wahsens: Option<f32>,
    pub wahmanual: Option<f32>,
    pub vinyl: Option<f32>,
    pub vinylwow: Option<f32>,
    pub vinylnoise: Option<f32>,
    pub vinyltone: Option<f32>,
    pub vinyltype: Option<VinylType>,

    // Smear
    pub smear: Option<f32>,
    pub smearfreq: Option<f32>,
    pub smearfb: Option<f32>,

    // Feedback delay
    pub feedback: Option<f32>,
    pub fbtime: Option<f32>,
    pub fbdamp: Option<f32>,
    pub fbcross: Option<f32>,

    // Chorus
    pub chorus: Option<f32>,
    pub chorusdepth: Option<f32>,
    pub chorusdelay: Option<f32>,
    pub chorustype: Option<ChorusType>,

    // Comb filter
    pub comb: Option<f32>,
    pub combfreq: Option<f32>,
    pub combfeedback: Option<f32>,
    pub combdamp: Option<f32>,

    // Sidechain compressor
    pub comp: Option<f32>,
    pub compattack: Option<f32>,
    pub comprelease: Option<f32>,
    pub compthresh: Option<f32>,
    pub compratio: Option<f32>,
    /// `Some(None)` = explicitly reset to self-sidechain (a negative value on
    /// the wire); `None` = the event did not mention it.
    pub comporbit: Option<Option<usize>>,

    // Distortion
    pub coarse: Option<f32>,
    pub crush: Option<f32>,
    pub fold: Option<f32>,
    pub wrap: Option<f32>,
    pub distort: Option<f32>,
    pub distortvol: Option<f32>,
    pub distortmode: Option<DistortMode>,
    pub distortasym: Option<f32>,
    pub foldmode: Option<FoldMode>,

    // Stereo
    pub width: Option<f32>,
    pub haas: Option<f32>,

    // Multichannel (superpan)
    pub superpan: Option<f32>,
    pub superwidth: Option<f32>,
    pub speakers: Option<SpeakerSet>,

    // EQ
    pub eqlo: Option<f32>,
    pub eqmid: Option<f32>,
    pub eqhi: Option<f32>,
    pub eqlofreq: Option<f32>,
    pub eqmidfreq: Option<f32>,
    pub eqmidq: Option<f32>,
    pub eqhifreq: Option<f32>,
    pub tilt: Option<f32>,

    // Delay
    pub delay: Option<f32>,
    pub delaytime: Option<f32>,
    pub delayfeedback: Option<f32>,
    pub delaytype: Option<DelayType>,

    // Reverb
    pub verb: Option<f32>,
    pub verbtype: Option<ReverbType>,
    pub verbdecay: Option<f32>,
    pub verbdamp: Option<f32>,
    pub verbpredelay: Option<f32>,
    pub verbdiff: Option<f32>,
    pub verbsize: Option<f32>,
    pub verbprelow: Option<f32>,
    pub verbprehigh: Option<f32>,
    pub verblowcut: Option<f32>,
    pub verbhighcut: Option<f32>,
    pub verblowgain: Option<f32>,
    pub verbhighgain: Option<f32>,
    pub verbchorus: Option<f32>,
    pub verbchorusfreq: Option<f32>,

    // Orbit arf patch (custom effect). `patch/off` clears; the name "off"
    // is rejected at install so it can never be shadowed.
    pub patch: Option<String>,
    pub patchlevel: Option<f32>,
    // Voice arf insert, resolved at dispatch like a source-patch sound.
    // `fx/off` clears the slot.
    pub fx: Option<String>,

    // Recorder
    pub overdub: Option<bool>,
    // Stop the active recording (paired stop verbs: endrec/endorec/enddub).
    pub rec_stop: Option<bool>,

    // Live input channel selection
    pub inchan: Option<usize>,

    // Pre-computed effective sample name (sound + bank suffix)
    pub effective_name: Option<String>,
}

impl Event {
    pub fn n_as_index(&self) -> usize {
        self.n
            .as_deref()
            .and_then(Self::num)
            .map(|f| f as usize)
            .unwrap_or(0)
    }

    pub fn n_as_float(&self) -> f32 {
        self.n.as_deref().and_then(Self::num).unwrap_or(0.0)
    }

    pub fn resolve_range(&self) -> (f32, f32) {
        if self.begin.is_some() || self.end.is_some() {
            return (self.begin.unwrap_or(0.0), self.end.unwrap_or(1.0));
        }
        if let Some(slices) = self.slice {
            let slices = (slices as u32).max(1);
            let pick = self.pick.unwrap_or(0.0) as i32;
            let idx = pick.rem_euclid(slices as i32) as u32;
            let step = 1.0 / slices as f32;
            let begin = idx as f32 * step;
            (begin, begin + step)
        } else {
            (0.0, 1.0)
        }
    }

    /// The single numeric parse for eval/OSC value tokens. Every value in the
    /// Dirt protocol is conceptually a float; integer params floor it via `as`.
    /// So "7", "7.0" and "7.3" all resolve to index 7 — no route rejects a
    /// decimal. Timing (`tick`, `delta`) and JS-set PCM offsets (`file_pcm`,
    /// `file_frames`) keep their own exact integer parse: they can exceed
    /// f32's 2^24 exact range and are never fractional.
    ///
    /// Non-finite tokens (`nan`, `inf`) are rejected here so no NaN/inf ever
    /// reaches DSP state — a single NaN summed into an orbit's feedback latches
    /// that orbit dead until engine rebuild.
    fn num(val: &str) -> Option<f32> {
        val.parse::<f32>().ok().filter(|f| f.is_finite())
    }

    fn parse_usize(val: &str) -> Option<usize> {
        Self::num(val).map(|f| f as usize)
    }

    fn parse_u8(val: &str) -> Option<u8> {
        Self::num(val).map(|f| f as u8)
    }

    pub fn parse(input: &str, sr: f32) -> Self {
        let mut event = Self::default();
        let mut iter = input.trim().split('/').filter(|s| !s.is_empty());

        // Pre-scan for the sound so per-source semantic params resolve
        // regardless of key order ("bright/0.2/s/pluck" works). Last `s`
        // wins, mirroring the main loop. Sample names and unknown sounds
        // yield None: semantic keys are then silently dropped, the generic
        // timbre/harmonics/morph names still work.
        let source_info = {
            let mut scan = iter.clone();
            let mut found: Option<Source> = None;
            while let (Some(k), Some(v)) = (scan.next(), scan.next()) {
                if k == "sound" || k == "s" {
                    found = v.parse::<Source>().ok();
                }
            }
            found.map(|s| s.info())
        };

        macro_rules! parse_param {
            ($val:expr, $field:ident, $id:expr) => {
                if let Some(chain) = ModChain::parse($val) {
                    event.mods.push(($id, chain));
                } else if let Some(v) = Self::num($val) {
                    event.$field = Some(v);
                    event.static_ids.push($id);
                }
            };
        }

        macro_rules! parse_orbit_param {
            ($val:expr, $field:ident, $id:expr) => {
                if let Some(chain) = ModChain::parse($val) {
                    event.orbit_mods.push(($id, chain));
                } else if let Some(v) = Self::num($val) {
                    event.$field = Some(v);
                    event.orbit_static_ids.push($id);
                }
            };
        }

        while let (Some(key), Some(val)) = (iter.next(), iter.next()) {
            match key {
                "doux" | "dirt" => {
                    event.cmd = Some(val.to_string());
                    if val == "rec" && iter.clone().count() % 2 == 1 {
                        if let Some(name) = iter.next() {
                            event.sound = Some(name.to_string());
                        }
                    }
                }
                "tick" => event.tick = val.parse().ok(),
                "time" | "t" => {
                    // Legacy: convert seconds to ticks
                    event.tick = val
                        .parse::<f64>()
                        .ok()
                        .map(|t| (t * sr as f64).floor() as u64);
                }
                "delta" => event.delta = val.parse().ok(),
                "gate" => event.gate = Self::num(val),
                "voice" => event.voice = Self::parse_usize(val),
                "reset" => event.reset = Some(val == "1" || val == "true"),
                "orbit" => event.orbit = Self::parse_usize(val),
                "freq" => parse_param!(val, freq, ParamId::Freq),
                "note" => {
                    if let Some(chain) = ModChain::parse(val).map(|c| c.map_values(midi2freq)) {
                        event.mods.push((ParamId::Freq, chain));
                    } else if let Some(n) = Self::num(val) {
                        event.freq = Some(midi2freq(n));
                        event.static_ids.push(ParamId::Freq);
                    }
                }
                "detune" => parse_param!(val, detune, ParamId::Detune),
                "speed" => parse_param!(val, speed, ParamId::Speed),
                "glide" => event.glide = Self::num(val),
                "stretch" => parse_param!(val, stretch, ParamId::Stretch),
                "fit" => event.fit = Self::num(val),
                "sound" | "s" => event.sound = Some(val.to_string()),
                "pw" => parse_param!(val, pw, ParamId::Pw),
                "spread" => event.spread = Self::num(val),
                "size" => event.size = Self::num(val).map(|f| f as u16),
                "warp" => event.warp = Self::num(val),
                "mirror" => parse_param!(val, mirror, ParamId::Mirror),
                "harmonics" | "harm" => parse_param!(val, harmonics, ParamId::Harmonics),
                "timbre" => parse_param!(val, timbre, ParamId::Timbre),
                "morph" => parse_param!(val, morph, ParamId::Morph),
                "n" => event.n = Some(val.to_string()),
                "cut" => event.cut = Self::parse_usize(val),
                "begin" => event.begin = Self::num(val),
                "end" => event.end = Self::num(val),
                "slice" => event.slice = Self::num(val),
                "pick" => event.pick = Self::num(val),
                "bank" => event.bank = Some(val.to_string()),
                "wave" | "waveform" => parse_param!(val, wave, ParamId::Wave),
                "sub" => parse_param!(val, sub, ParamId::Sub),
                "suboct" => event.sub_oct = Self::parse_u8(val),
                "subwave" => event.sub_wave = val.parse().ok(),
                "sync" => parse_param!(val, sync_ratio, ParamId::SyncRatio),
                "syncphase" | "syncph" => parse_param!(val, sync_phase, ParamId::SyncPhase),
                "syncmode" => event.sync_mode = val.parse().ok(),
                "scan" => parse_param!(val, scan, ParamId::Scan),
                "wtlen" => event.wtlen = Self::num(val).map(|f| f as u32),
                "file_pcm" => event.file_pcm = val.parse().ok(),
                "file_frames" => event.file_frames = val.parse().ok(),
                "file_channels" => event.file_channels = Self::parse_u8(val),
                "file_freq" => event.file_freq = Self::num(val),
                "gain" => parse_param!(val, gain, ParamId::Gain),
                "postgain" => parse_param!(val, postgain, ParamId::Postgain),
                "velocity" => event.velocity = Self::num(val),
                "pan" => parse_param!(val, pan, ParamId::Pan),
                "envdelay" | "envdly" => event.envdelay = Self::num(val),
                "attack" => event.attack = Self::num(val),
                "hold" | "hld" => event.hold = Self::num(val),
                "decay" => event.decay = Self::num(val),
                "sustain" => event.sustain = Self::num(val),
                "release" => event.release = Self::num(val),
                "lpf" | "cutoff" => parse_param!(val, lpf, ParamId::Lpf),
                "lpq" | "resonance" => parse_param!(val, lpq, ParamId::Lpq),
                "hpf" | "hcutoff" => parse_param!(val, hpf, ParamId::Hpf),
                "hpq" | "hresonance" => parse_param!(val, hpq, ParamId::Hpq),
                "bpf" | "bandf" => parse_param!(val, bpf, ParamId::Bpf),
                "bpq" | "bandq" => parse_param!(val, bpq, ParamId::Bpq),
                "slpf" => parse_param!(val, slpf, ParamId::Slpf),
                "slpq" => parse_param!(val, slpq, ParamId::Slpq),
                "shpf" => parse_param!(val, shpf, ParamId::Shpf),
                "shpq" => parse_param!(val, shpq, ParamId::Shpq),
                "sbpf" => parse_param!(val, sbpf, ParamId::Sbpf),
                "sbpq" => parse_param!(val, sbpq, ParamId::Sbpq),
                "llpf" => parse_param!(val, llpf, ParamId::Llpf),
                "llpq" => parse_param!(val, llpq, ParamId::Llpq),
                "lhpf" => parse_param!(val, lhpf, ParamId::Lhpf),
                "lhpq" => parse_param!(val, lhpq, ParamId::Lhpq),
                "lbpf" => parse_param!(val, lbpf, ParamId::Lbpf),
                "lbpq" => parse_param!(val, lbpq, ParamId::Lbpq),
                "vib" => parse_param!(val, vib, ParamId::Vib),
                "vibmod" => parse_param!(val, vibmod, ParamId::Vibmod),
                "vibshape" => event.vibshape = val.parse().ok(),
                "fm" | "fmi" => parse_param!(val, fm, ParamId::Fm),
                "fmh" => parse_param!(val, fmh, ParamId::Fmh),
                "fmshape" => event.fmshape = val.parse().ok(),
                "fm2" => parse_param!(val, fm2, ParamId::Fm2),
                "fm2h" => parse_param!(val, fm2h, ParamId::Fm2h),
                "fmpivot" => parse_param!(val, fmpivot, ParamId::Fmpivot),
                "fmfb" => parse_param!(val, fmfb, ParamId::Fmfb),
                "fmloop" => parse_param!(val, fmloop, ParamId::Fmloop),
                "am" => parse_param!(val, am, ParamId::Am),
                "amdepth" => parse_param!(val, amdepth, ParamId::Amdepth),
                "amshape" => event.amshape = val.parse().ok(),
                "rm" => parse_param!(val, rm, ParamId::Rm),
                "rmdepth" => parse_param!(val, rmdepth, ParamId::Rmdepth),
                "rmshape" => event.rmshape = val.parse().ok(),
                "phaser" | "phaserrate" => parse_param!(val, phaser, ParamId::Phaser),
                "phaserdepth" => parse_param!(val, phaserdepth, ParamId::Phaserdepth),
                "phasersweep" => parse_param!(val, phasersweep, ParamId::Phasersweep),
                "phasercenter" => parse_param!(val, phasercenter, ParamId::Phasercenter),
                "flanger" | "flangerrate" => parse_param!(val, flanger, ParamId::Flanger),
                "flangerdepth" => parse_param!(val, flangerdepth, ParamId::Flangerdepth),
                "flangerfeedback" => parse_param!(val, flangerfeedback, ParamId::Flangerfeedback),
                "flangermode" | "flmode" => event.flangermode = val.parse().ok(),
                "fshift" | "fsh" => parse_param!(val, fshift, ParamId::Fshift),
                "pshift" | "psh" => parse_param!(val, pshift, ParamId::Pshift),
                "pshiftwin" | "pwin" => parse_param!(val, pshiftwin, ParamId::Pshiftwin),
                "wah" => event.wah = Self::num(val),
                "wahpeak" => event.wahpeak = Self::num(val),
                "wahsens" => event.wahsens = Self::num(val),
                "wahmanual" => event.wahmanual = Self::num(val),
                "vinyl" => event.vinyl = Self::num(val),
                "vinylwow" => event.vinylwow = Self::num(val),
                "vinylnoise" => event.vinylnoise = Self::num(val),
                "vinyltone" => event.vinyltone = Self::num(val),
                "vinyltype" => event.vinyltype = val.parse().ok(),
                "smear" => parse_param!(val, smear, ParamId::Smear),
                "smearfreq" => parse_param!(val, smearfreq, ParamId::Smearfreq),
                "smearfb" => parse_param!(val, smearfb, ParamId::Smearfb),
                "feedback" | "fb" => parse_orbit_param!(val, feedback, OrbitParamId::Feedback),
                "fbtime" | "fbt" => parse_orbit_param!(val, fbtime, OrbitParamId::FbTime),
                "fbdamp" | "fbd" => parse_orbit_param!(val, fbdamp, OrbitParamId::FbDamp),
                "fbcross" | "fbc" => parse_orbit_param!(val, fbcross, OrbitParamId::FbCross),
                "chorus" | "chorusrate" => parse_param!(val, chorus, ParamId::Chorus),
                "chorusdepth" => parse_param!(val, chorusdepth, ParamId::Chorusdepth),
                "chorusdelay" => parse_param!(val, chorusdelay, ParamId::Chorusdelay),
                "chorustype" | "ctype" => event.chorustype = val.parse().ok(),
                "comb" => parse_orbit_param!(val, comb, OrbitParamId::Comb),
                "combfreq" => parse_orbit_param!(val, combfreq, OrbitParamId::CombFreq),
                "combfeedback" => {
                    parse_orbit_param!(val, combfeedback, OrbitParamId::CombFeedback)
                }
                "combdamp" => parse_orbit_param!(val, combdamp, OrbitParamId::CombDamp),
                "comp" => parse_orbit_param!(val, comp, OrbitParamId::Comp),
                "compattack" | "cattack" => {
                    parse_orbit_param!(val, compattack, OrbitParamId::CompAttack)
                }
                "comprelease" | "crelease" => {
                    parse_orbit_param!(val, comprelease, OrbitParamId::CompRelease)
                }
                "compthresh" | "cthresh" => {
                    parse_orbit_param!(val, compthresh, OrbitParamId::CompThresh)
                }
                "compratio" | "cratio" => {
                    parse_orbit_param!(val, compratio, OrbitParamId::CompRatio)
                }
                // Negative selects self-sidechain (glue); the reset script uses
                // -1 to put an orbit back to the default.
                "comporbit" | "corbit" => {
                    event.comporbit =
                        val.parse::<f32>()
                            .ok()
                            .map(|v| if v < 0.0 { None } else { Some(v as usize) })
                }
                "coarse" => parse_param!(val, coarse, ParamId::Coarse),
                "crush" => parse_param!(val, crush, ParamId::Crush),
                "fold" => parse_param!(val, fold, ParamId::Fold),
                "wrap" => parse_param!(val, wrap, ParamId::Wrap),
                "distort" => parse_param!(val, distort, ParamId::Distort),
                "distortvol" => event.distortvol = Self::num(val),
                "distortmode" | "dmode" => event.distortmode = val.parse().ok(),
                "distortasym" | "dasym" => {
                    parse_param!(val, distortasym, ParamId::Distortasym)
                }
                "foldmode" | "fmode" => event.foldmode = val.parse().ok(),
                "width" => parse_param!(val, width, ParamId::Width),
                "haas" => parse_param!(val, haas, ParamId::Haas),
                "superpan" | "span" => parse_param!(val, superpan, ParamId::Superpan),
                "superwidth" | "swidth" => parse_param!(val, superwidth, ParamId::Superwidth),
                "speakers" | "spk" => event.speakers = SpeakerSet::parse(val),
                "eqlo" => parse_param!(val, eqlo, ParamId::Eqlo),
                "eqmid" => parse_param!(val, eqmid, ParamId::Eqmid),
                "eqhi" => parse_param!(val, eqhi, ParamId::Eqhi),
                "eqlofreq" => parse_param!(val, eqlofreq, ParamId::EqLoFreq),
                "eqmidfreq" => parse_param!(val, eqmidfreq, ParamId::EqMidFreq),
                "eqmidq" => parse_param!(val, eqmidq, ParamId::EqMidQ),
                "eqhifreq" => parse_param!(val, eqhifreq, ParamId::EqHiFreq),
                "tilt" => parse_param!(val, tilt, ParamId::Tilt),
                "delay" => parse_orbit_param!(val, delay, OrbitParamId::Delay),
                "delaytime" => parse_orbit_param!(val, delaytime, OrbitParamId::DelayTime),
                "delayfeedback" => {
                    parse_orbit_param!(val, delayfeedback, OrbitParamId::DelayFeedback)
                }
                "delaytype" | "dtype" => event.delaytype = val.parse().ok(),
                "verb" | "reverb" => parse_orbit_param!(val, verb, OrbitParamId::Verb),
                "verbtype" | "vtype" => event.verbtype = val.parse().ok(),
                "verbdecay" => parse_orbit_param!(val, verbdecay, OrbitParamId::VerbDecay),
                "verbdamp" => parse_orbit_param!(val, verbdamp, OrbitParamId::VerbDamp),
                "verbpredelay" => {
                    parse_orbit_param!(val, verbpredelay, OrbitParamId::VerbPredelay)
                }
                "verbdiff" => parse_orbit_param!(val, verbdiff, OrbitParamId::VerbDiff),
                "verbsize" | "vsize" => parse_orbit_param!(val, verbsize, OrbitParamId::VerbSize),
                "verbprelow" => parse_orbit_param!(val, verbprelow, OrbitParamId::VerbPrelow),
                "verbprehigh" => parse_orbit_param!(val, verbprehigh, OrbitParamId::VerbPrehigh),
                "verblowcut" => parse_orbit_param!(val, verblowcut, OrbitParamId::VerbLowcut),
                "verbhighcut" => parse_orbit_param!(val, verbhighcut, OrbitParamId::VerbHighcut),
                "verblowgain" => parse_orbit_param!(val, verblowgain, OrbitParamId::VerbLowgain),
                "verbhighgain" => parse_orbit_param!(val, verbhighgain, OrbitParamId::VerbHighgain),
                "verbchorus" | "vchorus" => {
                    parse_orbit_param!(val, verbchorus, OrbitParamId::VerbChorus)
                }
                "verbchorusfreq" | "vchorusfreq" => {
                    parse_orbit_param!(val, verbchorusfreq, OrbitParamId::VerbChorusFreq)
                }
                "patch" => event.patch = Some(val.to_string()),
                "patchlevel" | "plevel" => {
                    parse_orbit_param!(val, patchlevel, OrbitParamId::PatchLevel)
                }
                "fx" => event.fx = Some(val.to_string()),
                "overdub" | "dub" => event.overdub = Some(val == "1" || val == "true"),
                "endrec" => event.rec_stop = Some(val == "1" || val == "true"),
                "inchan" => event.inchan = Self::parse_usize(val),
                // `p:name` addresses a named param declared by the voice's arf
                // patch (`param name default`). The prefix keeps the patch-param
                // namespace disjoint from the arms above by construction — a
                // patch's `cutoff` never collides with doux's `cutoff`.
                k if k.starts_with("p:") => {
                    let name = &k[2..];
                    if name.is_empty() {
                        continue;
                    }
                    if let Some(chain) = ModChain::parse(val) {
                        event
                            .patch_params
                            .push((name.to_string(), PatchParamValue::Chain(chain)));
                    } else if let Some(v) = Self::num(val) {
                        event
                            .patch_params
                            .push((name.to_string(), PatchParamValue::Value(v)));
                    }
                }
                // Per-source semantic names ("bright" on pluck, "drive" on
                // kick) resolve through the source's ParamInfo table to one
                // of the three generic slots. The flat arms above always win.
                _ => {
                    if let Some(info) = source_info {
                        match info.module.semantic_slot(key) {
                            Some(GenericSlot::Timbre) => {
                                parse_param!(val, timbre, ParamId::Timbre)
                            }
                            Some(GenericSlot::Harmonics) => {
                                parse_param!(val, harmonics, ParamId::Harmonics)
                            }
                            Some(GenericSlot::Morph) => {
                                parse_param!(val, morph, ParamId::Morph)
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        event.effective_name = match (&event.sound, &event.bank) {
            (Some(s), Some(b)) => Some(format!("{s}_{b}")),
            (Some(s), None) => Some(s.clone()),
            _ => None,
        };
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48000.0;

    #[test]
    fn slice_pick_basic() {
        let e = Event::parse("slice/8/pick/3", SR);
        let (b, end) = e.resolve_range();
        assert!((b - 0.375).abs() < 1e-6);
        assert!((end - 0.5).abs() < 1e-6);
    }

    #[test]
    fn slice_defaults_pick_zero() {
        let e = Event::parse("slice/4", SR);
        let (b, end) = e.resolve_range();
        assert!((b - 0.0).abs() < 1e-6);
        assert!((end - 0.25).abs() < 1e-6);
    }

    #[test]
    fn pick_without_slice_full_range() {
        let e = Event::parse("pick/3", SR);
        assert_eq!(e.resolve_range(), (0.0, 1.0));
    }

    #[test]
    fn slice_pick_wraps() {
        let e = Event::parse("slice/8/pick/10", SR);
        let (b, end) = e.resolve_range();
        // 10 % 8 = 2
        assert!((b - 0.25).abs() < 1e-6);
        assert!((end - 0.375).abs() < 1e-6);
    }

    #[test]
    fn slice_pick_negative() {
        let e = Event::parse("slice/8/pick/-1", SR);
        let (b, end) = e.resolve_range();
        // rem_euclid(-1, 8) = 7
        assert!((b - 0.875).abs() < 1e-6);
        assert!((end - 1.0).abs() < 1e-6);
    }

    #[test]
    fn begin_end_takes_precedence() {
        let e = Event::parse("begin/0.1/slice/8/pick/3", SR);
        let (b, end) = e.resolve_range();
        assert!((b - 0.1).abs() < 1e-6);
        assert!((end - 1.0).abs() < 1e-6);
    }

    #[test]
    fn patch_params_parse_static_and_chain() {
        let e = Event::parse("s/pp/p:cutoff/2000/p:res/0.5~0.9:2", SR);
        assert_eq!(e.patch_params.len(), 2);
        assert!(matches!(
            &e.patch_params[0],
            (n, PatchParamValue::Value(v)) if n == "cutoff" && *v == 2000.0
        ));
        assert!(matches!(&e.patch_params[1], (n, PatchParamValue::Chain(_)) if n == "res"));
    }

    #[test]
    fn patch_params_drop_junk() {
        // An empty name and a non-numeric value both vanish instead of erroring,
        // like any other unparseable wire pair.
        let e = Event::parse("p:/1/p:cutoff/loud", SR);
        assert!(e.patch_params.is_empty());
    }

    #[test]
    fn rec_start_captures_name() {
        let e = Event::parse("/doux/rec/loop", SR);
        assert_eq!(e.cmd.as_deref(), Some("rec"));
        assert_eq!(e.sound.as_deref(), Some("loop"));
        assert_eq!(e.rec_stop, None);
    }

    #[test]
    fn rec_stop_sets_flag_not_name() {
        let e = Event::parse("/doux/rec/endrec/1", SR);
        assert_eq!(e.cmd.as_deref(), Some("rec"));
        assert_eq!(e.rec_stop, Some(true));
        assert_eq!(e.sound, None);
    }

    #[test]
    fn rec_orbit_and_overdub() {
        let e = Event::parse("/doux/rec/drums/orbit/0", SR);
        assert_eq!(e.sound.as_deref(), Some("drums"));
        assert_eq!(e.orbit, Some(0));
        let d = Event::parse("/doux/rec/loop/overdub/1", SR);
        assert_eq!(d.sound.as_deref(), Some("loop"));
        assert_eq!(d.overdub, Some(true));
    }

    #[test]
    fn floor_prevents_boundary_collision() {
        // Two times straddling a sample boundary must not produce the same tick
        let t_low = format!("time/{}", 4.9999999 / SR as f64);
        let t_high = format!("time/{}", 5.0000001 / SR as f64);
        let e_low = Event::parse(&t_low, SR);
        let e_high = Event::parse(&t_high, SR);
        assert_ne!(
            e_low.tick, e_high.tick,
            "floor should keep boundary times on distinct ticks"
        );
    }
}
