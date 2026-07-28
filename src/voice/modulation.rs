use std::f32::consts::PI;

use crate::dsp::envelope::Dahdsr;
use crate::dsp::{cosf, exp2f, log2f, sinf};

#[inline]
pub fn lcg(seed: u32) -> u32 {
    seed.wrapping_mul(1103515245).wrapping_add(12345)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModCurve {
    Linear,
    Exponential,
    Smooth,
    Swell,
    Pluck,
    Stair,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Hold,
    Rand,
    Drunk,
}

#[derive(Clone, Copy, Debug)]
pub enum ModChain {
    Oscillate {
        min: f32,
        max: f32,
        freq: f32,
        shape: ModShape,
    },
    Transition {
        start: f32,
        target: f32,
        freq: f32,
        curve: ModCurve,
        looping: bool,
    },
    Envelope {
        min: f32,
        max: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    },
    Slew {
        target: f32,
        freq: f32,
        curve: ModCurve,
    },
}

impl ModChain {
    pub fn parse(s: &str) -> Option<Self> {
        if s.contains('^') {
            Self::parse_envelope(s)
        } else if s.contains('>') {
            if s.starts_with('>') {
                Self::parse_slew(s)
            } else {
                Self::parse_transition(s)
            }
        } else if s.contains('~') {
            Self::parse_oscillate(s)
        } else if s.contains('?') {
            Self::parse_random(s)
        } else {
            None
        }
    }

    pub fn map_values(self, f: impl Fn(f32) -> f32) -> Self {
        match self {
            ModChain::Oscillate {
                min,
                max,
                freq,
                shape,
            } => ModChain::Oscillate {
                min: f(min),
                max: f(max),
                freq,
                shape,
            },
            ModChain::Transition {
                start,
                target,
                freq,
                curve,
                looping,
            } => ModChain::Transition {
                start: f(start),
                target: f(target),
                freq,
                curve,
                looping,
            },
            ModChain::Envelope {
                min,
                max,
                attack,
                decay,
                sustain,
                release,
            } => ModChain::Envelope {
                min: f(min),
                max: f(max),
                attack,
                decay,
                sustain,
                release,
            },
            ModChain::Slew {
                target,
                freq,
                curve,
            } => ModChain::Slew {
                target: f(target),
                freq,
                curve,
            },
        }
    }

    fn parse_envelope(s: &str) -> Option<Self> {
        let caret = s.find('^')?;
        let min = finite_f32(&s[..caret])?;
        let rest = &s[caret + 1..];
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.is_empty() || parts.len() > 5 {
            return None;
        }
        let max = finite_f32(parts[0])?;
        let attack = parts.get(1).and_then(|s| finite_f32(s)).unwrap_or(0.003);
        let decay = parts.get(2).and_then(|s| finite_f32(s)).unwrap_or(0.0);
        let sustain = parts.get(3).and_then(|s| finite_f32(s)).unwrap_or(1.0);
        let release = parts.get(4).and_then(|s| finite_f32(s)).unwrap_or(0.005);
        Some(ModChain::Envelope {
            min,
            max,
            attack,
            decay,
            sustain,
            release,
        })
    }

    fn parse_oscillate(s: &str) -> Option<Self> {
        let tilde = s.find('~')?;
        let min = finite_f32(&s[..tilde])?;
        let rest = &s[tilde + 1..];
        let colon = rest.find(':')?;
        let max = finite_f32(&rest[..colon])?;
        let period_str = &rest[colon + 1..];

        let (period, shape) = if let Some(stripped) = period_str.strip_suffix('t') {
            (finite_f32(stripped)?, ModShape::Triangle)
        } else if let Some(stripped) = period_str.strip_suffix('w') {
            (finite_f32(stripped)?, ModShape::Saw)
        } else if let Some(stripped) = period_str.strip_suffix('q') {
            (finite_f32(stripped)?, ModShape::Square)
        } else {
            (finite_f32(period_str)?, ModShape::Sine)
        };

        if period <= 0.0 {
            return None;
        }
        Some(ModChain::Oscillate {
            min,
            max,
            freq: 1.0 / period,
            shape,
        })
    }

    fn parse_random(s: &str) -> Option<Self> {
        let q = s.find('?')?;
        let min = finite_f32(&s[..q])?;
        let rest = &s[q + 1..];
        let colon = rest.find(':')?;
        let max = finite_f32(&rest[..colon])?;
        let period_str = &rest[colon + 1..];

        let (period, shape) = if let Some(stripped) = period_str.strip_suffix('s') {
            (finite_f32(stripped)?, ModShape::Rand)
        } else if let Some(stripped) = period_str.strip_suffix('d') {
            (finite_f32(stripped)?, ModShape::Drunk)
        } else {
            (finite_f32(period_str)?, ModShape::Hold)
        };

        if period <= 0.0 {
            return None;
        }
        Some(ModChain::Oscillate {
            min,
            max,
            freq: 1.0 / period,
            shape,
        })
    }

    fn parse_transition(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('>').collect();
        if parts.len() != 2 {
            return None;
        }
        let start = finite_f32(parts[0])?;
        let colon = parts[1].find(':')?;
        let target = finite_f32(&parts[1][..colon])?;
        let dur_str = &parts[1][colon + 1..];
        let (dur_str, looping) = if let Some(stripped) = dur_str.strip_suffix('~') {
            (stripped, true)
        } else {
            (dur_str, false)
        };
        let (period, curve) = parse_duration_curve(dur_str)?;
        if period <= 0.0 {
            return None;
        }
        Some(ModChain::Transition {
            start,
            target,
            freq: 1.0 / period,
            curve,
            looping,
        })
    }

    fn parse_slew(s: &str) -> Option<Self> {
        let rest = &s[1..];
        let colon = rest.find(':')?;
        let target = finite_f32(&rest[..colon])?;
        let dur_str = &rest[colon + 1..];
        let (period, curve) = parse_duration_curve(dur_str)?;
        if period <= 0.0 {
            return None;
        }
        Some(ModChain::Slew {
            target,
            freq: 1.0 / period,
            curve,
        })
    }
}

/// Parse a token to a finite f32, rejecting `nan`/`inf`. Every modulation
/// endpoint and period flows through here: a NaN period sails past the
/// `period <= 0.0` guards (`NaN <= 0.0` is false) and would mint a NaN
/// frequency `1.0 / NaN`, latching the modulated param dead.
fn finite_f32(s: &str) -> Option<f32> {
    s.parse::<f32>().ok().filter(|f| f.is_finite())
}

fn parse_duration_curve(s: &str) -> Option<(f32, ModCurve)> {
    if let Some(stripped) = s.strip_suffix('e') {
        Some((finite_f32(stripped)?, ModCurve::Exponential))
    } else if let Some(stripped) = s.strip_suffix('s') {
        Some((finite_f32(stripped)?, ModCurve::Smooth))
    } else if let Some(stripped) = s.strip_suffix('i') {
        Some((finite_f32(stripped)?, ModCurve::Swell))
    } else if let Some(stripped) = s.strip_suffix('o') {
        Some((finite_f32(stripped)?, ModCurve::Pluck))
    } else if let Some(stripped) = s.strip_suffix('p') {
        Some((finite_f32(stripped)?, ModCurve::Stair))
    } else {
        Some((finite_f32(s)?, ModCurve::Linear))
    }
}

fn interpolate(from: f32, to: f32, t: f32, curve: ModCurve) -> f32 {
    match curve {
        ModCurve::Linear => from + (to - from) * t,
        ModCurve::Exponential => {
            if from > 0.0 && to > 0.0 {
                from * exp2f(t * log2f(to / from))
            } else {
                from + (to - from) * t * t
            }
        }
        ModCurve::Smooth => {
            let t = (1.0 - cosf(t * PI)) * 0.5;
            from + (to - from) * t
        }
        ModCurve::Swell => from + (to - from) * t * t,
        ModCurve::Pluck => {
            let inv = 1.0 - t;
            from + (to - from) * (1.0 - inv * inv)
        }
        ModCurve::Stair => {
            let stepped = (t * 8.0).floor() / 7.0;
            from + (to - from) * stepped
        }
    }
}

/// Single source of truth for the modulatable fixed-param surface, consumed by
/// `ParamId`, `read_param`, `write_param`, and `param_in_stage`. Row grammar:
///
/// ```text
/// Variant => [plain field | opt field = read_default] (, stage <Stage>)*;
/// ```
///
/// The bracketed kind carries the backing `VoiceParams` field and, for `opt`
/// (`Option<f32>`) params, the `unwrap_or` read default. Each `stage` tag names
/// a [`Stage`] whose per-sample body reads the param. `Mirror` (nested
/// `shape.mirror` + `shape_active` resync) and `PatchLane(u8)` (payload) are
/// hand-written at every expansion site so exhaustiveness stays compiler-checked.
///
/// Params with no `ParamId` — wah, vinyl, spread, velocity, enum modes — cannot
/// be modulated and are absent from this table.
macro_rules! for_each_param {
    ($apply:ident) => {
        $apply! {
            Freq            => [plain freq];
            Gain            => [plain gain], stage PreGain;
            Postgain        => [plain postgain], stage Vca, stage MonoStereo;
            Pan             => [plain pan], stage Pan;
            Speed           => [plain speed];
            Stretch         => [plain stretch];
            Grain           => [plain grain];
            Spray           => [plain spray];
            Dens            => [plain dens];
            Detune          => [plain detune];
            Pw              => [plain pw];
            Wave            => [plain wave];
            Sub             => [plain sub];
            Harmonics       => [plain harmonics];
            Timbre          => [plain timbre];
            Morph           => [plain morph];
            Scan            => [plain scan];
            Lpf             => [opt lpf = 20000.0], stage Lpf;
            Lpq             => [plain lpq], stage Lpf;
            Hpf             => [opt hpf = 0.0], stage Hpf;
            Hpq             => [plain hpq], stage Hpf;
            Bpf             => [opt bpf = 1000.0], stage Bpf;
            Bpq             => [plain bpq], stage Bpf;
            Slpf            => [opt slpf = 20000.0], stage SteepLpf;
            Slpq            => [plain slpq], stage SteepLpf;
            Shpf            => [opt shpf = 0.0], stage SteepHpf;
            Shpq            => [plain shpq], stage SteepHpf;
            Sbpf            => [opt sbpf = 1000.0], stage SteepBpf;
            Sbpq            => [plain sbpq], stage SteepBpf;
            Llpf            => [opt llpf = 20000.0], stage LadderLp;
            Llpq            => [plain llpq], stage LadderLp;
            Lhpf            => [opt lhpf = 0.0], stage LadderHp;
            Lhpq            => [plain lhpq], stage LadderHp;
            Lbpf            => [opt lbpf = 1000.0], stage LadderBp;
            Lbpq            => [plain lbpq], stage LadderBp;
            Fm              => [plain fm];
            Fmh             => [plain fmh];
            Fm2             => [plain fm2];
            Fm2h            => [plain fm2h];
            Fmpivot         => [plain fmpivot];
            Fmfb            => [plain fmfb];
            Fmloop          => [plain fmloop];
            Am              => [plain am], stage Am;
            Amdepth         => [plain amdepth], stage Am;
            Rm              => [plain rm], stage Rm;
            Rmdepth         => [plain rmdepth], stage Rm;
            Vib             => [plain vib];
            Vibmod          => [plain vibmod];
            Phaser          => [plain phaser], stage Phaser;
            Phaserdepth     => [plain phaserdepth], stage Phaser;
            Phasersweep     => [plain phasersweep], stage Phaser;
            Phasercenter    => [plain phasercenter], stage Phaser;
            Flanger         => [plain flanger], stage Flanger;
            Flangerdepth    => [plain flangerdepth], stage Flanger;
            Flangerfeedback => [plain flangerfeedback], stage Flanger;
            Fshift          => [plain fshift], stage FreqShift;
            Pshift          => [plain pshift], stage PitchShift;
            Pshiftwin       => [plain pshiftwin], stage PitchShift;
            Smear           => [plain smear], stage Smear;
            Smearfreq       => [plain smearfreq], stage Smear;
            Smearfb         => [plain smearfb], stage Smear;
            Chorus          => [plain chorus], stage Chorus;
            Chorusdepth     => [plain chorusdepth], stage Chorus;
            Chorusdelay     => [plain chorusdelay], stage Chorus;
            Fold            => [opt fold = 0.0], stage Fold;
            Crush           => [opt crush = 0.0], stage Crush;
            Coarse          => [opt coarse = 0.0], stage Coarse;
            Distort         => [opt distort = 0.0], stage Distort;
            Distortasym     => [plain distortasym], stage Distort;
            Eqlo            => [plain eqlo], stage Eq;
            Eqmid           => [plain eqmid], stage Eq;
            Eqhi            => [plain eqhi], stage Eq;
            Tilt            => [plain tilt], stage Tilt;
            Width           => [plain width], stage Width;
            Haas            => [plain haas], stage Haas;
            Wrap            => [opt wrap = 0.0], stage Wrap;
            EqLoFreq        => [plain eqlofreq], stage Eq;
            EqMidFreq       => [plain eqmidfreq], stage Eq;
            EqMidQ          => [plain eqmidq], stage Eq;
            EqHiFreq        => [plain eqhifreq], stage Eq;
            SyncRatio       => [plain sync_ratio];
            SyncPhase       => [plain sync_phase];
            Superpan        => [opt superpan = 0.0];
            Superwidth      => [plain superwidth];
        }
    };
}
pub(crate) use for_each_param;

macro_rules! define_param_ids {
    ($($v:ident => [$($kind:tt)*] $(, stage $s:ident)* ;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u8)]
        pub enum ParamId {
            $($v,)*
            Mirror,
            /// A named param of the voice's arf source patch, addressed by its absolute
            /// control lane (`PARAM_BASE + declaration index`). Rides the same ParamMod
            /// machinery as the fixed params, so chains modulate patch lanes per sample.
            PatchLane(u8),
        }
    };
}
for_each_param!(define_param_ids);

