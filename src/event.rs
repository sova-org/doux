use crate::types::{midi2freq, DelayType, LfoShape, ReverbType, SubWave};
use crate::voice::{ModChain, ParamId};

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

    // Pitch
    pub freq: Option<f32>,
    pub detune: Option<f32>,
    pub speed: Option<f32>,
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
    pub partials: Option<f32>,
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
    pub fmalgo: Option<u8>,
    pub fmfb: Option<f32>,

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

    // Smear
    pub smear: Option<f32>,
    pub smearfreq: Option<f32>,
    pub smearfb: Option<f32>,

    // Feedback delay
    pub feedback: Option<f32>,
    pub fbtime: Option<f32>,
    pub fbdamp: Option<f32>,
    pub fblfo: Option<f32>,
    pub fblfodepth: Option<f32>,
    pub fblfoshape: Option<LfoShape>,

    // Chorus
    pub chorus: Option<f32>,
    pub chorusdepth: Option<f32>,
    pub chorusdelay: Option<f32>,

    // Comb filter
    pub comb: Option<f32>,
    pub combfreq: Option<f32>,
    pub combfeedback: Option<f32>,
    pub combdamp: Option<f32>,

    // Sidechain compressor
    pub comp: Option<f32>,
    pub compattack: Option<f32>,
    pub comprelease: Option<f32>,
    pub comporbit: Option<usize>,

    // Distortion
    pub coarse: Option<f32>,
    pub crush: Option<f32>,
    pub fold: Option<f32>,
    pub wrap: Option<f32>,
    pub distort: Option<f32>,
    pub distortvol: Option<f32>,

    // Stereo
    pub width: Option<f32>,
    pub haas: Option<f32>,

    // EQ
    pub eqlo: Option<f32>,
    pub eqmid: Option<f32>,
    pub eqhi: Option<f32>,
    pub eqlofreq: Option<f32>,
    pub eqmidfreq: Option<f32>,
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
    pub verbprelow: Option<f32>,
    pub verbprehigh: Option<f32>,
    pub verblowcut: Option<f32>,
    pub verbhighcut: Option<f32>,
    pub verblowgain: Option<f32>,
    pub verbchorus: Option<f32>,
    pub verbchorusfreq: Option<f32>,

    // Recorder
    pub overdub: Option<bool>,

    // Live input channel selection
    pub inchan: Option<usize>,
}

