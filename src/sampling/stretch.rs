//! Phase vocoder for independent pitch and time control of sample playback.

#![allow(clippy::needless_range_loop)]

use std::f32::consts::TAU;
use std::sync::LazyLock;

use crate::dsp;
use crate::dsp::fft;

use super::registry::SampleData;

const WINDOW_SIZE: usize = 1024;
const NUM_BINS: usize = WINDOW_SIZE / 2 + 1;
const HOP_LEN: usize = 256;
const BUF_LEN: usize = 4096;
const BUF_MASK: usize = BUF_LEN - 1;
// Hann^2 COLA sum with 4x overlap = 1.5; normalize by its reciprocal.
const COLA_NORM: f32 = 2.0 / 3.0;

static HANN: LazyLock<[f32; WINDOW_SIZE]> = LazyLock::new(|| {
    let mut w = [0.0f32; WINDOW_SIZE];
    for i in 0..WINDOW_SIZE {
        w[i] = 0.5 * (1.0 - (TAU * i as f32 / WINDOW_SIZE as f32).cos());
    }
    w
});

static OMEGA: LazyLock<[f32; NUM_BINS]> = LazyLock::new(|| {
    let mut o = [0.0f32; NUM_BINS];
    for k in 0..NUM_BINS {
        o[k] = TAU * k as f32 / WINDOW_SIZE as f32;
    }
    o
});

#[derive(Clone, Copy)]
pub struct StretchState {
    output_buf: [[f32; BUF_LEN]; 2],
    prev_phase: [f32; NUM_BINS],
    synth_phase: [f32; NUM_BINS],
    prev_mag: [f32; NUM_BINS],
    flux_avg: f32,
    write_pos: usize,
    read_pos: f64,
    available: i32,
    analysis_pos: f64,
    analysis_consumed: f64,
    region_start: f64,
    region_end: f64,
    has_prev_phase: bool,
    loop_active: bool,
    done: bool,
}

impl Default for StretchState {
    fn default() -> Self {
        Self {
            output_buf: [[0.0; BUF_LEN]; 2],
            prev_phase: [0.0; NUM_BINS],
            synth_phase: [0.0; NUM_BINS],
            prev_mag: [0.0; NUM_BINS],
            flux_avg: 0.0,
            write_pos: 0,
            read_pos: 0.0,
            available: 0,
            analysis_pos: 0.0,
            analysis_consumed: 0.0,
            region_start: 0.0,
            region_end: 0.0,
            has_prev_phase: false,
            loop_active: false,
            done: false,
        }
    }
}

impl StretchState {
    pub fn reset(&mut self, start: f64, end: f64, looping: bool) {
        self.output_buf = [[0.0; BUF_LEN]; 2];
        self.prev_phase = [0.0; NUM_BINS];
        self.synth_phase = [0.0; NUM_BINS];
        self.prev_mag = [0.0; NUM_BINS];
        self.flux_avg = 0.0;
        self.write_pos = 0;
        self.read_pos = 0.0;
        self.available = 0;
        self.analysis_pos = start;
        self.analysis_consumed = 0.0;
        self.region_start = start;
        self.region_end = end;
        self.has_prev_phase = false;
        self.loop_active = looping;
        self.done = false;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.done
    }

    #[inline]
    pub fn needs_init(&self) -> bool {
        !self.has_prev_phase && self.available == 0
    }

    #[inline]
    pub fn ensure_available(&mut self, data: &SampleData, stretch: f32) {
        while self.available < 2 {
            self.produce_frame(data, stretch);
            if self.done {
                return;
            }
        }
    }

    #[inline]
    pub fn read(&self, channel: usize) -> f32 {
        let center = self.read_pos.floor() as usize & BUF_MASK;
        let frac = self.read_pos.fract() as f32;
        let buf = &self.output_buf[channel];
        // Ring buffer: BUF_MASK wraps all 4 taps, including the (-1) lookback.
        let y0 = buf[center.wrapping_sub(1) & BUF_MASK];
        let y1 = buf[center];
        let y2 = buf[(center + 1) & BUF_MASK];
        let y3 = buf[(center + 2) & BUF_MASK];
        crate::dsp::hermite4(y0, y1, y2, y3, frac)
    }