#[derive(Clone, Copy)]
pub struct ParamMod {
    pub chain: ModChain,
    pub phase: f32,
    pub prev_rand: f32,
    pub next_rand: f32,
    pub seed: u32,
    pub drunk_pos: f32,
    pub envelope: Dahdsr,
    /// `log2(target/start)` for an exponential `Transition` with positive
    /// endpoints, else 0.0. Chain-invariant (endpoints are fixed for a
    /// `ParamMod`'s lifetime — `set_mod` rebuilds via `new`), so the per-sample
    /// glide costs only an `exp2f`, not the `log2f` too.
    pub exp_log_ratio: f32,
}

impl Default for ParamMod {
    fn default() -> Self {
        Self {
            chain: ModChain::Oscillate {
                min: 0.0,
                max: 0.0,
                freq: 0.0,
                shape: ModShape::Sine,
            },
            phase: 0.0,
            prev_rand: 0.0,
            next_rand: 0.0,
            seed: 0,
            drunk_pos: 0.5,
            envelope: Dahdsr::default(),
            exp_log_ratio: 0.0,
        }
    }
}

impl ParamMod {
    pub fn new(chain: ModChain, seed: u32) -> Self {
        let exp_log_ratio = match chain {
            ModChain::Transition {
                start,
                target,
                curve: ModCurve::Exponential,
                ..
            } if start > 0.0 && target > 0.0 => log2f(target / start),
            _ => 0.0,
        };
        let mut m = Self {
            chain,
            phase: 0.0,
            prev_rand: 0.0,
            next_rand: 0.0,
            seed,
            drunk_pos: 0.5,
            envelope: Dahdsr::default(),
            exp_log_ratio,
        };
        m.prev_rand = m.rand();
        m.next_rand = m.rand();
        m
    }

