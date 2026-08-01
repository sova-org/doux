//! Granular reader for sample playback.
//!
//! The other half of [`super::stretch`]. Where the phase vocoder tries to be
//! transparent, this one tiles the playback region with short windowed grains:
//! the scan head walks the region at whatever rate `stretch` asks for, grains
//! launch from wherever the head is (plus `spray` of scatter, in position and
//! in stereo placement alike), and each grain reads at its own pitch. Time and
//! pitch come apart the same way, but the result is a cloud rather than a
//! faithful slowdown.

use crate::dsp::fastmath::ms_to_samples;
use crate::types::CHANNELS;

use super::registry::SampleData;
use super::stretch::HANN;

/// Grains alive at once, and therefore the ceiling on `dens`.
pub const MAX_GRAINS: usize = 8;

/// One grain in flight.
#[derive(Clone, Copy)]
struct Grain {
    /// Read position within the region, in frames from `region_start`.
    pos: f64,
    /// Window phase. `>= 1.0` means the slot is free.
    phase: f32,
    /// Window phase advanced per output sample.
    step: f32,
    /// Equal-power placement, normalized so centre is exactly unity.
    gain_l: f32,
    gain_r: f32,
}

impl Grain {
    const IDLE: Self = Self {
        pos: 0.0,
        phase: 1.0,
        step: 0.0,
        gain_l: 1.0,
        gain_r: 1.0,
    };
}

/// Cloud shape for one output sample.
///
/// Built fresh each sample so `grain`/`spray`/`dens`/`stretch` modulation lands,
/// and so every clamp lives in one place instead of in the reader's hot loop.
#[derive(Clone, Copy)]
pub struct Cloud {
    /// Grain length in samples.
    size: f32,
    /// Scatter, 0-1. Moves each grain's start position within the region and
    /// its placement across the stereo field by the same amount.
    spray: f32,
    /// Overlapping grains, 1..=[`MAX_GRAINS`].
    dens: usize,
    /// Frames the scan head advances per output sample. 0 = frozen.
    scan: f64,
    /// Driven head position, 0-1 across the region. `Some` overrides `scan`.
    head: Option<f32>,
    /// Frames a grain advances per output sample.
    pitch: f64,
}

impl Cloud {
    /// All comparisons here are NaN-safe by construction: `f32::max` returns the
    /// non-NaN operand, a `>` test sends NaN down the frozen arm, and a NaN
    /// float-to-int cast saturates to 0 before the integer clamp.
    // `clamp` is the one thing that would break that: it propagates NaN.
    #[allow(clippy::manual_clamp)]
    pub fn new(
        grain_ms: f32,
        spray: f32,
        dens: f32,
        stretch: f32,
        head: Option<f32>,
        pitch: f64,
        sr: f32,
    ) -> Self {
        Self {
            // A one-sample Hann grain is a zero, so never go below two.
            size: ms_to_samples(grain_ms, sr).max(2.0),
            spray: spray.max(0.0).min(1.0),
            dens: (dens as i32).clamp(1, MAX_GRAINS as i32) as usize,
            // Same 0.001 freeze cutoff the phase vocoder uses, so "frozen" means
            // one thing whichever reader is running.
            scan: if stretch > 0.001 {
                1.0 / stretch as f64
            } else {
                0.0
            },
            head: head.map(|t| t.max(0.0).min(1.0)),
            pitch: if pitch.is_finite() { pitch } else { 0.0 },
        }
    }
}

/// Per-voice granular state. Small enough (~200 bytes) to live inline on the
/// voice, unlike the boxed phase-vocoder state next door.
#[derive(Clone, Copy)]
pub struct GrainState {
    grains: [Grain; MAX_GRAINS],
    /// Scan head, absolute frames.
    head: f64,
    region_start: f64,
    region_end: f64,
    /// Output samples until the next launch.
    countdown: f32,
    seed: u32,
    looping: bool,
    /// Head ran past the region; live grains are still ringing out.
    exhausted: bool,
    primed: bool,
}

