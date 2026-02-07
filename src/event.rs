use crate::types::{midi2freq, DelayType, FilterSlope, LfoShape, ReverbType, SubWave};
use crate::voice::{ModChain, ParamId};

#[derive(Clone, Default, Debug)]
pub struct Event {
    pub cmd: Option<String>,

    // Timing
    pub time: Option<f64>,
    pub delta: Option<f64>,
    pub repeat: Option<f32>,
    pub duration: Option<f32>,
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
    pub glide: Option<f32>,

    // Fit sample playback into a target duration (seconds)
    pub fit: Option<f32>,

    // Source
    pub sound: Option<String>,
    pub pw: Option<f32>,
    pub spread: Option<f32>,
    pub size: Option<u16>,
    pub mult: Option<f32>,
    pub warp: Option<f32>,
    pub mirror: Option<f32>,
    pub harmonics: Option<f32>,
    pub timbre: Option<f32>,
    pub morph: Option<f32>,
    pub n: Option<usize>,
    pub cut: Option<usize>,
    pub begin: Option<f32>,
    pub end: Option<f32>,
    pub bank: Option<String>,
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
    pub attack: Option<f32>,
    pub decay: Option<f32>,
    pub sustain: Option<f32>,
    pub release: Option<f32>,

    // Lowpass filter
    pub lpf: Option<f32>,
    pub lpq: Option<f32>,
    pub lpe: Option<f32>,
    pub lpa: Option<f32>,
    pub lpd: Option<f32>,
    pub lps: Option<f32>,
    pub lpr: Option<f32>,

    // Highpass filter
    pub hpf: Option<f32>,
    pub hpq: Option<f32>,
    pub hpe: Option<f32>,
    pub hpa: Option<f32>,
    pub hpd: Option<f32>,
    pub hps: Option<f32>,
    pub hpr: Option<f32>,

    // Bandpass filter
    pub bpf: Option<f32>,
    pub bpq: Option<f32>,
    pub bpe: Option<f32>,
    pub bpa: Option<f32>,
    pub bpd: Option<f32>,
    pub bps: Option<f32>,
    pub bpr: Option<f32>,

    // Ladder filter
    pub llpf: Option<f32>,
    pub llpq: Option<f32>,
    pub lhpf: Option<f32>,
    pub lhpq: Option<f32>,
    pub lbpf: Option<f32>,
    pub lbpq: Option<f32>,

    // Filter type
    pub ftype: Option<FilterSlope>,

    // Pitch envelope
    pub penv: Option<f32>,
    pub patt: Option<f32>,
    pub pdec: Option<f32>,
    pub psus: Option<f32>,
    pub prel: Option<f32>,

    // Vibrato
    pub vib: Option<f32>,
    pub vibmod: Option<f32>,
    pub vibshape: Option<LfoShape>,

    // FM synthesis
    pub fm: Option<f32>,
    pub fmh: Option<f32>,
    pub fmshape: Option<LfoShape>,
    pub fme: Option<f32>,
    pub fma: Option<f32>,
    pub fmd: Option<f32>,
    pub fms: Option<f32>,
    pub fmr: Option<f32>,
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
}