    fn rand(&mut self) -> f32 {
        self.seed = lcg(self.seed);
        ((self.seed >> 16) & 0x7fff) as f32 / 32767.0
    }

    pub fn trigger(&mut self, gate: f32) {
        if matches!(self.chain, ModChain::Envelope { .. }) {
            self.envelope.trigger(gate);
        }
    }

    pub fn force_release(&mut self) {
        if matches!(self.chain, ModChain::Envelope { .. }) {
            self.envelope.force_release();
        }
    }

    pub fn tick(&mut self, isr: f32) -> f32 {
        match self.chain {
            ModChain::Oscillate {
                min,
                max,
                freq,
                shape,
            } => {
                self.phase += freq * isr;
                self.tick_oscillate(min, max, shape)
            }
            ModChain::Transition {
                start,
                target,
                freq,
                curve,
                looping,
            } => {
                self.phase += freq * isr;
                if self.phase >= 1.0 {
                    if looping {
                        self.phase -= 1.0;
                    } else {
                        self.phase = 1.0;
                        return target;
                    }
                }
                // Exponential glide with positive endpoints: use the cached
                // `log2(target/start)` so the per-sample cost is one `exp2f`.
                // Bit-identical to `interpolate`'s exponential branch. Other
                // curves (and non-positive endpoints) fall through.
                if curve == ModCurve::Exponential && start > 0.0 && target > 0.0 {
                    start * exp2f(self.phase * self.exp_log_ratio)
                } else {
                    interpolate(start, target, self.phase, curve)
                }
            }
            ModChain::Envelope {
                min,
                max,
                attack,
                decay,
                sustain,
                release,
            } => {
                let env_val = self
                    .envelope
                    .update(isr, 0.0, attack, 0.0, decay, sustain, release);
                min + (max - min) * env_val
            }
            ModChain::Slew { target, .. } => target,
        }
    }