impl Default for GrainState {
    fn default() -> Self {
        Self {
            grains: [Grain::IDLE; MAX_GRAINS],
            head: 0.0,
            region_start: 0.0,
            region_end: 0.0,
            countdown: 0.0,
            seed: 0x9E37_79B9,
            looping: false,
            exhausted: false,
            primed: false,
        }
    }
}

impl GrainState {
    /// Arms the cloud over a region. `seed` comes from the voice so two voices
    /// on the same sample scatter differently.
    pub fn reset(&mut self, start: f64, end: f64, looping: bool, seed: u32) {
        self.grains = [Grain::IDLE; MAX_GRAINS];
        self.head = start;
        self.region_start = start;
        self.region_end = end;
        self.countdown = 0.0;
        // 0 is a fixed point of xorshift.
        self.seed = if seed == 0 { 0x9E37_79B9 } else { seed };
        self.looping = looping;
        self.exhausted = false;
        self.primed = true;
    }

    /// False until [`Self::reset`] has armed the region.
    #[inline]
    pub fn is_primed(&self) -> bool {
        self.primed
    }

    /// The head has run out and the last grain has faded.
    #[inline]
    pub fn is_done(&self) -> bool {
        self.exhausted && self.grains.iter().all(|g| g.phase >= 1.0)
    }

    /// Produces one output frame.
    pub fn tick(&mut self, data: &SampleData, cloud: Cloud, out: &mut [f32; CHANNELS]) {
        let region_len = self.region_end - self.region_start;
        if region_len <= 0.0 {
            self.exhausted = true;
            *out = [0.0; CHANNELS];
            return;
        }

        // Launch. One grain every size/dens samples keeps `dens` of them alive.
        // The floor at 1 both bounds this loop and caps launches at one per
        // sample, which is all a grain shorter than `dens` samples can support.
        let interval = (cloud.size / cloud.dens as f32).max(1.0);
        self.countdown -= 1.0;
        while self.countdown <= 0.0 {
            if !self.exhausted {
                self.launch(cloud, region_len);
            }
            self.countdown += interval;
        }

        // A driven head is placed, not advanced, and never exhausts: the caller
        // is free to sweep past either edge and come back, so the voice has to
        // outlive the region and die on its envelope instead.
        match cloud.head {
            Some(t) => self.head = self.region_start + t as f64 * region_len,
            None => {
                self.head += cloud.scan;
                if self.looping {
                    self.head =
                        self.region_start + (self.head - self.region_start).rem_euclid(region_len);
                } else if self.head >= self.region_end {
                    self.exhausted = true;
                }
            }
        }

        let hann = &*HANN;
        let last = hann.len() - 1;
        let span = last as f32;
        let (mut l, mut r) = (0.0f32, 0.0f32);
        for g in &mut self.grains {
            if g.phase >= 1.0 {
                continue;
            }
            let x = g.phase * span;
            let i = (x as usize).min(last);
            let w = if i == last {
                hann[last]
            } else {
                let f = x - i as f32;
                hann[i] + f * (hann[i + 1] - hann[i])
            };

            let read = (self.region_start + g.pos) as f32;
            let s = data.read_interpolated_stereo(read);
            l += s[0] * w * g.gain_l;
            r += s[1] * w * g.gain_r;

            g.pos += cloud.pitch;
            // Fast path is two compares; the modulo runs once per wrap.
            if g.pos < 0.0 || g.pos >= region_len {
                g.pos = g.pos.rem_euclid(region_len);
            }
            g.phase += g.step;
        }

        // Power-preserving: a sprayed cloud holds its loudness as `dens` rises.
        // The orderly case (spray 0) sums coherently and runs hotter, which the
        // source prescale and the master limiter absorb.
        let norm = 1.0 / (0.5 * cloud.dens as f32).sqrt();
        out[0] = l * norm;
        out[1] = r * norm;
    }