    #[inline]
    pub fn advance(&mut self, pitch_ratio: f64) {
        let old_int = self.read_pos as usize;
        self.read_pos += pitch_ratio;
        let new_int = self.read_pos as usize;
        let consumed = (new_int - old_int) as i32;
        if consumed > 0 {
            self.available -= consumed;
        }
    }

    fn produce_frame(&mut self, data: &SampleData, stretch: f32) {
        if self.done {
            return;
        }

        let region_len = self.region_end - self.region_start;
        if region_len <= 0.0 {
            self.done = true;
            return;
        }

        let analysis_hop = if stretch <= 0.001 {
            0.0
        } else {
            HOP_LEN as f64 / stretch as f64
        };

        let frame_count = data.frame_count as f64;
        let channels = data.channels.min(2) as usize;

        // Phase analysis on channel 0, shared across channels for stereo coherence
        let mut mag_ch0 = [0.0f32; NUM_BINS];
        {
            let (mut re, mut im) = ([0.0f32; WINDOW_SIZE], [0.0f32; WINDOW_SIZE]);
            self.read_windowed(data, 0, frame_count, region_len, &mut re);
            fft::fft(&mut re, &mut im, false);

            let mut a_phase = [0.0f32; NUM_BINS];
            for k in 0..NUM_BINS {
                mag_ch0[k] = (re[k] * re[k] + im[k] * im[k]).sqrt();
                a_phase[k] = dsp::atan2f(im[k], re[k]);
            }

            // Transient detection via spectral flux
            let is_transient = if self.has_prev_phase {
                let mut flux = 0.0f32;
                for k in 0..NUM_BINS {
                    let diff = mag_ch0[k] - self.prev_mag[k];
                    if diff > 0.0 {
                        flux += diff * diff;
                    }
                }
                let transient = self.flux_avg > 0.0 && flux > self.flux_avg * 4.0;
                self.flux_avg += 0.1 * (flux - self.flux_avg);
                transient
            } else {
                false
            };

            if is_transient {
                self.synth_phase = a_phase;
                self.prev_phase = a_phase;
            } else {
                self.advance_phase(&a_phase, analysis_hop as f32, &mag_ch0);
            }

            self.prev_mag = mag_ch0;
            self.synthesize_into(0, &mag_ch0, &mut re, &mut im);
        }

        // Second channel: own magnitudes, shared phase
        if channels == 2 {
            let (mut re, mut im) = ([0.0f32; WINDOW_SIZE], [0.0f32; WINDOW_SIZE]);
            self.read_windowed(data, 1, frame_count, region_len, &mut re);
            fft::fft(&mut re, &mut im, false);

            let mut mag = [0.0f32; NUM_BINS];
            for k in 0..NUM_BINS {
                mag[k] = (re[k] * re[k] + im[k] * im[k]).sqrt();
            }
            self.synthesize_into(1, &mag, &mut re, &mut im);
        } else {
            for i in 0..WINDOW_SIZE {
                let idx = (self.write_pos + i) & BUF_MASK;
                self.output_buf[1][idx] = self.output_buf[0][idx];
            }
        }

        // Clear the next hop region for future overlap-add
        for i in 0..HOP_LEN {
            let idx = (self.write_pos + WINDOW_SIZE + i) & BUF_MASK;
            self.output_buf[0][idx] = 0.0;
            self.output_buf[1][idx] = 0.0;
        }

        self.analysis_pos += analysis_hop;
        self.analysis_consumed += analysis_hop;
        self.write_pos = (self.write_pos + HOP_LEN) & BUF_MASK;
        self.available += HOP_LEN as i32;

        if self.loop_active {
            if self.analysis_pos >= self.region_end {
                let overshoot = self.analysis_pos - self.region_end;
                self.analysis_pos = self.region_start + overshoot.rem_euclid(region_len);
            }
        } else if self.analysis_consumed >= region_len {
            self.done = true;
        }
    }

    fn read_windowed(
        &self,
        data: &SampleData,
        ch: usize,
        frame_count: f64,
        region_len: f64,
        re: &mut [f32; WINDOW_SIZE],
    ) {
        let hann = &*HANN;
        for i in 0..WINDOW_SIZE {
            let mut pos = self.analysis_pos + i as f64;
            if self.loop_active && region_len > 0.0 {
                pos = self.region_start + (pos - self.region_start).rem_euclid(region_len);
            }
            let sample = if pos >= 0.0 && pos < frame_count {
                data.read_interpolated(pos as f32, ch)
            } else {
                0.0
            };
            re[i] = sample * hann[i];
        }
    }

