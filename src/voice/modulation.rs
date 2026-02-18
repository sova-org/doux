use std::f32::consts::PI;

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
pub struct ModSegment {
    pub target: f32,
    pub freq: f32,
    pub curve: ModCurve,
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
        segments: [ModSegment; 4],
        count: u8,
        looping: bool,
    },
}

impl ModChain {
    pub fn parse(s: &str) -> Option<Self> {
        if s.contains('>') {
            Self::parse_transition(s)
        } else if s.contains('~') {
            Self::parse_oscillate(s)
        } else if s.contains('?') {
            Self::parse_random(s)
        } else {
            None
        }
    }

    fn parse_transition(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('>').collect();
        if parts.len() < 2 || parts.len() > 5 {
            return None;
        }
        let start: f32 = parts[0].parse().ok()?;
        let mut segments = [ModSegment { target: 0.0, freq: 1.0, curve: ModCurve::Linear }; 4];
        let mut looping = false;
        let count = (parts.len() - 1) as u8;

        for (i, part) in parts[1..].iter().enumerate() {
            let colon = part.find(':')?;
            let target: f32 = part[..colon].parse().ok()?;
            let dur_str = &part[colon + 1..];

            let (dur_str, is_loop) = if let Some(stripped) = dur_str.strip_suffix('~') {
                (stripped, true)
            } else {
                (dur_str, false)
            };

            if is_loop {
                if i == parts.len() - 2 {
                    looping = true;
                } else {
                    return None;
                }
            }

            let (period, curve) = parse_duration_curve(dur_str)?;
            if period <= 0.0 {
                return None;
            }
            segments[i] = ModSegment { target, freq: 1.0 / period, curve };
        }

        Some(ModChain::Transition { start, segments, count, looping })
    }

    fn parse_oscillate(s: &str) -> Option<Self> {
        let tilde = s.find('~')?;
        let min: f32 = s[..tilde].parse().ok()?;
        let rest = &s[tilde + 1..];
        let colon = rest.find(':')?;
        let max: f32 = rest[..colon].parse().ok()?;
        let period_str = &rest[colon + 1..];

        let (period, shape) = if let Some(stripped) = period_str.strip_suffix('t') {
            (stripped.parse::<f32>().ok()?, ModShape::Triangle)
        } else if let Some(stripped) = period_str.strip_suffix('w') {
            (stripped.parse::<f32>().ok()?, ModShape::Saw)
        } else if let Some(stripped) = period_str.strip_suffix('q') {
            (stripped.parse::<f32>().ok()?, ModShape::Square)
        } else {
            (period_str.parse::<f32>().ok()?, ModShape::Sine)
        };

        if period <= 0.0 {
            return None;
        }
        Some(ModChain::Oscillate { min, max, freq: 1.0 / period, shape })
    }

    fn parse_random(s: &str) -> Option<Self> {
        let q = s.find('?')?;
        let min: f32 = s[..q].parse().ok()?;
        let rest = &s[q + 1..];
        let colon = rest.find(':')?;
        let max: f32 = rest[..colon].parse().ok()?;
        let period_str = &rest[colon + 1..];

        let (period, shape) = if let Some(stripped) = period_str.strip_suffix('s') {
            (stripped.parse::<f32>().ok()?, ModShape::Rand)
        } else if let Some(stripped) = period_str.strip_suffix('d') {
            (stripped.parse::<f32>().ok()?, ModShape::Drunk)
        } else {
            (period_str.parse::<f32>().ok()?, ModShape::Hold)
        };

        if period <= 0.0 {
            return None;
        }
        Some(ModChain::Oscillate { min, max, freq: 1.0 / period, shape })
    }

}

