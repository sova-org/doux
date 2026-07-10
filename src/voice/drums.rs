//! Synthesized drum sources.
//!
//! Each drum generates a shaped waveform with internal timbral dynamics.
//! The engine's existing DAHDSR controls overall amplitude.

use std::f32::consts::{FRAC_PI_2, LOG2_E, TAU};

use crate::dsp::oscillator::poly_blep;
use crate::dsp::{cosf, exp2f, polyblep_square, sinf, SvfMode};
use crate::types::Source;

use super::Voice;

const COWBELL_RATIO: f32 = 1.4836;
/// Inharmonic partial ratios of the shared metal core (hat + cymbal). Modelled
/// on the commonly-cited 808 metal bank (base 205 Hz); the near-unison pair
/// `2.546 / 2.630` beats at ~5.9 Hz, which is what makes the stack shimmer as
/// metal instead of reading as a quasi-harmonic organ chord.
const CYMBAL_RATIOS: [f32; 6] = [1.0, 1.483, 1.800, 2.546, 2.630, 3.897];

/// Exponential decay `e^{-time·rate}`, expressed via base-2 `exp2f`.
///
/// `rate` is the natural decay rate in nepers/second (≈ 1/τ); larger = shorter.
#[inline]
fn decay(time: f32, rate: f32) -> f32 {
    exp2f(-time * rate * LOG2_E)
}

/// Drum oscillator with waveform morphing.
/// waveform: 0.0 = sine, 0.5 = triangle, 1.0 = sawtooth.
///
/// `dt = freq·isr` is the per-sample phase increment, used to band-limit the
/// sawtooth limb with PolyBLEP so pitch-swept hits don't alias.
#[inline]
fn drum_osc(phase: f32, waveform: f32, dt: f32) -> f32 {
    if waveform <= 0.0 {
        return sinf(phase * TAU);
    }
    if waveform >= 1.0 {
        return phase * 2.0 - 1.0 - poly_blep(phase, dt);
    }
    let tri = if phase < 0.5 {
        4.0 * phase - 1.0
    } else {
        3.0 - 4.0 * phase
    };
    if waveform >= 0.5 {
        let t = (waveform - 0.5) * 2.0;
        let saw = phase * 2.0 - 1.0 - poly_blep(phase, dt);
        tri + t * (saw - tri)
    } else {
        let t = waveform * 2.0;
        let sine = sinf(phase * TAU);
        sine + t * (tri - sine)
    }
}

/// Inverse of `dsp/svf.dsp`'s q mapping (`Q = 0.5 + 30q`), so drum call sites
/// state the real filter Q instead of the raw `[0,1]` slider value.
#[inline]
fn svf_q(q: f32) -> f32 {
    (q - 0.5) / 30.0
}

#[inline]
fn wrap_phase(phase: f32) -> f32 {
    if phase >= 1.0 {
        phase - 1.0
    } else if phase < 0.0 {
        phase + 1.0
    } else {
        phase
    }
}

/// Per-voice output trim for a *mixed* kit (not equal loudness). The kick is the
/// reference; the others sit at the produced-kit offsets below, measured as
/// short-term RMS over the first 300 ms relative to kick:
/// snare −1, tom −2, rim −5, cowbell −6, hat −7, cymbal −8 dB.
#[inline]
fn drum_trim(sound: Source) -> f32 {
    match sound {
        Source::Kick => 0.55,
        Source::Snare => 0.85,
        Source::Hat => 3.6,
        Source::Tom => 0.33,
        Source::Rim => 0.78,
        Source::Cowbell => 0.19,
        Source::Cymbal => 0.72,
        _ => 0.5,
    }
}

/// Transparent below `THRESH`, soft-knees toward ±1 above it. Catches the
/// overshoots of the unlimited noisy drums without colouring normal levels.
#[inline]
fn soft_limit(x: f32) -> f32 {
    const THRESH: f32 = 0.8;
    let a = x.abs();
    if a <= THRESH {
        x
    } else {
        let over = a - THRESH;
        let lim = THRESH + (1.0 - THRESH) * over / (over + (1.0 - THRESH));
        if x < 0.0 {
            -lim
        } else {
            lim
        }
    }
}

