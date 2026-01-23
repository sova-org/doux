//! Voice parameters - pure data structure for synthesis configuration.
//!
//! This module contains [`VoiceParams`], which holds all parameters that control
//! a single voice's sound. Parameters are grouped by function:
//!
//! - **Core** - frequency, gain, panning, gate
//! - **Oscillator** - sound source, pulse width, spread, waveshaping
//! - **Amplitude Envelope** - ADSR for volume
//! - **Filters** - lowpass, highpass, bandpass with optional envelopes
//! - **Pitch Modulation** - glide, pitch envelope, vibrato, FM
//! - **Amplitude Modulation** - AM, ring modulation
//! - **Effects** - phaser, flanger, chorus, distortion
//! - **Routing** - orbit assignment, effect sends

use crate::oscillator::PhaseShape;
use crate::types::{DelayType, FilterSlope, LfoShape, ReverbType, Source, SubWave};

/// All parameters that control a voice's sound generation.
///
/// This is a pure data structure with no methods beyond [`Default`].
/// The actual signal processing happens in [`Voice`](super::Voice).
#[derive(Clone, Copy)]
pub struct VoiceParams {
    // ─────────────────────────────────────────────────────────────────────
    // Core
    // ─────────────────────────────────────────────────────────────────────
    /// Base frequency in Hz.
    pub freq: f32,
    /// Pitch offset in cents (1/100th of a semitone).
    pub detune: f32,
    /// Playback speed multiplier (also affects pitch for samples).
    pub speed: f32,
    /// Pre-filter gain (0.0 to 1.0+).
    pub gain: f32,
    /// MIDI velocity (0.0 to 1.0), multiplied with gain.
    pub velocity: f32,
    /// Post-envelope gain (0.0 to 1.0+).
    pub postgain: f32,
    /// Stereo pan position (0.0 = left, 0.5 = center, 1.0 = right).
    pub pan: f32,
    /// Gate signal (> 0.0 = note on, 0.0 = note off).
    pub gate: f32,
    /// Optional note duration in seconds. Voice releases when exceeded.
    pub duration: Option<f32>,

    // ─────────────────────────────────────────────────────────────────────
    // Oscillator
    // ─────────────────────────────────────────────────────────────────────
    /// Sound source type (oscillator waveform, sample, or Plaits engine).
    pub sound: Source,
    /// Pulse width for pulse/square waves (0.0 to 1.0).
    pub pw: f32,
    /// Unison spread amount in cents. Enables 7-voice supersaw when > 0.
    pub spread: f32,
    /// Phase shaping parameters for waveform modification.
    pub shape: PhaseShape,
    /// Harmonics control for Plaits engines (0.0 to 1.0).
    pub harmonics: f32,
    /// Timbre control for Plaits engines (0.0 to 1.0).
    pub timbre: f32,
    /// Morph control for Plaits engines (0.0 to 1.0).
    pub morph: f32,
    /// Sample slice/cut index for sample playback.
    pub cut: Option<usize>,
    /// Sub oscillator mix level (0.0 = off, 1.0 = full).
    pub sub: f32,
    /// Sub oscillator octave offset below main (1-3).
    pub sub_oct: u8,
    /// Sub oscillator waveform.
    pub sub_wave: SubWave,

    // ─────────────────────────────────────────────────────────────────────
    // Amplitude Envelope (ADSR)
    // ─────────────────────────────────────────────────────────────────────
    /// Attack time in seconds.
    pub attack: f32,
    /// Decay time in seconds.
    pub decay: f32,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f32,
    /// Release time in seconds.
    pub release: f32,

    // ─────────────────────────────────────────────────────────────────────
    // Lowpass Filter
    // ─────────────────────────────────────────────────────────────────────
    /// Lowpass cutoff frequency in Hz. `None` = filter bypassed.
    pub lpf: Option<f32>,
    /// Lowpass resonance/Q (0.0 to 1.0).
    pub lpq: f32,
    /// Lowpass envelope depth multiplier.
    pub lpe: f32,
    /// Lowpass envelope attack time.
    pub lpa: f32,
    /// Lowpass envelope decay time.
    pub lpd: f32,
    /// Lowpass envelope sustain level.
    pub lps: f32,
    /// Lowpass envelope release time.
    pub lpr: f32,
    /// Enable lowpass filter envelope modulation.
    pub lp_env_active: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Highpass Filter
    // ─────────────────────────────────────────────────────────────────────
    /// Highpass cutoff frequency in Hz. `None` = filter bypassed.
    pub hpf: Option<f32>,
    /// Highpass resonance/Q (0.0 to 1.0).
    pub hpq: f32,
    /// Highpass envelope depth multiplier.
    pub hpe: f32,
    /// Highpass envelope attack time.
    pub hpa: f32,
    /// Highpass envelope decay time.
    pub hpd: f32,
    /// Highpass envelope sustain level.
    pub hps: f32,
    /// Highpass envelope release time.
    pub hpr: f32,
    /// Enable highpass filter envelope modulation.
    pub hp_env_active: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Bandpass Filter
    // ─────────────────────────────────────────────────────────────────────
    /// Bandpass center frequency in Hz. `None` = filter bypassed.
    pub bpf: Option<f32>,
    /// Bandpass resonance/Q (0.0 to 1.0).
    pub bpq: f32,
    /// Bandpass envelope depth multiplier.
    pub bpe: f32,
    /// Bandpass envelope attack time.
    pub bpa: f32,
    /// Bandpass envelope decay time.
    pub bpd: f32,
    /// Bandpass envelope sustain level.
    pub bps: f32,
    /// Bandpass envelope release time.
    pub bpr: f32,
    /// Enable bandpass filter envelope modulation.
    pub bp_env_active: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Filter Slope
    // ─────────────────────────────────────────────────────────────────────
    /// Filter slope (12/24/48 dB per octave) for all filters.
    pub ftype: FilterSlope,

