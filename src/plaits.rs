//! Unified wrapper for Mutable Instruments Plaits synthesis engines.
//!
//! This module provides a single enum that wraps all 13 synthesis engines from
//! the `mi_plaits_dsp` crate (a Rust port of the Mutable Instruments Plaits
//! Eurorack module). Each engine produces sound through a different synthesis
//! technique.
//!
//! # Engine Categories
//!
//! ## Pitched Engines
//! - [`Modal`](PlaitsEngine::Modal) - Physical modeling of resonant structures
//! - [`Va`](PlaitsEngine::Va) - Virtual analog (classic subtractive synthesis)
//! - [`Ws`](PlaitsEngine::Ws) - Waveshaping synthesis
//! - [`Fm`](PlaitsEngine::Fm) - 2-operator FM synthesis
//! - [`Grain`](PlaitsEngine::Grain) - Granular synthesis
//! - [`Additive`](PlaitsEngine::Additive) - Additive synthesis with harmonic control
//! - [`Wavetable`](PlaitsEngine::Wavetable) - Wavetable oscillator
//! - [`Chord`](PlaitsEngine::Chord) - Polyphonic chord generator
//! - [`Swarm`](PlaitsEngine::Swarm) - Swarm of detuned oscillators
//! - [`Noise`](PlaitsEngine::Noise) - Filtered noise with resonance
//!
//! ## Percussion Engines
//! - [`Bass`](PlaitsEngine::Bass) - Analog kick drum model
//! - [`Snare`](PlaitsEngine::Snare) - Analog snare drum model
//! - [`Hat`](PlaitsEngine::Hat) - Hi-hat synthesis
//!
//! # Control Parameters
//!
//! All engines share a common control interface via [`EngineParameters`]:
//! - `note` - MIDI note number (pitch)
//! - `harmonics` - Timbre brightness/harmonics control
//! - `timbre` - Primary timbre parameter
//! - `morph` - Secondary timbre/morph parameter
//! - `accent` - Velocity/accent amount
//! - `trigger` - Gate/trigger state

use crate::types::{Source, BLOCK_SIZE};
use mi_plaits_dsp::engine::additive_engine::AdditiveEngine;
use mi_plaits_dsp::engine::bass_drum_engine::BassDrumEngine;
use mi_plaits_dsp::engine::chord_engine::ChordEngine;
use mi_plaits_dsp::engine::fm_engine::FmEngine;
use mi_plaits_dsp::engine::grain_engine::GrainEngine;
use mi_plaits_dsp::engine::hihat_engine::HihatEngine;
use mi_plaits_dsp::engine::modal_engine::ModalEngine;
use mi_plaits_dsp::engine::noise_engine::NoiseEngine;
use mi_plaits_dsp::engine::snare_drum_engine::SnareDrumEngine;
use mi_plaits_dsp::engine::swarm_engine::SwarmEngine;
use mi_plaits_dsp::engine::virtual_analog_engine::VirtualAnalogEngine;
use mi_plaits_dsp::engine::waveshaping_engine::WaveshapingEngine;
use mi_plaits_dsp::engine::wavetable_engine::WavetableEngine;
use mi_plaits_dsp::engine::{Engine, EngineParameters};

