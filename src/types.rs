use std::str::FromStr;

pub const BLOCK_SIZE: usize = 128;
pub const CHANNELS: usize = 2;
pub const DEFAULT_MAX_VOICES: usize = 32;
pub const MAX_EVENTS: usize = 256;
pub const MAX_ORBITS: usize = 8;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Source {
    #[default]
    Tri,
    Sine,
    Saw,
    Zaw,
    Pulse,
    Pulze,
    Add,
    White,
    Pink,
    Brown,
    Kick,
    Snare,
    Hat,
    Tom,

    Rim,
    Cowbell,
    Cymbal,
    Sample,    // Native: disk-loaded samples via FileSource
    Wavetable, // Sample played as wavetable oscillator with pitch tracking
    WebSample, // Web: inline PCM from JavaScript
    LiveInput, // Live audio input (microphone, line-in)
    PlModal,
    PlVa,
    PlWs,
    PlFm,
    PlGrain,
    PlAdd,
    PlWt,
    PlChord,
    PlSwarm,
    PlNoise,
}

impl Source {
    /// Percussive envelope defaults: (freq, attack, decay, sustain, release)
    pub fn drum_defaults(&self) -> Option<(f32, f32, f32, f32, f32)> {
        match self {
            Self::Kick => Some((55.0, 0.001, 0.3, 0.0, 0.005)),
            Self::Snare => Some((180.0, 0.001, 0.15, 0.0, 0.005)),
            Self::Hat => Some((320.0, 0.001, 0.08, 0.0, 0.005)),
            Self::Tom => Some((120.0, 0.001, 0.25, 0.0, 0.005)),

            Self::Rim => Some((400.0, 0.001, 0.04, 0.0, 0.005)),
            Self::Cowbell => Some((540.0, 0.001, 0.12, 0.0, 0.005)),
            Self::Cymbal => Some((420.0, 0.001, 0.5, 0.0, 0.005)),
            _ => None,
        }
    }
}

impl FromStr for Source {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "triangle" | "tri" => Ok(Self::Tri),
            "sine" => Ok(Self::Sine),
            "sawtooth" | "saw" => Ok(Self::Saw),
            "zawtooth" | "zaw" => Ok(Self::Zaw),
            "pulse" | "square" => Ok(Self::Pulse),
            "pulze" | "zquare" => Ok(Self::Pulze),
            "add" => Ok(Self::Add),
            "white" => Ok(Self::White),
            "pink" => Ok(Self::Pink),
            "brown" => Ok(Self::Brown),
            "kick" => Ok(Self::Kick),
            "snare" | "sd" => Ok(Self::Snare),
            "hat" | "hh" | "hihat" => Ok(Self::Hat),
            "tom" => Ok(Self::Tom),

            "rim" | "rimshot" | "rs" => Ok(Self::Rim),
            "cowbell" | "cb" => Ok(Self::Cowbell),
            "cymbal" | "crash" | "cy" => Ok(Self::Cymbal),
            "sample" => Ok(Self::Sample),
            "wt" => Ok(Self::Wavetable),
            "websample" => Ok(Self::WebSample),
            "live" | "livein" | "mic" => Ok(Self::LiveInput),
            "plmodal" | "modal" => Ok(Self::PlModal),
            "plva" | "va" | "analog" => Ok(Self::PlVa),
            "plws" | "ws" | "waveshape" => Ok(Self::PlWs),
            "plfm" | "fm2" => Ok(Self::PlFm),
            "plgrain" | "grain" => Ok(Self::PlGrain),
            "pladd" | "additive" => Ok(Self::PlAdd),
            "plwt" | "wavetable" => Ok(Self::PlWt),
            "plchord" | "chord" => Ok(Self::PlChord),
            "plswarm" | "swarm" => Ok(Self::PlSwarm),
            "plnoise" | "pnoise" => Ok(Self::PlNoise),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SubWave {
    #[default]
    Tri,
    Sine,
    Square,
}

impl FromStr for SubWave {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tri" | "triangle" => Ok(Self::Tri),
            "sine" | "sin" => Ok(Self::Sine),
            "square" | "sq" => Ok(Self::Square),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum LfoShape {
    #[default]
    Sine,
    Tri,
    Saw,
    Square,
    Sh,
}

impl FromStr for LfoShape {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sine" | "sin" => Ok(Self::Sine),
            "tri" | "triangle" => Ok(Self::Tri),
            "saw" | "sawtooth" => Ok(Self::Saw),
            "square" | "sq" => Ok(Self::Square),
            "sh" | "sah" | "random" => Ok(Self::Sh),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ReverbType {
    Plate,
    #[default]
    Space,
}

impl FromStr for ReverbType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plate" | "dattorro" | "0" => Ok(Self::Plate),
            "space" | "vital" | "1" => Ok(Self::Space),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DelayType {
    #[default]
    Standard,
    PingPong,
    Tape,
    Multitap,
}

impl FromStr for DelayType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" | "std" | "0" => Ok(Self::Standard),
            "pingpong" | "pp" | "1" => Ok(Self::PingPong),
            "tape" | "2" => Ok(Self::Tape),
            "multitap" | "multi" | "3" => Ok(Self::Multitap),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Allpass,
    Peaking,
    Lowshelf,
    Highshelf,
}

pub fn midi2freq(note: f32) -> f32 {
    2.0_f32.powf((note - 69.0) / 12.0) * 440.0
}

pub fn freq2midi(freq: f32) -> f32 {
    let safe_freq = freq.max(0.001);
    69.0 + 12.0 * (safe_freq / 440.0).log2()
}