    fn advance_phase(&mut self, a_phase: &[f32; NUM_BINS], ha: f32, mag: &[f32; NUM_BINS]) {
        if !self.has_prev_phase {
            self.prev_phase = *a_phase;
            self.synth_phase = *a_phase;
            self.has_prev_phase = true;
            return;
        }

        let omega = &*OMEGA;
        if ha < 0.001 {
            // Freeze: advance each bin at its center frequency
            for k in 0..NUM_BINS {
                self.synth_phase[k] += omega[k] * HOP_LEN as f32;
            }
        } else {
            for k in 0..NUM_BINS {
                let expected = omega[k] * ha;
                let delta = a_phase[k] - self.prev_phase[k];
                let deviation = dsp::modpi(delta - expected);
                let inst_freq = omega[k] + deviation / ha;
                self.synth_phase[k] += inst_freq * HOP_LEN as f32;
            }
        }

        apply_phase_locking(&mut self.synth_phase, a_phase, mag);
        self.prev_phase = *a_phase;
    }

    fn synthesize_into(
        &mut self,
        ch: usize,
        mag: &[f32; NUM_BINS],
        re: &mut [f32; WINDOW_SIZE],
        im: &mut [f32; WINDOW_SIZE],
    ) {
        for k in 0..NUM_BINS {
            let phase = self.synth_phase[k];
            let (s, c) = (dsp::sinf(phase), dsp::cosf(phase));
            re[k] = mag[k] * c;
            im[k] = mag[k] * s;
        }
        // Conjugate symmetry for real-valued output
        for k in 1..WINDOW_SIZE / 2 {
            re[WINDOW_SIZE - k] = re[k];
            im[WINDOW_SIZE - k] = -im[k];
        }

        fft::fft(re, im, true);

        let hann = &*HANN;
        for i in 0..WINDOW_SIZE {
            let idx = (self.write_pos + i) & BUF_MASK;
            self.output_buf[ch][idx] += re[i] * hann[i] * COLA_NORM;
        }
    }
}

fn apply_phase_locking(
    synth_phase: &mut [f32; NUM_BINS],
    analysis_phase: &[f32; NUM_BINS],
    mag: &[f32; NUM_BINS],
) {
    let mut peak_left = [0u16; NUM_BINS];
    let mut peak_right = [0u16; NUM_BINS];

    let mut last = 0u16;
    for k in 0..NUM_BINS {
        if is_peak(mag, k) {
            last = k as u16;
        }
        peak_left[k] = last;
    }

    last = (NUM_BINS - 1) as u16;
    for k in (0..NUM_BINS).rev() {
        if is_peak(mag, k) {
            last = k as u16;
        }
        peak_right[k] = last;
    }

    for k in 0..NUM_BINS {
        let left = peak_left[k] as usize;
        let right = peak_right[k] as usize;
        let peak = if k - left <= right - k { left } else { right };
        if k != peak {
            synth_phase[k] = synth_phase[peak] + analysis_phase[k] - analysis_phase[peak];
        }
    }
}

