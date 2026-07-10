//! Synthesized drum sources.
//!
//! Each drum generates a shaped waveform with internal timbral dynamics.
//! The engine's existing DAHDSR controls overall amplitude.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::dsp::modal::{ModalParams, MODAL_MODES};
use crate::dsp::oscillator::poly_blep;
use crate::dsp::{cosf, decay, exp2f, polyblep_square, sinf, SvfMode};
use crate::types::Source;

use super::Voice;

const COWBELL_RATIO: f32 = 1.4836;
/// Inharmonic mode ratios of the metal resonator bank (hat + cymbal). The first
/// six are the commonly-cited 808 metal set; the near-unison pair `2.546 / 2.630`
/// beats at ~5.9 Hz, which is what makes the stack read as metal instead of a
/// quasi-harmonic organ chord. The upper ten extend the bank into the sizzle
/// region with progressively wider inharmonic spacing so the tail shimmers.
const MODAL_RATIOS: [f32; MODAL_MODES] = [
    1.0, 1.483, 1.800, 2.546, 2.630, 3.897, 4.24, 4.83, 5.41, 6.09, 6.77, 7.53, 8.31, 9.17, 10.05,
    11.02,
];

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
/// snare −1, tom −2, rim −5, cowbell −6, hat −7, cymbal −8 dB, clap ≈ snare.
///
/// These constants were tuned on the pre-stereo mono kit. Each drum now carries
/// the full tonal core per channel (correlated) plus per-channel decorrelated
/// noise, so per-channel RMS ≈ the old mono level — the values below are the
/// post-stereo parity starting point. Final calibration is by ear.
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
        Source::Clap => 0.9,
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

/// Per-drum onset-overshoot amount for the shared punch transient. `0.0` = no
/// punch, which returns the drum to its un-punched output exactly. Kick/snare/tom
/// gain the most snap from it; cowbell already has its own clank accent, and the
/// metal/noise drums don't want it.
#[inline]
fn punch_amount(sound: Source) -> f32 {
    match sound {
        Source::Kick => 0.5,
        Source::Snare => 0.4,
        Source::Tom => 0.3,
        _ => 0.0,
    }
}

/// Brief amplitude overshoot at onset (~7 ms time constant): `1 + amount·e^{-t·rate}`.
/// A percussive "smack" without changing the sustained level.
#[inline]
fn punch(t: f32, amount: f32) -> f32 {
    1.0 + amount * decay(t, 150.0)
}

impl Voice {
    #[inline]
    pub(super) fn run_drum(&mut self, freq: f32, isr: f32) -> [f32; 2] {
        let trim = drum_trim(self.params.sound);
        let punch_amt = punch_amount(self.params.sound);
        let raw = match self.params.sound {
            Source::Kick => self.drum_kick(freq, isr),
            Source::Snare => self.drum_snare(freq, isr),
            Source::Hat => self.drum_hat(freq, isr),
            Source::Tom => self.drum_tom(freq, isr),

            Source::Rim => self.drum_rim(freq, isr),
            Source::Cowbell => self.drum_cowbell(freq, isr),
            Source::Cymbal => self.drum_cymbal(freq, isr),
            Source::Clap => self.drum_clap(freq, isr),
            _ => [0.0, 0.0],
        };
        let p = punch(self.time, punch_amt);
        [
            soft_limit(raw[0] * trim * p),
            soft_limit(raw[1] * trim * p),
        ]
    }

    /// Per-hit pitch and decay-rate multipliers for the pitched drums. Harder hits
    /// ride slightly sharper and shorter (real head tension); a small per-hit
    /// jitter (seeded at trigger) keeps programmed patterns from sounding
    /// identical. Both are subtle, fixed-depth — not knobs. Returns
    /// `(pitch_mul, decay_rate_mul)`; a larger decay-rate mul = a shorter hit.
    #[inline]
    fn drum_humanize(&self) -> (f32, f32) {
        let v = self.params.velocity - 0.7;
        let pitch = (1.0 + 0.03 * v) * (1.0 + 0.006 * self.drum_jitter[0]);
        let decay = (1.0 + 0.15 * v) * (1.0 + 0.05 * self.drum_jitter[1]);
        (pitch, decay)
    }