    // ─────────────────────────────────────────────────────────────────────
    // Glide (Portamento)
    // ─────────────────────────────────────────────────────────────────────
    /// Glide time in seconds. `None` = no glide.
    pub glide: Option<f32>,

    // ─────────────────────────────────────────────────────────────────────
    // Pitch Envelope
    // ─────────────────────────────────────────────────────────────────────
    /// Pitch envelope depth in semitones.
    pub penv: f32,
    /// Pitch envelope attack time.
    pub patt: f32,
    /// Pitch envelope decay time.
    pub pdec: f32,
    /// Pitch envelope sustain level.
    pub psus: f32,
    /// Pitch envelope release time.
    pub prel: f32,
    /// Enable pitch envelope modulation.
    pub pitch_env_active: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Vibrato
    // ─────────────────────────────────────────────────────────────────────
    /// Vibrato LFO rate in Hz.
    pub vib: f32,
    /// Vibrato depth in semitones.
    pub vibmod: f32,
    /// Vibrato LFO waveform.
    pub vibshape: LfoShape,

    // ─────────────────────────────────────────────────────────────────────
    // FM Synthesis
    // ─────────────────────────────────────────────────────────────────────
    /// FM modulation index (depth).
    pub fm: f32,
    /// FM harmonic ratio (modulator freq = carrier freq * fmh).
    pub fmh: f32,
    /// FM modulator waveform.
    pub fmshape: LfoShape,
    /// FM envelope depth multiplier.
    pub fme: f32,
    /// FM envelope attack time.
    pub fma: f32,
    /// FM envelope decay time.
    pub fmd: f32,
    /// FM envelope sustain level.
    pub fms: f32,
    /// FM envelope release time.
    pub fmr: f32,
    /// Enable FM envelope modulation.
    pub fm_env_active: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Amplitude Modulation
    // ─────────────────────────────────────────────────────────────────────
    /// AM LFO rate in Hz.
    pub am: f32,
    /// AM depth (0.0 to 1.0).
    pub amdepth: f32,
    /// AM LFO waveform.
    pub amshape: LfoShape,

    // ─────────────────────────────────────────────────────────────────────
    // Ring Modulation
    // ─────────────────────────────────────────────────────────────────────
    /// Ring modulator frequency in Hz.
    pub rm: f32,
    /// Ring modulation depth (0.0 to 1.0).
    pub rmdepth: f32,
    /// Ring modulator waveform.
    pub rmshape: LfoShape,

    // ─────────────────────────────────────────────────────────────────────
    // Phaser
    // ─────────────────────────────────────────────────────────────────────
    /// Phaser LFO rate in Hz. 0 = bypassed.
    pub phaser: f32,
    /// Phaser depth/feedback (0.0 to 1.0).
    pub phaserdepth: f32,
    /// Phaser sweep range in Hz.
    pub phasersweep: f32,
    /// Phaser center frequency in Hz.
    pub phasercenter: f32,

    // ─────────────────────────────────────────────────────────────────────
    // Flanger
    // ─────────────────────────────────────────────────────────────────────
    /// Flanger LFO rate in Hz. 0 = bypassed.
    pub flanger: f32,
    /// Flanger depth (0.0 to 1.0).
    pub flangerdepth: f32,
    /// Flanger feedback amount (0.0 to 1.0).
    pub flangerfeedback: f32,

    // ─────────────────────────────────────────────────────────────────────
    // Chorus
    // ─────────────────────────────────────────────────────────────────────
    /// Chorus LFO rate in Hz. 0 = bypassed.
    pub chorus: f32,
    /// Chorus depth/modulation amount (0.0 to 1.0).
    pub chorusdepth: f32,
    /// Chorus base delay time in ms.
    pub chorusdelay: f32,