impl Event {
    pub fn parse(input: &str) -> Self {
        let mut event = Self::default();
        let tokens: Vec<&str> = input.trim().split('/').filter(|s| !s.is_empty()).collect();

        macro_rules! parse_param {
            ($val:expr, $field:ident, $id:expr) => {
                if let Some(chain) = ModChain::parse($val) {
                    event.mods.push(($id, chain));
                } else {
                    event.$field = $val.parse().ok();
                }
            };
        }

        let mut i = 0;
        while i + 1 < tokens.len() {
            let key = tokens[i];
            let val = tokens[i + 1];
            match key {
                "doux" | "dirt" => event.cmd = Some(val.to_string()),
                "time" | "t" => event.time = val.parse().ok(),
                "delta" => event.delta = val.parse().ok(),
                "repeat" | "rep" => event.repeat = val.parse().ok(),
                "duration" | "dur" | "d" => event.duration = val.parse().ok(),
                "gate" => event.gate = val.parse().ok(),
                "voice" => event.voice = val.parse::<f32>().ok().map(|f| f as usize),
                "reset" => event.reset = Some(val == "1" || val == "true"),
                "orbit" => event.orbit = val.parse::<f32>().ok().map(|f| f as usize),
                "freq" => parse_param!(val, freq, ParamId::Freq),
                "note" => event.freq = val.parse().ok().map(midi2freq),
                "detune" => parse_param!(val, detune, ParamId::Detune),
                "speed" => parse_param!(val, speed, ParamId::Speed),
                "fit" => event.fit = val.parse().ok(),
                "glide" => event.glide = val.parse().ok(),
                "sound" | "s" => event.sound = Some(val.to_string()),
                "pw" => parse_param!(val, pw, ParamId::Pw),
                "spread" => event.spread = val.parse().ok(),
                "size" => event.size = val.parse().ok(),
                "mult" => event.mult = val.parse().ok(),
                "warp" => event.warp = val.parse().ok(),
                "mirror" => event.mirror = val.parse().ok(),
                "harmonics" | "harm" => parse_param!(val, harmonics, ParamId::Harmonics),
                "timbre" => parse_param!(val, timbre, ParamId::Timbre),
                "morph" => parse_param!(val, morph, ParamId::Morph),
                "n" => event.n = val.parse::<f32>().ok().map(|f| f as usize),
                "cut" => event.cut = val.parse::<f32>().ok().map(|f| f as usize),
                "begin" => event.begin = val.parse().ok(),
                "end" => event.end = val.parse().ok(),
                "bank" => event.bank = Some(val.to_string()),
                "sub" => parse_param!(val, sub, ParamId::Sub),
                "suboct" => event.sub_oct = val.parse::<f32>().ok().map(|f| f as u8),
                "subwave" => event.sub_wave = val.parse().ok(),
                "scan" => parse_param!(val, scan, ParamId::Scan),
                "wtlen" => event.wtlen = val.parse().ok(),
                "file_pcm" => event.file_pcm = val.parse().ok(),
                "file_frames" => event.file_frames = val.parse().ok(),
                "file_channels" => event.file_channels = val.parse::<f32>().ok().map(|f| f as u8),
                "file_freq" => event.file_freq = val.parse().ok(),
                "gain" => parse_param!(val, gain, ParamId::Gain),
                "postgain" => parse_param!(val, postgain, ParamId::Postgain),
                "velocity" => event.velocity = val.parse().ok(),
                "pan" => parse_param!(val, pan, ParamId::Pan),
                "attack" => event.attack = val.parse().ok(),
                "decay" => event.decay = val.parse().ok(),
                "sustain" => event.sustain = val.parse().ok(),
                "release" => event.release = val.parse().ok(),
                "lpf" | "cutoff" => parse_param!(val, lpf, ParamId::Lpf),
                "lpq" | "resonance" => parse_param!(val, lpq, ParamId::Lpq),
                "lpe" | "lpenv" => event.lpe = val.parse().ok(),
                "lpa" | "lpattack" => event.lpa = val.parse().ok(),
                "lpd" | "lpdecay" => event.lpd = val.parse().ok(),
                "lps" | "lpsustain" => event.lps = val.parse().ok(),
                "lpr" | "lprelease" => event.lpr = val.parse().ok(),
                "hpf" | "hcutoff" => parse_param!(val, hpf, ParamId::Hpf),
                "hpq" | "hresonance" => parse_param!(val, hpq, ParamId::Hpq),
                "hpe" | "hpenv" => event.hpe = val.parse().ok(),
                "hpa" => event.hpa = val.parse().ok(),
                "hpd" => event.hpd = val.parse().ok(),
                "hps" => event.hps = val.parse().ok(),
                "hpr" => event.hpr = val.parse().ok(),
                "bpf" | "bandf" => parse_param!(val, bpf, ParamId::Bpf),
                "bpq" | "bandq" => parse_param!(val, bpq, ParamId::Bpq),
                "bpe" | "bpenv" => event.bpe = val.parse().ok(),
                "bpa" | "bpattack" => event.bpa = val.parse().ok(),
                "bpd" | "bpdecay" => event.bpd = val.parse().ok(),
                "bps" | "bpsustain" => event.bps = val.parse().ok(),
                "bpr" | "bprelease" => event.bpr = val.parse().ok(),
                "llpf" => parse_param!(val, llpf, ParamId::Llpf),
                "llpq" => parse_param!(val, llpq, ParamId::Llpq),
                "lhpf" => parse_param!(val, lhpf, ParamId::Lhpf),
                "lhpq" => parse_param!(val, lhpq, ParamId::Lhpq),
                "lbpf" => parse_param!(val, lbpf, ParamId::Lbpf),
                "lbpq" => parse_param!(val, lbpq, ParamId::Lbpq),
                "ftype" => event.ftype = val.parse().ok(),
                "penv" => event.penv = val.parse().ok(),
                "patt" => event.patt = val.parse().ok(),
                "pdec" => event.pdec = val.parse().ok(),
                "psus" => event.psus = val.parse().ok(),
                "prel" => event.prel = val.parse().ok(),
                "vib" => parse_param!(val, vib, ParamId::Vib),
                "vibmod" => parse_param!(val, vibmod, ParamId::Vibmod),
                "vibshape" => event.vibshape = val.parse().ok(),
                "fm" | "fmi" => parse_param!(val, fm, ParamId::Fm),
                "fmh" => parse_param!(val, fmh, ParamId::Fmh),
                "fmshape" => event.fmshape = val.parse().ok(),
                "fme" => event.fme = val.parse().ok(),
                "fma" => event.fma = val.parse().ok(),
                "fmd" => event.fmd = val.parse().ok(),
                "fms" => event.fms = val.parse().ok(),
                "fmr" => event.fmr = val.parse().ok(),
                "fm2" => parse_param!(val, fm2, ParamId::Fm2),
                "fm2h" => parse_param!(val, fm2h, ParamId::Fm2h),
                "fmalgo" => event.fmalgo = val.parse::<f32>().ok().map(|f| f as u8),
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
                "feedback" | "fb" => parse_param!(val, feedback, ParamId::Feedback),
                "fbtime" | "fbt" => event.fbtime = val.parse().ok(),
                "fbdamp" | "fbd" => event.fbdamp = val.parse().ok(),
                "fblfo" => event.fblfo = val.parse().ok(),
                "fblfodepth" => event.fblfodepth = val.parse().ok(),
                "fblfoshape" => event.fblfoshape = val.parse().ok(),
                "chorus" | "chorusrate" => parse_param!(val, chorus, ParamId::Chorus),
                "chorusdepth" => parse_param!(val, chorusdepth, ParamId::Chorusdepth),
                "chorusdelay" => parse_param!(val, chorusdelay, ParamId::Chorusdelay),
                "comb" => parse_param!(val, comb, ParamId::Comb),
                "combfreq" => event.combfreq = val.parse().ok(),
                "combfeedback" => event.combfeedback = val.parse().ok(),
                "combdamp" => event.combdamp = val.parse().ok(),
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
                "tilt" => parse_param!(val, tilt, ParamId::Tilt),
                "delay" => parse_param!(val, delay, ParamId::Delay),
                "delaytime" => event.delaytime = val.parse().ok(),
                "delayfeedback" => event.delayfeedback = val.parse().ok(),
                "delaytype" | "dtype" => event.delaytype = val.parse().ok(),
                "verb" | "reverb" => parse_param!(val, verb, ParamId::Verb),
                "verbtype" | "vtype" => event.verbtype = val.parse().ok(),
                "verbdecay" => event.verbdecay = val.parse().ok(),
                "verbdamp" => event.verbdamp = val.parse().ok(),
                "verbpredelay" => event.verbpredelay = val.parse().ok(),
                "verbdiff" => event.verbdiff = val.parse().ok(),
                _ => {}
            }
            i += 2;
        }
        event
    }
}