    /// Three summed layers — click / body / sub — each with its own decay, so one
    /// model spans a punchy techno thump (`drive`→0) and a long saturated 808
    /// (long `decay` + `drive`→1). Mono: everything is centered, both channels
    /// carry the same signal. Stereo bass is a mix error.
    #[inline]
    fn drum_kick(&mut self, freq: f32, isr: f32) -> [f32; 2] {
        let (pm, dm) = self.drum_humanize();
        let freq = freq * pm;
        let sweep_oct = self.params.morph * 4.0;
        let rate = (20.0 + self.params.harmonics * 80.0) * dm;
        // Two-stage pitch: fast initial spike (~3 ms) rides on top of main decay.
        let slow = decay(self.time, rate);
        let fast = decay(self.time, 300.0);
        let pitch_env = slow + 0.3 * fast;
        let actual_freq = freq * exp2f(sweep_oct * pitch_env);

        // --- BODY layer: swept pitched osc + self-blooming lowpass. Defines the
        // punchy attack. A lowpass tracks the (swept) fundamental, so the body is
        // bright on the attack and rounds off as the pitch settles; this also tames
        // the swept-saw aliasing (corner above the low fundamental). Q 1.4 is a
        // gentle shoulder, not a resonant peak.
        let dt = actual_freq * isr;
        let phase = self.phasor.phase;
        let body = drum_osc(phase, self.params.wave, dt);
        self.phasor.phase = wrap_phase(phase + dt);
        self.drum_svf[0].cutoff = actual_freq * 3.0;
        let body = self.drum_svf[0].process(body, SvfMode::Lp, svf_q(1.4), self.sr);

        // --- CLICK layer: a short noise burst band-limited to a ~3 kHz beater
        // band, its own fast decay. Defines the attack snap; reads as a beater
        // strike, not digital dust. Survives a lowpassed monitor.
        let w = self.white();
        self.drum_svf2[0].cutoff = 3000.0;
        let click = self.drum_svf2[0].process(w, SvfMode::Bp, svf_q(1.0), self.sr)
            * decay(self.time, 600.0)
            * 0.25;

        // --- SUB layer: a clean sine at the settled fundamental with a longer
        // decay, saturated by `drive` through the transient-scoped saturator.
        // Defines the tail and the 808 weight. Reuses spread_phasors[2] (free for
        // the kick), so no new state.
        let sub_dt = freq * isr;
        let sp = &mut self.spread_phasors[2];
        let sub_sine = sinf(sp.phase * TAU);
        sp.phase = wrap_phase(sp.phase + sub_dt);
        let sub = sub_sine * decay(self.time, rate * 0.6);
        // Transient-scoped drive: saturate only while the pitch envelope is high
        // (`slow`), so the onset is fat but the sub tail settles to a clean sine.
        // Velocity opens the drive so hard hits bark; `sv` is neutral at 1.
        let sv = 0.6 + 0.4 * self.params.velocity;
        let drive = self.params.timbre * 4.0 * slow * sv;
        let sub = if drive > 0.0 {
            let x = sub * (1.0 + drive);
            x / (1.0 + x.abs())
        } else {
            sub
        };

        let out = body + click + sub * 0.8;
        [out, out]
    }

    /// Tonal partials are mono/centered; both noise bands (wires + crack) are
    /// decorrelated per channel, so the snare images wide while the body holds
    /// center. The main stereo element of the kit.
    #[inline]
    fn drum_snare(&mut self, freq: f32, isr: f32) -> [f32; 2] {
        // Two detuned partials (~1 : 1.833) with separate decays so the body
        // brightens then darkens, plus highpassed pink noise for the wire rattle.
        // `wave` morphs sine → triangle for a 909-leaning timbre.
        let (pm, dm) = self.drum_humanize();
        let freq = freq * pm;
        let shape = self.params.wave * 0.5;
        let h = self.params.harmonics;
        let tone_rate = (18.0 + h * 30.0) * dm;
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
        // the DAHDSR owns the very end). Crack: a broadband onset burst,
        // highpassed, with its own fast envelope (τ≈25 ms). Velocity drives the
        // crack — the crack is the hit strength.
        let wire_env = decay(self.time, 10.0 + h * 20.0);
        let crack_env = decay(self.time, 40.0);
        let sv = 0.6 + 0.4 * self.params.velocity;

        // Equal-power crossfade — no crossfade hole to compensate for. The body
        // leg is correlated across channels; each noise leg is drawn per channel.
        let mix = self.params.timbre;
        let dry_tone = tones * cosf(mix * FRAC_PI_2);
        let noise_gain = sinf(mix * FRAC_PI_2);

        let mut out = [0.0f32; 2];
        for (c, o) in out.iter_mut().enumerate() {
            let w = self.white();
            let pink = self.pink_noise[c].next(w);
            self.drum_svf[c].cutoff = 1800.0 * band;
            let wires = self.drum_svf[c].process(pink, SvfMode::Bp, svf_q(1.5), self.sr) * wire_env;

            let w2 = self.white();
            self.drum_svf2[c].cutoff = 6000.0 * band;
            let crack = self.drum_svf2[c].process(w2, SvfMode::Hp, svf_q(0.7), self.sr)
                * crack_env
                * 0.7
                * sv;

            *o = dry_tone + (wires + crack) * noise_gain;
        }
        out
    }