    // ─────────────────────────────────────────────────────────────────────
    // Distortion
    // ─────────────────────────────────────────────────────────────────────
    /// Coarse sample rate reduction factor. `None` = bypassed.
    pub coarse: Option<f32>,
    /// Bit crush depth (bits). `None` = bypassed.
    pub crush: Option<f32>,
    /// Wavefolding amount. `None` = bypassed.
    pub fold: Option<f32>,
    /// Wavewrapping amount. `None` = bypassed.
    pub wrap: Option<f32>,
    /// Distortion/saturation amount. `None` = bypassed.
    pub distort: Option<f32>,
    /// Distortion output volume compensation.
    pub distortvol: f32,

    // ─────────────────────────────────────────────────────────────────────
    // Routing / Sends
    // ─────────────────────────────────────────────────────────────────────
    /// Orbit index for effect bus routing (0 to MAX_ORBITS-1).
    pub orbit: usize,
    /// Delay send level (0.0 to 1.0).
    pub delay: f32,
    /// Delay time in seconds (overrides orbit default).
    pub delaytime: f32,
    /// Delay feedback amount (overrides orbit default).
    pub delayfeedback: f32,
    /// Delay type (overrides orbit default).
    pub delaytype: DelayType,
    /// Reverb send level (0.0 to 1.0).
    pub verb: f32,
    /// Reverb algorithm type.
    pub verbtype: ReverbType,
    /// Reverb decay time (overrides orbit default).
    pub verbdecay: f32,
    /// Reverb damping (overrides orbit default).
    pub verbdamp: f32,
    /// Reverb pre-delay in seconds.
    pub verbpredelay: f32,
    /// Reverb diffusion amount.
    pub verbdiff: f32,
    /// Comb filter send level (0.0 to 1.0).
    pub comb: f32,
    /// Comb filter frequency in Hz.
    pub combfreq: f32,
    /// Comb filter feedback amount.
    pub combfeedback: f32,
    /// Comb filter damping.
    pub combdamp: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            freq: 330.0,
            detune: 0.0,
            speed: 1.0,
            gain: 1.0,
            velocity: 1.0,
            postgain: 1.0,
            pan: 0.5,
            gate: 1.0,
            duration: None,
            sound: Source::Tri,
            pw: 0.5,
            spread: 0.0,
            shape: PhaseShape::default(),
            harmonics: 0.5,
            timbre: 0.5,
            morph: 0.5,
            cut: None,
            sub: 0.0,
            sub_oct: 1,
            sub_wave: SubWave::Tri,
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.005,
            lpf: None,
            lpq: 0.2,
            lpe: 1.0,
            lpa: 0.001,
            lpd: 0.0,
            lps: 1.0,
            lpr: 0.005,
            lp_env_active: false,
            hpf: None,
            hpq: 0.2,
            hpe: 1.0,
            hpa: 0.001,
            hpd: 0.0,
            hps: 1.0,
            hpr: 0.005,
            hp_env_active: false,
            bpf: None,
            bpq: 0.2,
            bpe: 1.0,
            bpa: 0.001,
            bpd: 0.0,
            bps: 1.0,
            bpr: 0.005,
            bp_env_active: false,
            ftype: FilterSlope::Db12,
            glide: None,
            penv: 1.0,
            patt: 0.001,
            pdec: 0.0,
            psus: 1.0,
            prel: 0.005,
            pitch_env_active: false,
            vib: 0.0,
            vibmod: 0.5,
            vibshape: LfoShape::Sine,
            fm: 0.0,
            fmh: 1.0,
            fmshape: LfoShape::Sine,
            fme: 1.0,
            fma: 0.001,
            fmd: 0.0,
            fms: 1.0,
            fmr: 0.005,
            fm_env_active: false,
            am: 0.0,
            amdepth: 0.5,
            amshape: LfoShape::Sine,
            rm: 0.0,
            rmdepth: 1.0,
            rmshape: LfoShape::Sine,
            phaser: 0.0,
            phaserdepth: 0.75,
            phasersweep: 2000.0,
            phasercenter: 1000.0,
            flanger: 0.0,
            flangerdepth: 0.5,
            flangerfeedback: 0.5,
            chorus: 0.0,
            chorusdepth: 0.5,
            chorusdelay: 25.0,
            coarse: None,
            crush: None,
            fold: None,
            wrap: None,
            distort: None,
            distortvol: 1.0,
            orbit: 0,
            delay: 0.0,
            delaytime: 0.333,
            delayfeedback: 0.6,
            delaytype: DelayType::Standard,
            verb: 0.0,
            verbtype: ReverbType::Dattorro,
            verbdecay: 0.75,
            verbdamp: 0.95,
            verbpredelay: 0.1,
            verbdiff: 0.7,
            comb: 0.0,
            combfreq: 220.0,
            combfeedback: 0.9,
            combdamp: 0.1,
        }
    }
}