    fn launch(&mut self, cloud: Cloud, region_len: f64) {
        // Take the grain closest to silence, the same policy `steal_voice_slot`
        // applies to voices one level up. Free slots sit at `phase >= 1.0` and
        // win outright, so steady state is unaffected; when every slot is busy
        // (a `grain` sweep downward outruns the long grains still fading) this
        // is the quietest cut available instead of an arbitrary one.
        let slot = self
            .grains
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.phase.total_cmp(&b.phase))
            .map(|(i, _)| i)
            .unwrap_or(0);
        // Both draws happen whatever `spray` is, so the stream does not depend
        // on which scatter is switched on.
        let jitter = (cloud.spray * (self.rand() * 2.0 - 1.0)) as f64 * region_len;
        let pan = 0.5 + cloud.spray * (self.rand() * 2.0 - 1.0) * 0.5;
        self.grains[slot] = Grain {
            pos: (self.head - self.region_start + jitter).rem_euclid(region_len),
            phase: 0.0,
            step: 1.0 / cloud.size,
            // `2(1-p)` and `2p` sum to a constant 2, and at the centre both are
            // `sqrt(1)`. Exactly unity, so `spray 0` is bit-identical to no
            // placement at all rather than 3 dB down.
            gain_l: (2.0 * (1.0 - pan)).sqrt(),
            gain_r: (2.0 * pan).sqrt(),
        };
    }

    #[inline]
    fn rand(&mut self) -> f32 {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        (x >> 8) as f32 / 16_777_216.0 // 24-bit, [0,1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;
    use std::sync::Arc;

    const SR: f32 = 44100.0;

    fn sine(frames: u32, freq_hz: f32) -> Arc<SampleData> {
        let mut samples = Vec::with_capacity(frames as usize * 2);
        for i in 0..frames {
            let v = (TAU * freq_hz * i as f32 / SR).sin();
            samples.push(v);
            samples.push(v);
        }
        Arc::new(SampleData::new(samples, 2, 261.626))
    }

    fn cloud(grain_ms: f32, spray: f32, dens: f32, stretch: f32) -> Cloud {
        Cloud::new(grain_ms, spray, dens, stretch, None, 1.0, SR)
    }

    fn armed(start: f64, end: f64, looping: bool) -> GrainState {
        let mut g = GrainState::default();
        g.reset(start, end, looping, 12345);
        g
    }

    #[test]
    fn default_is_not_primed() {
        assert!(!GrainState::default().is_primed());
        assert!(armed(0.0, 100.0, false).is_primed());
    }

    #[test]
    fn orderly_cloud_reconstructs_the_source() {
        // spray 0, dens 2, stretch 1 is Hann COLA at 50% overlap: the cloud
        // should hand back roughly what it read.
        let data = sine(8192, 440.0);
        let mut st = armed(0.0, 8192.0, false);
        let c = cloud(20.0, 0.0, 2.0, 1.0);

        let mut frame = [0.0; CHANNELS];
        // Skip the ramp-in, where fewer than `dens` grains are alive.
        for _ in 0..2048 {
            st.tick(&data, c, &mut frame);
        }
        let mut peak = 0.0f32;
        for _ in 0..4096 {
            st.tick(&data, c, &mut frame);
            peak = peak.max(frame[0].abs());
        }
        assert!(peak > 0.5, "orderly cloud lost the source: peak {peak}");
        assert!(peak < 1.6, "orderly cloud blew up: peak {peak}");
    }

    #[test]
    fn output_stays_finite_across_the_range() {
        let data = sine(4096, 440.0);
        for &size in &[0.001f32, 0.5, 5.0, 50.0, 500.0] {
            for &spray in &[0.0f32, 0.3, 1.0] {
                for &dens in &[1.0f32, 2.0, 8.0] {
                    for &stretch in &[0.0f32, 0.25, 1.0, 8.0] {
                        let mut st = armed(0.0, 4096.0, true);
                        let c = cloud(size, spray, dens, stretch);
                        let mut frame = [0.0; CHANNELS];
                        for _ in 0..4000 {
                            st.tick(&data, c, &mut frame);
                            assert!(
                                frame[0].is_finite() && frame[1].is_finite(),
                                "non-finite at size={size} spray={spray} dens={dens} stretch={stretch}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn nan_params_do_not_poison_the_output() {
        let data = sine(4096, 440.0);
        for head in [None, Some(f32::NAN)] {
            let mut st = armed(0.0, 4096.0, false);
            let c = Cloud::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN, head, f64::NAN, SR);
            let mut frame = [0.0; CHANNELS];
            for _ in 0..1000 {
                st.tick(&data, c, &mut frame);
                assert!(frame[0].is_finite() && frame[1].is_finite());
            }
        }
    }

    #[test]
    fn frozen_head_never_finishes() {
        let data = sine(4096, 440.0);
        let mut st = armed(0.0, 4096.0, false);
        let c = cloud(50.0, 0.1, 4.0, 0.0);
        let mut frame = [0.0; CHANNELS];
        for _ in 0..20000 {
            st.tick(&data, c, &mut frame);
        }
        assert!(!st.is_done(), "a frozen cloud must sustain until its gate");
    }

    #[test]
    fn driven_head_never_finishes() {
        // A swept head runs off the end and back. `stretch` is 1 here, so the
        // natural motion would have exhausted the region long before the budget.
        let data = sine(4096, 440.0);
        let mut st = armed(0.0, 4096.0, false);
        let mut frame = [0.0; CHANNELS];
        for i in 0..20000 {
            let t = (i % 1000) as f32 / 1000.0;
            let c = Cloud::new(20.0, 0.0, 2.0, 1.0, Some(t), 1.0, SR);
            st.tick(&data, c, &mut frame);
            assert!(!st.is_done(), "a driven cloud must sustain until its gate");
        }
    }

    #[test]
    fn driven_head_lands_where_it_is_told() {
        let (start, end) = (2048.0, 3072.0);
        let data = sine(4096, 440.0);
        let mut st = armed(start, end, false);
        let mut frame = [0.0; CHANNELS];
        for &(t, want) in &[(0.0, 2048.0), (0.5, 2560.0), (1.0, 3072.0), (0.25, 2304.0)] {
            let c = Cloud::new(20.0, 0.0, 2.0, 1.0, Some(t), 1.0, SR);
            st.tick(&data, c, &mut frame);
            assert_eq!(st.head, want, "head at scan {t}");
        }
    }

    #[test]
    fn non_looping_terminates() {
        let data = sine(4096, 440.0);
        for &stretch in &[0.5f32, 1.0, 2.0, 4.0] {
            let mut st = armed(0.0, 4096.0, false);
            let c = cloud(20.0, 0.5, 4.0, stretch);
            let mut frame = [0.0; CHANNELS];
            let budget = (4096.0 * stretch) as usize + 8192;
            let mut ticks = 0;
            while !st.is_done() && ticks < budget {
                st.tick(&data, c, &mut frame);
                ticks += 1;
            }
            assert!(st.is_done(), "did not terminate at stretch={stretch}");
        }
    }

    #[test]
    fn looping_never_finishes() {
        let data = sine(1024, 440.0);
        let mut st = armed(0.0, 1024.0, true);
        let c = cloud(10.0, 0.5, 4.0, 1.0);
        let mut frame = [0.0; CHANNELS];
        for _ in 0..20000 {
            st.tick(&data, c, &mut frame);
            assert!(!st.is_done());
        }
    }

    #[test]
    fn grains_stay_inside_the_region() {
        // begin 0.5, end 0.75 of a 4096-frame sample.
        let (start, end) = (2048.0, 3072.0);
        let data = sine(4096, 440.0);
        let mut st = armed(start, end, true);
        let c = Cloud::new(30.0, 1.0, 8.0, 0.5, None, 2.0, SR);
        let mut frame = [0.0; CHANNELS];
        for _ in 0..8000 {
            st.tick(&data, c, &mut frame);
            for g in &st.grains {
                if g.phase >= 1.0 {
                    continue;
                }
                let abs = st.region_start + g.pos;
                assert!(
                    abs >= start && abs < end,
                    "grain escaped the region: {abs} not in [{start}, {end})"
                );
            }
        }
    }

    #[test]
    fn empty_region_is_silent_and_done() {
        let data = sine(4096, 440.0);
        let mut st = armed(1000.0, 1000.0, false);
        let c = cloud(20.0, 0.0, 2.0, 1.0);
        let mut frame = [1.0; CHANNELS];
        st.tick(&data, c, &mut frame);
        assert_eq!(frame, [0.0; CHANNELS]);
        assert!(st.is_done());
    }

    /// The whole path, wire to speaker: `grain` on the URL has to reach the
    /// granular branch and come back as audio.
    #[test]
    fn engine_renders_a_granular_voice() {
        use crate::offline::{create_engine, render_to_buffer, OfflineEngineConfig};

        let render = |cmd: &str| {
            let config = OfflineEngineConfig::default();
            let mut engine = create_engine(config, None).expect("engine");
            let sr = config.sample_rate;

            let frames = sr as u32;
            let mut pcm = Vec::with_capacity(frames as usize * 2);
            for i in 0..frames {
                let v = (TAU * 440.0 * i as f32 / sr).sin();
                pcm.push(v);
                pcm.push(v);
            }
            // The registry alone is not enough: `get_registry_sample` resolves
            // the bare name through the index first, and the index keys samples
            // as `folder/n`, which is also the registry key.
            engine
                .sample_registry()
                .insert("grtest/0".into(), Arc::new(SampleData::new(pcm, 2, 261.626)));
            engine.set_sample_index(vec![super::super::SampleEntry {
                path: Arc::new(std::path::PathBuf::from("grtest/0.wav")),
                name: "grtest/0".into(),
            }]);

            engine.evaluate(cmd);
            render_to_buffer(&mut engine, 0.5)
                .output
                .expect("captured output")
        };

        let granular = render("/sound/grtest/grain/40/spray/0.5/dens/6/stretch/4/gate/1");
        assert!(
            granular.iter().all(|s| s.is_finite()),
            "granular voice went NaN"
        );
        let peak = granular.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(peak > 0.01, "granular voice was silent: peak {peak}");

        // Same event without `grain` takes the phase vocoder. If the two agree,
        // the granular branch was never entered.
        let vocoded = render("/sound/grtest/spray/0.5/dens/6/stretch/4/gate/1");
        assert!(
            granular
                .iter()
                .zip(&vocoded)
                .any(|(g, v)| (g - v).abs() > 1e-6),
            "grain did not change the reader"
        );
    }

    /// The whole point of normalizing the placement law to unity at centre: a
    /// cloud with no spray must be exactly as loud, and exactly as centred, as
    /// one with no placement at all.
    #[test]
    fn zero_spray_leaves_the_image_untouched() {
        let data = sine(4096, 440.0); // identical L and R
        let mut st = armed(0.0, 4096.0, true);
        let c = cloud(20.0, 0.0, 4.0, 1.0);
        let mut frame = [0.0; CHANNELS];
        for _ in 0..4000 {
            st.tick(&data, c, &mut frame);
            assert_eq!(
                frame[0], frame[1],
                "unsprayed grains must stay dead centre, bit for bit"
            );
        }
        for g in &st.grains {
            assert_eq!(g.gain_l, 1.0);
            assert_eq!(g.gain_r, 1.0);
        }
    }

    #[test]
    fn spray_places_grains_across_the_field() {
        // Source has identical channels, so any L/R difference is placement.
        let data = sine(4096, 440.0);
        let mut st = armed(0.0, 4096.0, true);
        let c = cloud(20.0, 1.0, 4.0, 1.0);
        let mut frame = [0.0; CHANNELS];
        let mut widest = 0.0f32;
        for _ in 0..4000 {
            st.tick(&data, c, &mut frame);
            widest = widest.max((frame[0] - frame[1]).abs());
        }
        assert!(widest > 0.05, "spray did not widen the cloud: {widest}");
    }

    /// Equal power: the two gains always square-sum to 2, wherever a grain lands.
    #[test]
    fn placement_is_equal_power() {
        let data = sine(4096, 440.0);
        for &spray in &[0.0f32, 0.25, 0.5, 1.0] {
            let mut st = armed(0.0, 4096.0, true);
            let c = cloud(20.0, spray, 8.0, 1.0);
            let mut frame = [0.0; CHANNELS];
            for _ in 0..2000 {
                st.tick(&data, c, &mut frame);
                for g in &st.grains {
                    let power = g.gain_l * g.gain_l + g.gain_r * g.gain_r;
                    assert!(
                        (power - 2.0).abs() < 1e-5,
                        "placement lost power at spray={spray}: {power}"
                    );
                }
            }
        }
    }

    /// A grain that is still sounding must not have its slot reclaimed.
    /// Sweeping `grain` downward is the documented gesture that used to do
    /// exactly that: the launch rate climbs while the long grains already in
    /// the air are still fading, so the allocator wrapped onto one mid-window
    /// and cut a Hann peak straight to zero.
    #[test]
    fn shrinking_grain_size_does_not_truncate_a_sounding_grain() {
        // DC source, so every sample-to-sample change in the output is the
        // grain envelopes moving and a truncation shows up as a bare step.
        let data = Arc::new(SampleData::new(vec![1.0; 16384 * 2], 2, 261.626));
        let mut st = armed(0.0, 16384.0, true);
        let mut frame = [0.0; CHANNELS];

        let long = cloud(80.0, 0.0, 2.0, 1.0);
        for _ in 0..4000 {
            st.tick(&data, long, &mut frame);
        }

        let short = cloud(5.0, 0.0, 2.0, 1.0);
        let mut prev = frame[0];
        let mut worst = 0.0f32;
        for _ in 0..4000 {
            st.tick(&data, short, &mut frame);
            worst = worst.max((frame[0] - prev).abs());
            prev = frame[0];
        }
        // Round-robin scored 0.601 here, a truncated grain dumping its whole
        // window value in one sample. Stealing the most-faded slot scores
        // 0.0141, which is just a 5 ms Hann's own per-sample motion, so the
        // bound sits between the two and nearer the floor.
        assert!(
            worst < 0.03,
            "a sounding grain was cut off: step of {worst} in one sample"
        );
    }

    /// At a fixed `grain` a slot is always free when a launch comes due, so the
    /// cloud must stay smooth for every density. DC again, so the only thing
    /// that can move the output is a grain envelope.
    #[test]
    fn steady_state_never_reclaims_a_sounding_grain() {
        let data = Arc::new(SampleData::new(vec![1.0; 8192 * 2], 2, 261.626));
        for &dens in &[1.0f32, 2.0, 4.0, 8.0] {
            let c = cloud(20.0, 0.3, dens, 1.0);
            let mut st = armed(0.0, 8192.0, true);
            let mut frame = [0.0; CHANNELS];
            let mut prev = 0.0f32;
            let mut worst = 0.0f32;
            for _ in 0..8000 {
                st.tick(&data, c, &mut frame);
                worst = worst.max((frame[0] - prev).abs());
                prev = frame[0];
            }
            assert!(
                worst < 0.05,
                "steady state stomped a grain at dens={dens}: step of {worst}"
            );
        }
    }

    #[test]
    fn spray_scatters_and_zero_spray_does_not() {
        // Frozen head, so any spread between live grains is either their own
        // travel (bounded by one grain length) or spray.
        let data = sine(4096, 440.0);
        let size = ms_to_samples(10.0, SR) as f64;

        let spread = |spray: f32, dens: f32| {
            let mut st = armed(0.0, 4096.0, true);
            let c = cloud(10.0, spray, dens, 0.0);
            let mut frame = [0.0; CHANNELS];
            for _ in 0..2000 {
                st.tick(&data, c, &mut frame);
            }
            let live: Vec<f64> = st
                .grains
                .iter()
                .filter(|g| g.phase < 1.0)
                .map(|g| g.pos)
                .collect();
            let hi = live.iter().copied().fold(f64::MIN, f64::max);
            let lo = live.iter().copied().fold(f64::MAX, f64::min);
            hi - lo
        };

        let orderly = spread(0.0, 2.0);
        assert!(
            orderly <= size,
            "unsprayed grains spread past one grain length: {orderly} > {size}"
        );
        let scattered = spread(1.0, 8.0);
        assert!(
            scattered > size * 4.0,
            "spray did not scatter: {scattered} vs grain length {size}"
        );
    }
}