    /// Metal resonator bank excited by per-channel noise, then highpassed to a
    /// tight bright band — the closed-hat voicing. `metal`(morph) spreads the mode
    /// ratios, `bright`(harm) raises the base band and the final HP, `reso`(timbre)
    /// the mode Q. Fast modal decays + the short DAHDSR (default 80 ms) keep it
    /// tight; per-channel noise draws give width.
    #[inline]
    fn drum_hat(&mut self, freq: f32, _isr: f32) -> [f32; 2] {
        let spread_amt = 0.5 + self.params.morph;
        let ratios: [f32; MODAL_MODES] =
            std::array::from_fn(|m| 1.0 + (MODAL_RATIOS[m] - 1.0) * spread_amt);

        // `bright` raises the whole band; velocity darkens soft hits.
        let bright = 0.6 + self.params.harmonics;
        let vscale = 0.7 + 0.3 * self.params.velocity;
        let base = freq * 4.0 * vscale;
        let reso = self.params.timbre;
        let q = 0.7 + 11.6 * reso * reso;
        let base_decay = 40.0;

        let mut out = [0.0f32; 2];
        for (c, o) in out.iter_mut().enumerate() {
            let off = if c == 0 { 0.98 } else { 1.02 };
            let w = self.white();
            let p = ModalParams {
                base: base * off,
                ratios: &ratios,
                base_decay,
                q,
            };
            let metal = self.modal.process(c, w, &p, self.time, self.sr);
            // Highpass strips the low modes, leaving the bright metallic cluster.
            self.drum_svf2[c].cutoff = 5000.0 * bright * off;
            *o = self.drum_svf2[c].process(metal, SvfMode::Hp, svf_q(0.7), self.sr);
        }
        out
    }

    /// Body (both membrane modes + skin LP) is mono/centered; only the stick-noise
    /// burst is decorrelated per channel, for subtle width without smearing the
    /// low body.
    #[inline]
    fn drum_tom(&mut self, freq: f32, isr: f32) -> [f32; 2] {
        let (pm, dm) = self.drum_humanize();
        let freq = freq * pm;
        let sweep_oct = self.params.morph * 1.5;
        let rate = (15.0 + self.params.harmonics * 40.0) * dm;
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
        // Mono — the low body stays centered.
        self.drum_svf[0].cutoff = actual_freq * 4.0;
        let body = self.drum_svf[0].process(body, SvfMode::Lp, svf_q(1.1), self.sr);

        // Stick attack: a short band-limited noise burst (was unfiltered dust).
        // Velocity scales the stick level; `sv` is neutral at 1. Drawn per channel
        // so the stick transient images wide.
        let sv = 0.6 + 0.4 * self.params.velocity;
        let attack_env = decay(self.time, 200.0) * self.params.timbre * 0.4 * sv;
        let mut out = [0.0f32; 2];
        for (c, o) in out.iter_mut().enumerate() {
            let w = self.white();
            self.drum_svf2[c].cutoff = 2000.0;
            let attack = self.drum_svf2[c].process(w, SvfMode::Bp, svf_q(1.0), self.sr) * attack_env;
            *o = body + attack;
        }
        out
    }

    /// Short click; the tuned "tock" is mono/centered, with a slight ±1.5% click
    /// filter offset (and independent noise per channel) for a touch of width.
    #[inline]
    fn drum_rim(&mut self, freq: f32, isr: f32) -> [f32; 2] {
        // Two short tuned partials → a woody "tock"; no pitch sweep. `timbre` sets
        // the ring length, `morph` shifts the upper partial, `harmonics` the click
        // brightness.
        let (pm, dm) = self.drum_humanize();
        let freq = freq * pm;
        let ring = 0.5 + self.params.timbre;
        let lo_env = decay(self.time, 90.0 / ring * dm);
        let hi_env = decay(self.time, 150.0 / ring * dm);

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
        let center = 3000.0 + self.params.harmonics * 6000.0;
        let click_env = decay(self.time, 200.0) * 0.5;
        let mut out = [0.0f32; 2];
        for (c, o) in out.iter_mut().enumerate() {
            let off = if c == 0 { 0.985 } else { 1.015 };
            let click_noise = self.white();
            self.drum_svf[c].cutoff = center * off;
            let click =
                self.drum_svf[c].process(click_noise, SvfMode::Bp, svf_q(5.7), self.sr) * click_env;
            *o = tock + click;
        }
        out
    }