#[inline]
fn is_peak(mag: &[f32], k: usize) -> bool {
    if k == 0 || k >= mag.len() - 1 {
        return true;
    }
    mag[k] >= mag[k - 1] && mag[k] >= mag[k + 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_sine_data(frames: u32, freq_hz: f32, sr: f32) -> Arc<SampleData> {
        let mut samples = Vec::with_capacity(frames as usize * 2);
        for i in 0..frames {
            let t = i as f32 / sr;
            let val = (TAU * freq_hz * t).sin();
            samples.push(val);
            samples.push(val);
        }
        Arc::new(SampleData::new(samples, 2, 261.626))
    }

    fn step(st: &mut StretchState, data: &SampleData, stretch: f32, pitch_ratio: f64) -> f32 {
        st.ensure_available(data, stretch);
        let out = st.read(0);
        st.advance(pitch_ratio);
        out
    }

    #[test]
    fn ring_buffer_wrap_around() {
        let data = make_sine_data(8000, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 8000.0, false);

        for _ in 0..BUF_LEN * 3 {
            let s = step(&mut st, &data, 2.0, 1.0);
            assert!(s.is_finite(), "output must be finite");
        }
    }

    #[test]
    fn stretch_1_near_identity() {
        let data = make_sine_data(4096, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 4096.0, false);

        let mut count = 0;
        for _ in 0..10000 {
            if st.is_done() {
                break;
            }
            step(&mut st, &data, 1.0, 1.0);
            count += 1;
        }
        assert!(st.is_done(), "should terminate");
        let expected = 4096;
        assert!(
            count > expected * 3 / 4,
            "too short: {count} vs expected ~{expected}"
        );
        assert!(
            count < expected * 5 / 4,
            "too long: {count} vs expected ~{expected}"
        );
    }

    #[test]
    fn stretch_2_doubles_duration() {
        let data = make_sine_data(4096, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 4096.0, false);

        let mut count = 0;
        for _ in 0..20000 {
            if st.is_done() {
                break;
            }
            step(&mut st, &data, 2.0, 1.0);
            count += 1;
        }
        assert!(st.is_done(), "should terminate");
        let expected = 4096 * 2;
        assert!(
            count > expected * 3 / 4,
            "too short: {count} vs expected ~{expected}"
        );
        assert!(
            count < expected * 5 / 4,
            "too long: {count} vs expected ~{expected}"
        );
    }

    #[test]
    fn stretch_4_quadruples_duration() {
        let data = make_sine_data(4096, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 4096.0, false);

        let mut count = 0;
        for _ in 0..40000 {
            if st.is_done() {
                break;
            }
            step(&mut st, &data, 4.0, 1.0);
            count += 1;
        }
        assert!(st.is_done(), "should terminate");
        let expected = 4096 * 4;
        assert!(
            count > expected * 3 / 4,
            "too short: {count} vs expected ~{expected}"
        );
        assert!(
            count < expected * 5 / 4,
            "too long: {count} vs expected ~{expected}"
        );
    }

    #[test]
    fn freeze_produces_stable_output() {
        let data = make_sine_data(4096, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 4096.0, false);

        let mut samples = Vec::new();
        for _ in 0..2000 {
            samples.push(step(&mut st, &data, 0.0, 1.0));
        }
        assert!(!st.is_done(), "freeze should not end playback");
        assert!(samples.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn non_looping_terminates() {
        let data = make_sine_data(4096, 440.0, 44100.0);
        for &stretch in &[0.5, 1.0, 2.0, 4.0, 5.0] {
            let mut st = StretchState::default();
            st.reset(0.0, 4096.0, false);
            let max_samples = (4096.0 * stretch * 2.0) as usize + 4096;
            for _ in 0..max_samples {
                if st.is_done() {
                    break;
                }
                step(&mut st, &data, stretch, 1.0);
            }
            assert!(st.is_done(), "should have terminated at stretch={stretch}");
        }
    }

    #[test]
    fn stereo_coherence() {
        // Test data has identical L/R channels, so output must match exactly
        let data = make_sine_data(4096, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 4096.0, false);

        for _ in 0..1000 {
            if st.is_done() {
                break;
            }
            st.ensure_available(&data, 1.5);
            let l = st.read(0);
            let r = st.read(1);
            st.advance(1.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
            assert_eq!(l, r, "identical channels must produce identical output");
        }
    }

    #[test]
    fn sine_reconstruction_quality() {
        // With stretch=1 on a pure sine, the output should be close to the input
        let data = make_sine_data(8192, 440.0, 44100.0);
        let mut st = StretchState::default();
        st.reset(0.0, 8192.0, false);

        // Skip the first window to let the overlap-add stabilize
        for _ in 0..WINDOW_SIZE {
            if st.is_done() {
                break;
            }
            step(&mut st, &data, 1.0, 1.0);
        }

        let mut max_abs = 0.0f32;
        for _ in 0..4096 {
            if st.is_done() {
                break;
            }
            let s = step(&mut st, &data, 1.0, 1.0);
            if s.abs() > max_abs {
                max_abs = s.abs();
            }
        }
        assert!(
            max_abs > 0.3,
            "output should have meaningful amplitude, got {max_abs}"
        );
    }
}