/// Wrapper enum containing all Plaits synthesis engines.
///
/// Only one engine is active at a time. The engine is lazily initialized
/// when first needed and can be switched by creating a new instance with
/// [`PlaitsEngine::new`].
pub enum PlaitsEngine {
    /// Physical modeling of resonant structures (strings, plates, tubes).
    Modal(ModalEngine),
    /// Classic virtual analog with saw, pulse, and sub oscillator.
    Va(VirtualAnalogEngine),
    /// Waveshaping synthesis for harsh, aggressive timbres.
    Ws(WaveshapingEngine),
    /// Two-operator FM synthesis.
    Fm(FmEngine),
    /// Granular synthesis with pitch-shifting grains.
    Grain(GrainEngine),
    /// Additive synthesis with individual harmonic control.
    Additive(AdditiveEngine),
    /// Wavetable oscillator with smooth morphing.
    Wavetable(WavetableEngine<'static>),
    /// Polyphonic chord generator (boxed due to size).
    Chord(Box<ChordEngine<'static>>),
    /// Swarm of detuned sawtooth oscillators.
    Swarm(SwarmEngine),
    /// Filtered noise with variable resonance.
    Noise(NoiseEngine),
    /// Analog bass drum synthesis.
    Bass(BassDrumEngine),
    /// Analog snare drum synthesis.
    Snare(SnareDrumEngine),
    /// Metallic hi-hat synthesis.
    Hat(HihatEngine),
}

impl PlaitsEngine {
    /// Creates and initializes a new engine based on the given source type.
    ///
    /// # Panics
    /// Panics if `source` is not a Plaits source variant (e.g., `Source::Tri`).
    pub fn new(source: Source, sample_rate: f32) -> Self {
        match source {
            Source::PlModal => {
                let mut e = ModalEngine::new(BLOCK_SIZE);
                e.init(sample_rate);
                Self::Modal(e)
            }
            Source::PlVa => {
                let mut e = VirtualAnalogEngine::new(BLOCK_SIZE);
                e.init(sample_rate);
                Self::Va(e)
            }
            Source::PlWs => {
                let mut e = WaveshapingEngine::new();
                e.init(sample_rate);
                Self::Ws(e)
            }
            Source::PlFm => {
                let mut e = FmEngine::new();
                e.init(sample_rate);
                Self::Fm(e)
            }
            Source::PlGrain => {
                let mut e = GrainEngine::new();
                e.init(sample_rate);
                Self::Grain(e)
            }
            Source::PlAdd => {
                let mut e = AdditiveEngine::new();
                e.init(sample_rate);
                Self::Additive(e)
            }
            Source::PlWt => {
                let mut e = WavetableEngine::new();
                e.init(sample_rate);
                Self::Wavetable(e)
            }
            Source::PlChord => {
                let mut e = ChordEngine::new();
                e.init(sample_rate);
                Self::Chord(Box::new(e))
            }
            Source::PlSwarm => {
                let mut e = SwarmEngine::new();
                e.init(sample_rate);
                Self::Swarm(e)
            }
            Source::PlNoise => {
                let mut e = NoiseEngine::new(BLOCK_SIZE);
                e.init(sample_rate);
                Self::Noise(e)
            }
            Source::PlBass => {
                let mut e = BassDrumEngine::new();
                e.init(sample_rate);
                Self::Bass(e)
            }
            Source::PlSnare => {
                let mut e = SnareDrumEngine::new();
                e.init(sample_rate);
                Self::Snare(e)
            }
            Source::PlHat => {
                let mut e = HihatEngine::new(BLOCK_SIZE);
                e.init(sample_rate);
                Self::Hat(e)
            }
            _ => unreachable!(),
        }
    }

    /// Renders a block of audio samples.
    ///
    /// # Arguments
    /// - `params` - Engine parameters (pitch, timbre, morph, etc.)
    /// - `out` - Output buffer for main signal (length must be `BLOCK_SIZE`)
    /// - `aux` - Output buffer for auxiliary signal (length must be `BLOCK_SIZE`)
    /// - `already_enveloped` - Set to true by percussion engines that apply their own envelope
    pub fn render(
        &mut self,
        params: &EngineParameters,
        out: &mut [f32],
        aux: &mut [f32],
        already_enveloped: &mut bool,
    ) {
        match self {
            Self::Modal(e) => e.render(params, out, aux, already_enveloped),
            Self::Va(e) => e.render(params, out, aux, already_enveloped),
            Self::Ws(e) => e.render(params, out, aux, already_enveloped),
            Self::Fm(e) => e.render(params, out, aux, already_enveloped),
            Self::Grain(e) => e.render(params, out, aux, already_enveloped),
            Self::Additive(e) => e.render(params, out, aux, already_enveloped),
            Self::Wavetable(e) => e.render(params, out, aux, already_enveloped),
            Self::Chord(e) => e.render(params, out, aux, already_enveloped),
            Self::Swarm(e) => e.render(params, out, aux, already_enveloped),
            Self::Noise(e) => e.render(params, out, aux, already_enveloped),
            Self::Bass(e) => e.render(params, out, aux, already_enveloped),
            Self::Snare(e) => e.render(params, out, aux, already_enveloped),
            Self::Hat(e) => e.render(params, out, aux, already_enveloped),
        }
    }

    /// Returns the [`Source`] variant corresponding to the current engine.
    pub fn source(&self) -> Source {
        match self {
            Self::Modal(_) => Source::PlModal,
            Self::Va(_) => Source::PlVa,
            Self::Ws(_) => Source::PlWs,
            Self::Fm(_) => Source::PlFm,
            Self::Grain(_) => Source::PlGrain,
            Self::Additive(_) => Source::PlAdd,
            Self::Wavetable(_) => Source::PlWt,
            Self::Chord(_) => Source::PlChord,
            Self::Swarm(_) => Source::PlSwarm,
            Self::Noise(_) => Source::PlNoise,
            Self::Bass(_) => Source::PlBass,
            Self::Snare(_) => Source::PlSnare,
            Self::Hat(_) => Source::PlHat,
        }
    }
}