    /// Tonal — mono/centered.
    #[inline]
    fn drum_cowbell(&mut self, freq: f32, isr: f32) -> [f32; 2] {
        let (pm, dm) = self.drum_humanize();
        let freq = freq * pm;
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
        self.drum_svf[0].cutoff = cutoff;
        let tone = self.drum_svf[0].process(saturated, SvfMode::Bp, svf_q(4.6), self.sr);

        // Die-cast "clank then hum": an amplitude accent on the onset (the 808's
        // accent circuit). Velocity scales the clank depth; `sv` neutral at 1.
        let sv = 0.6 + 0.4 * self.params.velocity;
        let out = tone * (1.0 + 0.8 * decay(self.time, 120.0 * dm) * sv);
        [out, out]
    }

    /// Fully stereo — the widest element of the kit, and the single biggest lever
    /// for the sound. The metal resonator bank (per-channel noise excitation) gives
    /// the blooming, evolving clang + body; a per-channel highpassed pink shimmer
    /// band adds the sizzle tail. `metal`(morph) spreads the mode ratios,
    /// `bright`(harm) raises the base band, `sizzle`(timbre) the shimmer weight.
    #[inline]
    fn drum_cymbal(&mut self, freq: f32, _isr: f32) -> [f32; 2] {
        let spread_amt = 0.5 + self.params.morph;
        let ratios: [f32; MODAL_MODES] =
            std::array::from_fn(|m| 1.0 + (MODAL_RATIOS[m] - 1.0) * spread_amt);

        // `bright` scales the base band; the bank rings long (base_decay small) so
        // the tail outlasts the strike, with high modes dying first.
        let bright = 0.6 + self.params.harmonics * 1.2;
        let base = freq * bright;
        let q = 3.0;
        let base_decay = 1.2;
        let vscale = 0.7 + 0.3 * self.params.velocity;

        // Shimmer band: highpassed pink, decaying slowly (τ≈833 ms) so it outlasts
        // the modal strike. Scaled by `sizzle`.
        let shimmer_env = decay(self.time, 1.2) * self.params.timbre;

        let mut out = [0.0f32; 2];
        for (c, o) in out.iter_mut().enumerate() {
            let off = if c == 0 { 0.98 } else { 1.02 };
            let w = self.white() * vscale;
            let p = ModalParams {
                base: base * off,
                ratios: &ratios,
                base_decay,
                q,
            };
            let metal = self.modal.process(c, w, &p, self.time, self.sr);

            let w2 = self.white();
            let pink = self.pink_noise[c].next(w2);
            self.drum_svf2[c].cutoff = 7500.0 * bright * off;
            let shimmer =
                self.drum_svf2[c].process(pink, SvfMode::Hp, svf_q(0.9), self.sr) * shimmer_env;

            *o = metal + shimmer;
        }
        out
    }

    /// Multi-tap noise burst into a resonant bandpass, plus a diffuse tail —
    /// a hand-clap "shhhk-ap". `morph` (spread) stretches the tap spacing,
    /// `harmonics` (tone) moves the bandpass center, `timbre` (tail) lengthens
    /// the wash. Stereo via a slight per-channel bandpass offset on independent
    /// noise draws. Taps key off `self.time` (absolute voice seconds).
    #[inline]
    fn drum_clap(&mut self, _freq: f32, _isr: f32) -> [f32; 2] {
        // 3 sharp taps + 1 softer, spacing scaled by `spread` (~8–18 ms).
        let spacing = 0.008 + self.params.morph * 0.010;
        let taps = [0.0, spacing, 2.0 * spacing, 3.0 * spacing];
        let mut burst = 0.0f32;
        for (k, &t0) in taps.iter().enumerate() {
            if self.time >= t0 {
                let a = if k == 3 { 0.6 } else { 1.0 };
                burst += a * decay(self.time - t0, 350.0); // fast per-tap decay
            }
        }
        // Diffuse tail after the taps — longer as `timbre`→1.
        let tail_rate = 40.0 - self.params.timbre * 25.0;
        let tail = 0.5 * decay(self.time, tail_rate);
        let env = burst + tail;

        let center = 700.0 + self.params.harmonics * 1400.0;
        let q = svf_q(2.0);
        let mut out = [0.0f32; 2];
        for (c, o) in out.iter_mut().enumerate() {
            let w = self.white();
            self.drum_svf[c].cutoff = center * if c == 0 { 0.97 } else { 1.03 };
            *o = self.drum_svf[c].process(w, SvfMode::Bp, q, self.sr) * env;
        }
        out
    }
}