impl Voice {
    #[inline]
    pub(super) fn run_drum(&mut self, freq: f32, isr: f32) -> f32 {
        let raw = match self.params.sound {
            Source::Kick => self.drum_kick(freq, isr),
            Source::Snare => self.drum_snare(freq, isr),
            Source::Hat => self.drum_hat(freq, isr),
            Source::Tom => self.drum_tom(freq, isr),

            Source::Rim => self.drum_rim(freq, isr),
            Source::Cowbell => self.drum_cowbell(freq, isr),
            Source::Cymbal => self.drum_cymbal(freq, isr),
            _ => 0.0,
        };
        soft_limit(raw * drum_trim(self.params.sound))
    }

    #[inline]
    fn drum_kick(&mut self, freq: f32, isr: f32) -> f32 {
        let sweep_oct = self.params.morph * 4.0;
        let rate = 20.0 + self.params.harmonics * 80.0;
        // Two-stage pitch: fast initial spike (~3 ms) rides on top of main decay.
        let slow = decay(self.time, rate);
        let fast = decay(self.time, 300.0);
        let pitch_env = slow + 0.3 * fast;
        let actual_freq = freq * exp2f(sweep_oct * pitch_env);

        let dt = actual_freq * isr;
        let phase = self.phasor.phase;
        let body = drum_osc(phase, self.params.wave, dt);
        self.phasor.phase = wrap_phase(phase + dt);

        // Self-blooming body: a lowpass tracks the (swept) fundamental, so the
        // body is bright on the attack and rounds off as the pitch settles. This
        // also tames the swept-saw aliasing, since the corner sits above the low
        // fundamental. Q 1.4 is a gentle shoulder, not a resonant peak.
        self.drum_svf.cutoff = actual_freq * 3.0;
        let body = self.drum_svf.process(body, SvfMode::Lp, svf_q(1.4), self.sr);

        // 909-style click transient: a short noise burst band-limited to a
        // ~3 kHz beater band, so it reads as a beater strike, not digital dust.
        let w = self.white();
        self.drum_svf2.cutoff = 3000.0;
        let click = self.drum_svf2.process(w, SvfMode::Bp, svf_q(1.0), self.sr)
            * decay(self.time, 600.0)
            * 0.25;
        let sample = body + click;

        // Transient-scoped drive: saturate only while the pitch envelope is high
        // (`slow`), so the attack is fat but the sub tail stays a clean sine.
        // Velocity opens the drive so hard hits bark; `sv` is neutral at 1.
        let sv = 0.6 + 0.4 * self.params.velocity;
        let drive = self.params.timbre * 4.0 * slow * sv;
        if drive > 0.0 {
            let x = sample * (1.0 + drive);
            x / (1.0 + x.abs())
        } else {
            sample
        }
    }

    #[inline]
    fn drum_snare(&mut self, freq: f32, isr: f32) -> f32 {
        // Two detuned partials (~1 : 1.833) with separate decays so the body
        // brightens then darkens, plus highpassed pink noise for the wire rattle.
        // `wave` morphs sine → triangle for a 909-leaning timbre.
        let shape = self.params.wave * 0.5;
        let h = self.params.harmonics;
        let tone_rate = 18.0 + h * 30.0;
        let env1 = decay(self.time, tone_rate);
        let env2 = decay(self.time, tone_rate * 1.7);

        // Short upward pitch transient on the partials for a snappy attack.
        let ptrans = exp2f(0.15 * decay(self.time, 200.0));

        let f1 = freq * ptrans;
        let dt1 = f1 * isr;
        let p0 = &mut self.spread_phasors[0];
        let t1 = drum_osc(p0.phase, shape, dt1) * env1;
        p0.phase = wrap_phase(p0.phase + dt1);

        let f2 = freq * 1.833 * ptrans;
        let dt2 = f2 * isr;
        let p1 = &mut self.spread_phasors[1];
        let t2 = drum_osc(p1.phase, shape, dt2) * env2 * 0.7;
        p1.phase = wrap_phase(p1.phase + dt2);

        let tones = t1 + t2;

        // Two-band noise. `bright` scales both band centers together rather than
        // sweeping one cutoff.
        let band = 0.6 + h * 0.9;

        // Wires: a midband pink resonance with a slow tail (τ≈50 ms at default;
        // the DAHDSR owns the very end).
        let wire_env = decay(self.time, 10.0 + h * 20.0);
        let w = self.white();
        let pink = self.pink_noise.next(w);
        self.drum_svf.cutoff = 1800.0 * band;
        let wires = self.drum_svf.process(pink, SvfMode::Bp, svf_q(1.5), self.sr) * wire_env;

        // Crack: a broadband onset burst, highpassed, with its own fast envelope
        // (τ≈25 ms). Velocity drives the crack — the crack is the hit strength.
        let sv = 0.6 + 0.4 * self.params.velocity;
        let w2 = self.white();
        self.drum_svf2.cutoff = 6000.0 * band;
        let crack = self.drum_svf2.process(w2, SvfMode::Hp, svf_q(0.7), self.sr)
            * decay(self.time, 40.0)
            * 0.7
            * sv;

        let noise = wires + crack;

        // Equal-power crossfade — no crossfade hole to compensate for.
        let mix = self.params.timbre;
        tones * cosf(mix * FRAC_PI_2) + noise * sinf(mix * FRAC_PI_2)
    }