    /// Advance the modulation by `n` per-sample steps and return the final value.
    ///
    /// Intended for block-rate parameter writes: call once per audio block instead
    /// of `n` times per sample, then write the returned value to the target field once.
    #[inline]
    pub fn tick_block(&mut self, isr: f32, n: usize) -> f32 {
        // SAFETY: no allocation, all paths advance internal state in place.
        let mut v = 0.0;
        for _ in 0..n {
            v = self.tick(isr);
        }
        v
    }

    /// Advance the modulation by one step per element of `out`, writing the value
    /// at each sample. The audio-rate counterpart of [`tick_block`]: where that
    /// keeps only the final value (block-rate write), this keeps the whole
    /// per-sample trajectory so a click-sensitive Faust param can be fed as an
    /// audio-rate input. Returns the final value (for keeping the param field in
    /// sync with the block boundary).
    #[inline]
    pub fn tick_into(&mut self, isr: f32, out: &mut [f32]) -> f32 {
        let mut v = 0.0;
        for slot in out.iter_mut() {
            v = self.tick(isr);
            *slot = v;
        }
        v
    }

    fn tick_oscillate(&mut self, min: f32, max: f32, shape: ModShape) -> f32 {
        let range = max - min;
        match shape {
            ModShape::Sine => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                let mid = min + range * 0.5;
                let amp = range * 0.5;
                mid + amp * sinf(self.phase * 2.0 * PI)
            }
            ModShape::Triangle => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                let t = if self.phase < 0.5 {
                    self.phase * 2.0
                } else {
                    2.0 - self.phase * 2.0
                };
                min + t * range
            }
            ModShape::Saw => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                min + self.phase * range
            }
            ModShape::Square => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                if self.phase < 0.5 {
                    min
                } else {
                    max
                }
            }
            ModShape::Rand => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                    self.prev_rand = self.next_rand;
                    self.next_rand = self.rand();
                }
                let t = (1.0 - cosf(self.phase * PI)) * 0.5;
                let val = self.prev_rand + (self.next_rand - self.prev_rand) * t;
                min + val * range
            }
            ModShape::Hold => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                    self.prev_rand = self.rand();
                }
                min + self.prev_rand * range
            }
            ModShape::Drunk => {
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                    let step = (self.rand() - 0.5) * 0.3;
                    self.drunk_pos = (self.drunk_pos + step).clamp(0.0, 1.0);
                }
                min + self.drunk_pos * range
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_returns_none() {
        assert!(ModChain::parse("200:4000:2").is_none());
        assert!(ModChain::parse("200:4000:2r").is_none());
        assert!(ModChain::parse("200:4000:2h").is_none());
        assert!(ModChain::parse("200:4000:2l").is_none());
        assert!(ModChain::parse("200:4000:2e").is_none());
    }

    #[test]
    fn parse_oscillate_sine() {
        let m = ModChain::parse("200~4000:2").unwrap();
        match m {
            ModChain::Oscillate {
                min,
                max,
                freq,
                shape,
            } => {
                assert_eq!(min, 200.0);
                assert_eq!(max, 4000.0);
                assert_eq!(freq, 0.5);
                assert_eq!(shape, ModShape::Sine);
            }
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_oscillate_triangle() {
        let m = ModChain::parse("200~4000:2t").unwrap();
        match m {
            ModChain::Oscillate { shape, .. } => assert_eq!(shape, ModShape::Triangle),
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_oscillate_saw() {
        let m = ModChain::parse("200~4000:2w").unwrap();
        match m {
            ModChain::Oscillate { shape, .. } => assert_eq!(shape, ModShape::Saw),
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_oscillate_square() {
        let m = ModChain::parse("200~4000:2q").unwrap();
        match m {
            ModChain::Oscillate { shape, .. } => assert_eq!(shape, ModShape::Square),
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_random_hold() {
        let m = ModChain::parse("200?4000:0.5").unwrap();
        match m {
            ModChain::Oscillate {
                min, max, shape, ..
            } => {
                assert_eq!(min, 200.0);
                assert_eq!(max, 4000.0);
                assert_eq!(shape, ModShape::Hold);
            }
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_random_smooth() {
        let m = ModChain::parse("200?4000:0.5s").unwrap();
        match m {
            ModChain::Oscillate { shape, .. } => assert_eq!(shape, ModShape::Rand),
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_random_drunk() {
        let m = ModChain::parse("200?4000:0.1d").unwrap();
        match m {
            ModChain::Oscillate { shape, .. } => assert_eq!(shape, ModShape::Drunk),
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_negative_values() {
        let m = ModChain::parse("-50~50:0.2").unwrap();
        match m {
            ModChain::Oscillate { min, max, .. } => {
                assert_eq!(min, -50.0);
                assert_eq!(max, 50.0);
            }
            _ => panic!("expected Oscillate"),
        }
    }

    #[test]
    fn parse_transition_single() {
        let m = ModChain::parse("200>4000:2").unwrap();
        match m {
            ModChain::Transition {
                start,
                target,
                freq,
                curve,
                looping,
            } => {
                assert_eq!(start, 200.0);
                assert_eq!(target, 4000.0);
                assert_eq!(freq, 0.5);
                assert_eq!(curve, ModCurve::Linear);
                assert!(!looping);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_exp() {
        let m = ModChain::parse("200>4000:2e").unwrap();
        match m {
            ModChain::Transition { curve, .. } => assert_eq!(curve, ModCurve::Exponential),
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_smooth() {
        let m = ModChain::parse("200>4000:2s").unwrap();
        match m {
            ModChain::Transition { curve, .. } => assert_eq!(curve, ModCurve::Smooth),
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_swell() {
        let m = ModChain::parse("200>4000:2i").unwrap();
        match m {
            ModChain::Transition { curve, .. } => assert_eq!(curve, ModCurve::Swell),
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_pluck() {
        let m = ModChain::parse("200>4000:2o").unwrap();
        match m {
            ModChain::Transition { curve, .. } => assert_eq!(curve, ModCurve::Pluck),
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_stair() {
        let m = ModChain::parse("200>4000:2p").unwrap();
        match m {
            ModChain::Transition { curve, .. } => assert_eq!(curve, ModCurve::Stair),
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_looping() {
        let m = ModChain::parse("200>4000:2~").unwrap();
        match m {
            ModChain::Transition { looping, .. } => assert!(looping),
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_direction_reversal() {
        let m = ModChain::parse("4000>200:2").unwrap();
        match m {
            ModChain::Transition { start, target, .. } => {
                assert_eq!(start, 4000.0);
                assert_eq!(target, 200.0);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_multi_segment_rejected() {
        assert!(ModChain::parse("200>4000:1>800:2").is_none());
    }

    #[test]
    fn interpolate_swell() {
        let v = interpolate(0.0, 100.0, 0.5, ModCurve::Swell);
        assert!((v - 25.0).abs() < 0.01);
    }

    #[test]
    fn interpolate_pluck() {
        let v = interpolate(0.0, 100.0, 0.5, ModCurve::Pluck);
        assert!((v - 75.0).abs() < 0.01);
    }

    #[test]
    fn interpolate_stair() {
        let v0 = interpolate(0.0, 100.0, 0.0, ModCurve::Stair);
        assert!((v0 - 0.0).abs() < 0.01);
        let v1 = interpolate(0.0, 100.0, 0.99, ModCurve::Stair);
        assert!((v1 - 100.0).abs() < 0.01);
    }

    #[test]
    fn parse_slew_linear() {
        let m = ModChain::parse(">4000:0.2").unwrap();
        match m {
            ModChain::Slew {
                target,
                freq,
                curve,
            } => {
                assert_eq!(target, 4000.0);
                assert_eq!(freq, 5.0);
                assert_eq!(curve, ModCurve::Linear);
            }
            _ => panic!("expected Slew"),
        }
    }

    #[test]
    fn parse_slew_exponential() {
        let m = ModChain::parse(">800:0.5e").unwrap();
        match m {
            ModChain::Slew {
                target,
                freq,
                curve,
            } => {
                assert_eq!(target, 800.0);
                assert_eq!(freq, 2.0);
                assert_eq!(curve, ModCurve::Exponential);
            }
            _ => panic!("expected Slew"),
        }
    }

    #[test]
    fn parse_slew_smooth() {
        let m = ModChain::parse(">1200:0.1s").unwrap();
        match m {
            ModChain::Slew { curve, .. } => assert_eq!(curve, ModCurve::Smooth),
            _ => panic!("expected Slew"),
        }
    }

    #[test]
    fn parse_slew_map_values() {
        let m = ModChain::Slew {
            target: 100.0,
            freq: 5.0,
            curve: ModCurve::Linear,
        };
        let mapped = m.map_values(|v| v * 2.0);
        match mapped {
            ModChain::Slew { target, freq, .. } => {
                assert_eq!(target, 200.0);
                assert_eq!(freq, 5.0);
            }
            _ => panic!("expected Slew"),
        }
    }

    #[test]
    fn parse_slew_invalid() {
        assert!(ModChain::parse(">4000:0").is_none());
        assert!(ModChain::parse(">:0.2").is_none());
    }

    #[test]
    fn parse_invalid() {
        assert!(ModChain::parse("hello").is_none());
        assert!(ModChain::parse("200:4000:0").is_none());
        assert!(ModChain::parse("200~4000:0").is_none());
        assert!(ModChain::parse("200~4000:-1").is_none());
    }

    #[test]
    fn parse_static_value() {
        assert!(ModChain::parse("440").is_none());
        assert!(ModChain::parse("0.5").is_none());
    }

    #[test]
    fn parse_envelope_full() {
        let m = ModChain::parse("200^8000:0.01:0.1:0.5:0.3").unwrap();
        match m {
            ModChain::Envelope {
                min,
                max,
                attack,
                decay,
                sustain,
                release,
            } => {
                assert_eq!(min, 200.0);
                assert_eq!(max, 8000.0);
                assert_eq!(attack, 0.01);
                assert_eq!(decay, 0.1);
                assert_eq!(sustain, 0.5);
                assert_eq!(release, 0.3);
            }
            _ => panic!("expected Envelope"),
        }
    }

    #[test]
    fn parse_envelope_attack_only() {
        let m = ModChain::parse("0^1:0.01").unwrap();
        match m {
            ModChain::Envelope {
                min,
                max,
                attack,
                decay,
                sustain,
                release,
            } => {
                assert_eq!(min, 0.0);
                assert_eq!(max, 1.0);
                assert_eq!(attack, 0.01);
                assert_eq!(decay, 0.0);
                assert_eq!(sustain, 1.0);
                assert_eq!(release, 0.005);
            }
            _ => panic!("expected Envelope"),
        }
    }

    #[test]
    fn parse_envelope_min_max_only() {
        let m = ModChain::parse("200^8000").unwrap();
        match m {
            ModChain::Envelope {
                min, max, attack, ..
            } => {
                assert_eq!(min, 200.0);
                assert_eq!(max, 8000.0);
                assert_eq!(attack, 0.003);
            }
            _ => panic!("expected Envelope"),
        }
    }

    #[test]
    fn parse_envelope_negative() {
        let m = ModChain::parse("-12^12:0.01:0.1:0.0:0.5").unwrap();
        match m {
            ModChain::Envelope {
                min, max, sustain, ..
            } => {
                assert_eq!(min, -12.0);
                assert_eq!(max, 12.0);
                assert_eq!(sustain, 0.0);
            }
            _ => panic!("expected Envelope"),
        }
    }

    #[test]
    fn envelope_map_values() {
        let m = ModChain::Envelope {
            min: 100.0,
            max: 200.0,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.3,
        };
        let mapped = m.map_values(|v| v * 2.0);
        match mapped {
            ModChain::Envelope {
                min, max, attack, ..
            } => {
                assert_eq!(min, 200.0);
                assert_eq!(max, 400.0);
                assert_eq!(attack, 0.01);
            }
            _ => panic!("expected Envelope"),
        }
    }
}