impl Event {
    pub fn n_as_index(&self) -> usize {
        self.n.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0)
    }

    pub fn n_as_float(&self) -> f32 {
        self.n.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0.0)
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

    fn parse_usize(val: &str) -> Option<usize> {
        val.parse::<f32>().ok().map(|f| f as usize)
    }

    fn parse_u8(val: &str) -> Option<u8> {
        val.parse::<f32>().ok().map(|f| f as u8)
    }

    pub fn parse(input: &str, sr: f32) -> Self {
        let mut event = Self::default();
        let mut iter = input.trim().split('/').filter(|s| !s.is_empty());

        macro_rules! parse_param {
            ($val:expr, $field:ident, $id:expr) => {
                if let Some(chain) = ModChain::parse($val) {
                    event.mods.push(($id, chain));
                } else {
                    event.$field = $val.parse().ok();
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
                "gate" => event.gate = val.parse().ok(),
                "voice" => event.voice = Self::parse_usize(val),
                "reset" => event.reset = Some(val == "1" || val == "true"),
                "orbit" => event.orbit = Self::parse_usize(val),
                "freq" => parse_param!(val, freq, ParamId::Freq),
                "note" => {
                    if let Some(chain) = ModChain::parse(val).map(|c| c.map_values(midi2freq)) {
                        event.mods.push((ParamId::Freq, chain));
                    } else {
                        event.freq = val.parse().ok().map(midi2freq);
                    }
                }
                "detune" => parse_param!(val, detune, ParamId::Detune),
                "speed" => parse_param!(val, speed, ParamId::Speed),
                "stretch" => parse_param!(val, stretch, ParamId::Stretch),
                "fit" => event.fit = val.parse().ok(),
                "sound" | "s" => event.sound = Some(val.to_string()),
                "pw" => parse_param!(val, pw, ParamId::Pw),
                "spread" => event.spread = val.parse().ok(),
                "size" => event.size = val.parse().ok(),
                "warp" => event.warp = val.parse().ok(),
                "mirror" => event.mirror = val.parse().ok(),
                "harmonics" | "harm" => parse_param!(val, harmonics, ParamId::Harmonics),
                "timbre" => parse_param!(val, timbre, ParamId::Timbre),
                "morph" => parse_param!(val, morph, ParamId::Morph),
                "partials" => parse_param!(val, partials, ParamId::Partials),
                "n" => event.n = Some(val.to_string()),
                "cut" => event.cut = Self::parse_usize(val),
                "begin" => event.begin = val.parse().ok(),
                "end" => event.end = val.parse().ok(),
                "slice" => event.slice = val.parse().ok(),
                "pick" => event.pick = val.parse().ok(),
                "bank" => event.bank = Some(val.to_string()),
                "wave" | "waveform" => parse_param!(val, wave, ParamId::Wave),
                "sub" => parse_param!(val, sub, ParamId::Sub),
                "suboct" => event.sub_oct = Self::parse_u8(val),
                "subwave" => event.sub_wave = val.parse().ok(),
                "scan" => parse_param!(val, scan, ParamId::Scan),
                "wtlen" => event.wtlen = val.parse().ok(),
                "file_pcm" => event.file_pcm = val.parse().ok(),
                "file_frames" => event.file_frames = val.parse().ok(),
                "file_channels" => event.file_channels = Self::parse_u8(val),
                "file_freq" => event.file_freq = val.parse().ok(),
                "gain" => parse_param!(val, gain, ParamId::Gain),
                "postgain" => parse_param!(val, postgain, ParamId::Postgain),
                "velocity" => event.velocity = val.parse().ok(),
                "pan" => parse_param!(val, pan, ParamId::Pan),
                "envdelay" | "envdly" => event.envdelay = val.parse().ok(),
                "attack" => event.attack = val.parse().ok(),
                "hold" | "hld" => event.hold = val.parse().ok(),
                "decay" => event.decay = val.parse().ok(),
                "sustain" => event.sustain = val.parse().ok(),
                "release" => event.release = val.parse().ok(),
                "lpf" | "cutoff" => parse_param!(val, lpf, ParamId::Lpf),
                "lpq" | "resonance" => parse_param!(val, lpq, ParamId::Lpq),
                "hpf" | "hcutoff" => parse_param!(val, hpf, ParamId::Hpf),
                "hpq" | "hresonance" => parse_param!(val, hpq, ParamId::Hpq),
                "bpf" | "bandf" => parse_param!(val, bpf, ParamId::Bpf),
                "bpq" | "bandq" => parse_param!(val, bpq, ParamId::Bpq),
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
                "fmalgo" => event.fmalgo = Self::parse_u8(val),
                "fmfb" => parse_param!(val, fmfb, ParamId::Fmfb),
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
                "smear" => parse_param!(val, smear, ParamId::Smear),
                "smearfreq" => parse_param!(val, smearfreq, ParamId::Smearfreq),
                "smearfb" => parse_param!(val, smearfb, ParamId::Smearfb),
                "feedback" | "fb" => parse_param!(val, feedback, ParamId::Feedback),
                "fbtime" | "fbt" => parse_param!(val, fbtime, ParamId::FbTime),
                "fbdamp" | "fbd" => event.fbdamp = val.parse().ok(),
                "fblfo" => event.fblfo = val.parse().ok(),
                "fblfodepth" => event.fblfodepth = val.parse().ok(),
                "fblfoshape" => event.fblfoshape = val.parse().ok(),
                "chorus" | "chorusrate" => parse_param!(val, chorus, ParamId::Chorus),
                "chorusdepth" => parse_param!(val, chorusdepth, ParamId::Chorusdepth),
                "chorusdelay" => parse_param!(val, chorusdelay, ParamId::Chorusdelay),
                "comb" => parse_param!(val, comb, ParamId::Comb),
                "combfreq" => parse_param!(val, combfreq, ParamId::CombFreq),
                "combfeedback" => parse_param!(val, combfeedback, ParamId::CombFeedback),
                "combdamp" => event.combdamp = val.parse().ok(),
                "comp" => parse_param!(val, comp, ParamId::Comp),
                "compattack" | "cattack" => event.compattack = val.parse().ok(),
                "comprelease" | "crelease" => event.comprelease = val.parse().ok(),
                "comporbit" | "corbit" => event.comporbit = Self::parse_usize(val),
                "coarse" => parse_param!(val, coarse, ParamId::Coarse),
                "crush" => parse_param!(val, crush, ParamId::Crush),
                "fold" => parse_param!(val, fold, ParamId::Fold),
                "wrap" => parse_param!(val, wrap, ParamId::Wrap),
                "distort" => parse_param!(val, distort, ParamId::Distort),
                "distortvol" => event.distortvol = val.parse().ok(),
                "width" => parse_param!(val, width, ParamId::Width),
                "haas" => parse_param!(val, haas, ParamId::Haas),
                "eqlo" => parse_param!(val, eqlo, ParamId::Eqlo),
                "eqmid" => parse_param!(val, eqmid, ParamId::Eqmid),
                "eqhi" => parse_param!(val, eqhi, ParamId::Eqhi),
                "eqlofreq" => parse_param!(val, eqlofreq, ParamId::EqLoFreq),
                "eqmidfreq" => parse_param!(val, eqmidfreq, ParamId::EqMidFreq),
                "eqhifreq" => parse_param!(val, eqhifreq, ParamId::EqHiFreq),
                "tilt" => parse_param!(val, tilt, ParamId::Tilt),
                "delay" => parse_param!(val, delay, ParamId::Delay),
                "delaytime" => parse_param!(val, delaytime, ParamId::DelayTime),
                "delayfeedback" => parse_param!(val, delayfeedback, ParamId::DelayFeedback),
                "delaytype" | "dtype" => event.delaytype = val.parse().ok(),
                "verb" | "reverb" => parse_param!(val, verb, ParamId::Verb),
                "verbtype" | "vtype" => event.verbtype = val.parse().ok(),
                "verbdecay" => event.verbdecay = val.parse().ok(),
                "verbdamp" => event.verbdamp = val.parse().ok(),
                "verbpredelay" => event.verbpredelay = val.parse().ok(),
                "verbdiff" => event.verbdiff = val.parse().ok(),
                "verbprelow" => event.verbprelow = val.parse().ok(),
                "verbprehigh" => event.verbprehigh = val.parse().ok(),
                "verblowcut" => event.verblowcut = val.parse().ok(),
                "verbhighcut" => event.verbhighcut = val.parse().ok(),
                "verblowgain" => event.verblowgain = val.parse().ok(),
                "verbchorus" | "vchorus" => event.verbchorus = val.parse().ok(),
                "verbchorusfreq" | "vchorusfreq" => event.verbchorusfreq = val.parse().ok(),
                "overdub" | "dub" => event.overdub = Some(val == "1" || val == "true"),
                "inchan" => event.inchan = Self::parse_usize(val),
                _ => {}
            }
        }
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
    fn floor_prevents_boundary_collision() {
        // Two times straddling a sample boundary must not produce the same tick
        let t_low = format!("time/{}", 4.9999999 / SR as f64);
        let t_high = format!("time/{}", 5.0000001 / SR as f64);
        let e_low = Event::parse(&t_low, SR);
        let e_high = Event::parse(&t_high, SR);
        assert_ne!(e_low.tick, e_high.tick, "floor should keep boundary times on distinct ticks");
    }
}