    #[inline]
    fn drum_hat(&mut self, freq: f32, isr: f32) -> f32 {
        // Inharmonic square cluster (the same metal core as the cymbal) through a
        // BP→HP cascade — the canonical 808 hat topology. The short DAHDSR
        // (default 80 ms) gives the closed-hat tightness.
        let spread_amt = 0.5 + self.params.morph;

        let mut metallic = 0.0_f32;
        for (i, &ratio) in CYMBAL_RATIOS.iter().enumerate() {
            let r = 1.0 + (ratio - 1.0) * spread_amt;
            let f = freq * r;
            let dt = f * isr;
            let p = &mut self.spread_phasors[i];
            metallic += polyblep_square(p.phase, dt);
            p.phase = wrap_phase(p.phase + dt);
        }
        metallic /= 6.0;

        // `bright` scales both band centers; `reso` sets the BP resonance (a
        // screaming hat is available at reso→1, not at default). Velocity darkens
        // the band on soft hits.
        let band = 0.6 + self.params.harmonics;
        let reso = self.params.timbre;
        let vscale = 0.7 + 0.3 * self.params.velocity;

        // Stage 1: resonant bandpass sets the metallic band.
        self.drum_svf.cutoff = 8500.0 * band * vscale;
        let bp_q = svf_q(0.7 + 11.6 * reso * reso);
        let stage1 = self.drum_svf.process(metallic, SvfMode::Bp, bp_q, self.sr);
        // Stage 2: highpass strips the low square leakage.
        self.drum_svf2.cutoff = 6800.0 * band;
        self.drum_svf2.process(stage1, SvfMode::Hp, svf_q(0.7), self.sr)
    }

    #[inline]
    fn drum_tom(&mut self, freq: f32, isr: f32) -> f32 {
        let sweep_oct = self.params.morph * 1.5;
        let rate = 15.0 + self.params.harmonics * 40.0;
        let pitch_env = decay(self.time, rate);
        let actual_freq = freq * exp2f(sweep_oct * pitch_env);

        let dt = actual_freq * isr;
        let phase = self.phasor.phase;
        let body1 = drum_osc(phase, self.params.wave, dt);
        self.phasor.phase = wrap_phase(phase + dt);

        // Second membrane mode (~1.59×, first overtone of an ideal drumhead). Its
        // decay is proportional to the fundamental's (×1.6) so the overtone /
        // fundamental ratio holds across the whole `punch` range instead of the
        // overtone vanishing at low rates.
        let f2 = actual_freq * 1.59;
        let dt2 = f2 * isr;
        let mode2_env = decay(self.time, rate * 1.6);
        let p1 = &mut self.spread_phasors[0];
        let body2 = drum_osc(p1.phase, self.params.wave, dt2) * 0.4 * mode2_env;
        p1.phase = wrap_phase(p1.phase + dt2);

        let body = body1 + body2;

        // Skin warmth: a gentle lowpass tracking the fundamental (near-transparent
        // for the default sine, rounds off the upper harmonics when `wave` > 0).
        self.drum_svf.cutoff = actual_freq * 4.0;
        let body = self.drum_svf.process(body, SvfMode::Lp, svf_q(1.1), self.sr);

        // Stick attack: a short band-limited noise burst (was unfiltered dust).
        // Velocity scales the stick level; `sv` is neutral at 1.
        let sv = 0.6 + 0.4 * self.params.velocity;
        let w = self.white();
        self.drum_svf2.cutoff = 2000.0;
        let attack = self.drum_svf2.process(w, SvfMode::Bp, svf_q(1.0), self.sr)
            * decay(self.time, 200.0)
            * self.params.timbre
            * 0.4
            * sv;
        body + attack
    }