fn parse_duration_curve(s: &str) -> Option<(f32, ModCurve)> {
    if let Some(stripped) = s.strip_suffix('e') {
        Some((stripped.parse().ok()?, ModCurve::Exponential))
    } else if let Some(stripped) = s.strip_suffix('s') {
        Some((stripped.parse().ok()?, ModCurve::Smooth))
    } else {
        Some((s.parse().ok()?, ModCurve::Linear))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ParamId {
    Freq,
    Gain,
    Postgain,
    Pan,
    Speed,
    Detune,
    Pw,
    Sub,
    Harmonics,
    Timbre,
    Morph,
    Scan,
    Lpf,
    Lpq,
    Hpf,
    Hpq,
    Bpf,
    Bpq,
    Llpf,
    Llpq,
    Lhpf,
    Lhpq,
    Lbpf,
    Lbpq,
    Fm,
    Fmh,
    Fm2,
    Fm2h,
    Fmfb,
    Am,
    Amdepth,
    Rm,
    Rmdepth,
    Vib,
    Vibmod,
    Phaser,
    Phaserdepth,
    Phasersweep,
    Phasercenter,
    Flanger,
    Flangerdepth,
    Flangerfeedback,
    Smear,
    Smearfreq,
    Smearfb,
    Chorus,
    Chorusdepth,
    Chorusdelay,
    Fold,
    Crush,
    Coarse,
    Distort,
    Eqlo,
    Eqmid,
    Eqhi,
    Tilt,
    Width,
    Haas,
    Delay,
    Verb,
    Comb,
    Wrap,
    Feedback,
}

#[derive(Clone, Copy)]
pub struct ParamMod {
    pub chain: ModChain,
    pub phase: f32,
    pub segment_idx: u8,
    pub prev_rand: f32,
    pub next_rand: f32,
    pub seed: u32,
    pub drunk_pos: f32,
}

impl Default for ParamMod {
    fn default() -> Self {
        Self {
            chain: ModChain::Oscillate { min: 0.0, max: 0.0, freq: 0.0, shape: ModShape::Sine },
            phase: 0.0,
            segment_idx: 0,
            prev_rand: 0.0,
            next_rand: 0.0,
            seed: 0,
            drunk_pos: 0.5,
        }
    }
}

impl ParamMod {
    pub fn new(chain: ModChain, seed: u32) -> Self {
        let mut m = Self {
            chain,
            phase: 0.0,
            segment_idx: 0,
            prev_rand: 0.0,
            next_rand: 0.0,
            seed,
            drunk_pos: 0.5,
        };
        m.prev_rand = m.rand();
        m.next_rand = m.rand();
        m
    }

    fn rand(&mut self) -> f32 {
        self.seed = lcg(self.seed);
        ((self.seed >> 16) & 0x7fff) as f32 / 32767.0
    }

    pub fn tick(&mut self, isr: f32) -> f32 {
        match self.chain {
            ModChain::Oscillate { min, max, freq, shape } => {
                self.phase += freq * isr;
                self.tick_oscillate(min, max, shape)
            }
            ModChain::Transition { start, segments, count, looping } => {
                self.tick_transition(start, segments, count, looping, isr)
            }
        }
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
                let t = if self.phase < 0.5 { self.phase * 2.0 } else { 2.0 - self.phase * 2.0 };
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
                if self.phase < 0.5 { min } else { max }
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

    fn tick_transition(
        &mut self,
        start: f32,
        segments: [ModSegment; 4],
        count: u8,
        looping: bool,
        isr: f32,
    ) -> f32 {
        let mut first = true;
        loop {
            let idx = self.segment_idx as usize;
            if idx >= count as usize {
                return segments[count as usize - 1].target;
            }

            let seg = &segments[idx];
            if first {
                self.phase += seg.freq * isr;
                first = false;
            }

            if self.phase >= 1.0 {
                if idx + 1 < count as usize {
                    self.phase -= 1.0;
                    self.segment_idx += 1;
                } else if looping {
                    self.phase -= 1.0;
                    self.segment_idx = 0;
                } else {
                    self.phase = 1.0;
                    return seg.target;
                }
                continue;
            }

            let seg_start = if idx == 0 { start } else { segments[idx - 1].target };
            return interpolate(seg_start, seg.target, self.phase, seg.curve);
        }
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
            ModChain::Oscillate { min, max, freq, shape } => {
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
    fn parse_transition_single() {
        let m = ModChain::parse("200>4000:2").unwrap();
        match m {
            ModChain::Transition { start, segments, count, looping } => {
                assert_eq!(start, 200.0);
                assert_eq!(count, 1);
                assert_eq!(segments[0].target, 4000.0);
                assert_eq!(segments[0].freq, 0.5);
                assert_eq!(segments[0].curve, ModCurve::Linear);
                assert!(!looping);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_exp() {
        let m = ModChain::parse("200>4000:2e").unwrap();
        match m {
            ModChain::Transition { segments, .. } => {
                assert_eq!(segments[0].curve, ModCurve::Exponential);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_smooth() {
        let m = ModChain::parse("200>4000:2s").unwrap();
        match m {
            ModChain::Transition { segments, .. } => {
                assert_eq!(segments[0].curve, ModCurve::Smooth);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_multi() {
        let m = ModChain::parse("200>4000:1>800:2").unwrap();
        match m {
            ModChain::Transition { start, segments, count, looping } => {
                assert_eq!(start, 200.0);
                assert_eq!(count, 2);
                assert_eq!(segments[0].target, 4000.0);
                assert_eq!(segments[0].freq, 1.0);
                assert_eq!(segments[1].target, 800.0);
                assert_eq!(segments[1].freq, 0.5);
                assert!(!looping);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_multi_curves() {
        let m = ModChain::parse("200>4000:1e>200:1.5s").unwrap();
        match m {
            ModChain::Transition { segments, count, .. } => {
                assert_eq!(count, 2);
                assert_eq!(segments[0].curve, ModCurve::Exponential);
                assert_eq!(segments[1].curve, ModCurve::Smooth);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_transition_looping() {
        let m = ModChain::parse("200>4000:1>200:1~").unwrap();
        match m {
            ModChain::Transition { looping, count, .. } => {
                assert_eq!(count, 2);
                assert!(looping);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_random_hold() {
        let m = ModChain::parse("200?4000:0.5").unwrap();
        match m {
            ModChain::Oscillate { min, max, shape, .. } => {
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
    fn parse_direction_reversal() {
        let m = ModChain::parse("4000>200:2").unwrap();
        match m {
            ModChain::Transition { start, segments, .. } => {
                assert_eq!(start, 4000.0);
                assert_eq!(segments[0].target, 200.0);
            }
            _ => panic!("expected Transition"),
        }
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
    fn parse_percussive_envelope() {
        let m = ModChain::parse("0>1:0.01>0.7:0.1>0:2").unwrap();
        match m {
            ModChain::Transition { start, segments, count, looping } => {
                assert_eq!(start, 0.0);
                assert_eq!(count, 3);
                assert_eq!(segments[0].target, 1.0);
                assert_eq!(segments[1].target, 0.7);
                assert_eq!(segments[2].target, 0.0);
                assert!(!looping);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn parse_looping_sawtooth() {
        let m = ModChain::parse("200>4000:2~").unwrap();
        match m {
            ModChain::Transition { count, looping, .. } => {
                assert_eq!(count, 1);
                assert!(looping);
            }
            _ => panic!("expected Transition"),
        }
    }
}
