use std::str::FromStr;

pub const BLOCK_SIZE: usize = 128;
pub const CHANNELS: usize = 2;
pub const MAX_VOICES: usize = 32;
pub const MAX_EVENTS: usize = 64;
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
    White,
    Pink,
    Brown,
    Sample,    // Native: disk-loaded samples via FileSource
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
    PlBass,
    PlSnare,
    PlHat,
}

impl Source {
    pub fn is_plaits_percussion(&self) -> bool {
        matches!(self, Self::PlBass | Self::PlSnare | Self::PlHat)
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
            "white" => Ok(Self::White),
            "pink" => Ok(Self::Pink),
            "brown" => Ok(Self::Brown),
            "sample" => Ok(Self::Sample),
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
            "plbass" | "bass" | "kick" => Ok(Self::PlBass),
            "plsnare" | "snare" => Ok(Self::PlSnare),
            "plhat" | "hat" | "hihat" => Ok(Self::PlHat),
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
pub enum FilterSlope {
    #[default]
    Db12,
    Db24,
    Db48,
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

impl FromStr for FilterSlope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "12db" | "0" => Ok(Self::Db12),
            "24db" | "1" => Ok(Self::Db24),
            "48db" | "2" => Ok(Self::Db48),
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