    #[inline]
    fn drum_rim(&mut self, freq: f32, isr: f32) -> f32 {
        // Two short tuned partials → a woody "tock"; no pitch sweep. `timbre` sets
        // the ring length, `morph` shifts the upper partial, `harmonics` the click
        // brightness.
        let ring = 0.5 + self.params.timbre;
        let lo_env = decay(self.time, 90.0 / ring);
        let hi_env = decay(self.time, 150.0 / ring);

        let dt_lo = freq * isr;
        let lo = sinf(self.phasor.phase * TAU) * lo_env;
        self.phasor.phase = wrap_phase(self.phasor.phase + dt_lo);

        let f_hi = freq * (3.0 + self.params.morph * 1.5);
        let dt_hi = f_hi * isr;
        let p1 = &mut self.spread_phasors[0];
        let hi = sinf(p1.phase * TAU) * hi_env;
        p1.phase = wrap_phase(p1.phase + dt_hi);

        let tock = lo + hi * 0.7;

        // Short bandpassed noise click for the stick contact.
        self.drum_svf.cutoff = 3000.0 + self.params.harmonics * 6000.0;
        let click_noise = self.white();
        let click = self
            .drum_svf
            .process(click_noise, SvfMode::Bp, svf_q(5.7), self.sr)
            * decay(self.time, 200.0)
            * 0.5;

        tock + click
    }

    #[inline]
    fn drum_cowbell(&mut self, freq: f32, isr: f32) -> f32 {
        let detune = 1.0 + (COWBELL_RATIO - 1.0) * (0.5 + self.params.morph);
        let freq2 = freq * detune;

        let dt0 = freq * isr;
        let p0 = &mut self.spread_phasors[0];
        let sq0 = polyblep_square(p0.phase, dt0);
        p0.phase = wrap_phase(p0.phase + dt0);

        let dt1 = freq2 * isr;
        let p1 = &mut self.spread_phasors[1];
        let sq1 = polyblep_square(p1.phase, dt1);
        p1.phase = wrap_phase(p1.phase + dt1);

        let mixed = (sq0 + sq1) * 0.5;

        // Soft saturation — emulates 808's "swing type" VCAs
        let drive = 1.0 + self.params.timbre * 4.0;
        let driven = mixed * drive;
        let saturated = driven / (1.0 + driven.abs());

        // Bandpass centered just below the fundamentals so they dominate their
        // own 3rd harmonics.
        let cutoff = freq2 * (0.8 + self.params.harmonics * 0.8);
        self.drum_svf.cutoff = cutoff;
        let tone = self.drum_svf.process(saturated, SvfMode::Bp, svf_q(4.6), self.sr);

        // Die-cast "clank then hum": an amplitude accent on the onset (the 808's
        // accent circuit). Velocity scales the clank depth; `sv` neutral at 1.
        let sv = 0.6 + 0.4 * self.params.velocity;
        tone * (1.0 + 0.8 * decay(self.time, 120.0) * sv)
    }

    #[inline]
    fn drum_cymbal(&mut self, freq: f32, isr: f32) -> f32 {
        let spread_amt = 0.5 + self.params.morph;

        let mut metallic = 0.0_f32;
        for (i, &ratio) in CYMBAL_RATIOS.iter().enumerate() {
            let r = 1.0 + (ratio - 1.0) * spread_amt;
            let cym_freq = freq * r;
            let dt = cym_freq * isr;
            let p = &mut self.spread_phasors[i];
            let pulse = polyblep_square(p.phase, dt);
            p.phase = wrap_phase(p.phase + dt);
            metallic += pulse;
        }
        metallic /= 6.0;

        // `bright` scales both band centers together.
        let band = 0.6 + self.params.harmonics * 1.2;

        // Strike band: the metallic clang, mid-band, decaying fast (τ≈125 ms).
        // Velocity scales the attack; the shimmer tail is left alone.
        let sv = 0.6 + 0.4 * self.params.velocity;
        self.drum_svf.cutoff = 3500.0 * band;
        let strike = self.drum_svf.process(metallic, SvfMode::Bp, svf_q(1.0), self.sr)
            * decay(self.time, 8.0)
            * sv;

        // Shimmer band: mostly metal (0.7) with a little pink (0.3) so the tail
        // reads as sustained metal, not tape hiss. Highpassed, decaying slowly
        // (τ≈833 ms) so it outlasts the strike. Scaled by `sizzle`.
        let w = self.white();
        let pink = self.pink_noise.next(w);
        let air = 0.7 * metallic + 0.3 * pink;
        self.drum_svf2.cutoff = 7500.0 * band;
        let shimmer = self.drum_svf2.process(air, SvfMode::Hp, svf_q(0.9), self.sr)
            * decay(self.time, 1.2)
            * self.params.timbre;

        strike + shimmer
    }
}
