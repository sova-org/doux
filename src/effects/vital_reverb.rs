use crate::dsp::ftz;

const NUM_CONTAINERS: usize = 4;
const CONTAINER_SIZE: usize = 4;
const NUM_LINES: usize = NUM_CONTAINERS * CONTAINER_SIZE;
const BASE_SR: f32 = 44100.0;
const ALLPASS_COEFF: f32 = 0.6;
const MAX_PREDELAY_SEC: f32 = 0.3;
const SQRT2: f32 = std::f32::consts::SQRT_2;

// Feedback delay lengths in samples at 44100Hz (per container, per line).
const FEEDBACK_DELAYS: [[f32; CONTAINER_SIZE]; NUM_CONTAINERS] = [
    [6753.2, 9278.4, 7704.5, 11328.5],
    [9701.12, 5512.5, 8480.45, 5638.65],
    [3120.73, 3429.5, 3626.37, 7713.52],
    [4521.54, 6518.97, 5265.56, 5630.25],
];

// Allpass delays in samples at 44100Hz.
const ALLPASS_DELAYS: [[usize; CONTAINER_SIZE]; NUM_CONTAINERS] = [
    [1001, 799, 933, 876],
    [895, 807, 907, 853],
    [957, 1019, 711, 567],
    [833, 779, 663, 997],
];

// LFO sign per container: +1 or -1 (containers 0/2 positive, 1/3 negative).
const LFO_SIGN: [f32; NUM_CONTAINERS] = [1.0, -1.0, 1.0, -1.0];

fn next_pow2(n: usize) -> usize {
    n.next_power_of_two()
}

fn midikey2hz(key: f32) -> f32 {
    440.0 * 2.0f32.powf((key - 69.0) / 12.0)
}

// 4-point Lagrange cubic interpolation into a power-of-2 buffer.
fn lagrange_read(buf: &[f32], mask: usize, write_pos: usize, delay: f32) -> f32 {
    let d = delay.max(1.0);
    let i = d as usize;
    let frac = d - i as f32;

    let idx = |offset: usize| buf[(write_pos.wrapping_sub(i + offset)) & mask];
    let s0 = idx(0);
    let s1 = idx(1);
    let s2 = idx(2);
    let s3 = idx(3);

    // Cubic Lagrange coefficients.
    let fm1 = frac - 1.0;
    let fm2 = frac - 2.0;
    let fp1 = frac + 1.0;
    let c0 = -frac * fm1 * fm2 * (1.0 / 6.0);
    let c1 = fp1 * fm1 * fm2 * 0.5;
    let c2 = -fp1 * frac * fm2 * 0.5;
    let c3 = fp1 * frac * fm1 * (1.0 / 6.0);

    c0 * s3 + c1 * s2 + c2 * s1 + c3 * s0
}

// One-pole lowpass: state = state + coeff * (input - state)
fn onepole_lp(state: &mut f32, input: f32, coeff: f32) -> f32 {
    *state += coeff * (input - *state);
    *state
}

// One-pole highpass: input - lowpass(input)
fn onepole_hp(state: &mut f32, input: f32, coeff: f32) -> f32 {
    *state += coeff * (input - *state);
    input - *state
}

// Convert a frequency to a one-pole coefficient (bilinear approximation).
fn freq_to_coeff(freq: f32, sr: f32) -> f32 {
    let w = std::f32::consts::PI * freq / sr;
    (2.0 * w) / (1.0 + 2.0 * w)
}

// Low-shelf filter: boosts/cuts below cutoff.
// gain_db < 0 means cut. Returns filtered sample.
fn low_shelf(state: &mut f32, input: f32, coeff: f32, gain_linear: f32) -> f32 {
    let lp = onepole_lp(state, input, coeff);
    let hp = input - lp;
    lp * gain_linear + hp
}

// High-shelf filter: boosts/cuts above cutoff.
fn high_shelf(state: &mut f32, input: f32, coeff: f32, gain_linear: f32) -> f32 {
    let lp = onepole_lp(state, input, coeff);
    let hp = input - lp;
    lp + hp * gain_linear
}

fn db2linear(db: f32) -> f32 {
    10.0f32.powf(db * 0.05)
}

// Map 0-1 normalized param using vital's MIDI key mapping: key 16-135 -> Hz.
fn param_to_freq(p: f32) -> f32 {
    let key = 16.0 + p * (135.0 - 16.0);
    midikey2hz(key)
}

#[derive(Clone)]
pub struct VitalVerb {
    // Pre-delay (stereo, but we process mono input).
    predelay_buf: Vec<f32>,
    predelay_mask: usize,

    // Pre-filter state (HP + LP).
    pre_hp_state: f32,
    pre_lp_state: f32,

    // 16 feedback delay lines (modulated).
    delay_bufs: [Vec<f32>; NUM_LINES],
    delay_masks: [usize; NUM_LINES],

    // 16 allpass comb filters.
    allpass_bufs: [Vec<f32>; NUM_LINES],
    allpass_write: [usize; NUM_LINES],

    // Shelf filter states (low + high per line).
    shelf_low_state: [f32; NUM_LINES],
    shelf_high_state: [f32; NUM_LINES],

    // LFO phases (2 quadrature pairs, but we track per-line).
    lfo_phase1: f32,
    lfo_phase2: f32,

    // Feedback signals circulating in the loop.
    feedback: [f32; NUM_LINES],

    write_pos: usize,
    sr: f32,
}

impl VitalVerb {
    pub fn new(sr: f32) -> Self {
        // Max predelay buffer.
        let max_predelay = (MAX_PREDELAY_SEC * sr) as usize + 4;
        let pd_size = next_pow2(max_predelay);

        // Allocate delay lines large enough for 2x size multiplier.
        let max_size_mult = 2.0;
        let sr_ratio = sr / BASE_SR;

        let delay_bufs = std::array::from_fn(|line| {
            let c = line / CONTAINER_SIZE;
            let l = line % CONTAINER_SIZE;
            let max_delay = (FEEDBACK_DELAYS[c][l] * max_size_mult * sr_ratio) as usize + 8;
            vec![0.0; next_pow2(max_delay)]
        });
        let delay_masks = std::array::from_fn(|i| delay_bufs[i].len() - 1);

        let allpass_bufs = std::array::from_fn(|line| {
            let c = line / CONTAINER_SIZE;
            let l = line % CONTAINER_SIZE;
            let len = ((ALLPASS_DELAYS[c][l] as f32 * max_size_mult * sr_ratio) as usize + 4).max(1);
            vec![0.0; next_pow2(len)]
        });

        Self {
            predelay_buf: vec![0.0; pd_size],
            predelay_mask: pd_size - 1,
            pre_hp_state: 0.0,
            pre_lp_state: 0.0,
            delay_bufs,
            delay_masks,
            allpass_bufs,
            allpass_write: [0; NUM_LINES],
            shelf_low_state: [0.0; NUM_LINES],
            shelf_high_state: [0.0; NUM_LINES],
            lfo_phase1: 0.0,
            lfo_phase2: 0.0,
            feedback: [0.0; NUM_LINES],
            write_pos: 0,
            sr,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        input: f32,
        decay: f32,      // 0-1: reverb time
        damp: f32,        // 0-1: high-frequency damping (inverted for high_gain)
        predelay: f32,    // 0-1: pre-delay amount
        size: f32,        // 0-1: room size / diffusion
        prelow: f32,      // 0-1: pre-filter low cutoff
        prehigh: f32,     // 0-1: pre-filter high cutoff
        lowcut: f32,      // 0-1: shelf filter low cutoff
        highcut: f32,     // 0-1: shelf filter high cutoff
        lowgain: f32,     // 0-1: shelf low gain
        chorus_amt: f32,  // 0-1: chorus/modulation amount
        chorus_freq: f32, // 0-1: chorus LFO frequency
    ) -> [f32; 2] {
        let sr = self.sr;
        let sr_ratio = sr / BASE_SR;
        let wp = self.write_pos;

        // Clamp all parameters to [0, 1] range for safety.
        let decay = decay.clamp(0.0, 1.0);
        let damp = damp.clamp(0.0, 1.0);
        let predelay = predelay.clamp(0.0, 1.0);
        let size = size.clamp(0.0, 1.0);
        let prelow = prelow.clamp(0.0, 1.0);
        let prehigh = prehigh.clamp(0.0, 1.0);
        let lowcut = lowcut.clamp(0.0, 1.0);
        let highcut = highcut.clamp(0.0, 1.0);
        let lowgain = lowgain.clamp(0.0, 1.0);
        let chorus_amt = chorus_amt.clamp(0.0, 1.0);
        let chorus_freq = chorus_freq.clamp(0.0, 1.0);

        // --- Parameter mapping (vital formulas) ---

        // Decay: exp(remap(0,1,-6,6)) -> clamp [0.1, 100] seconds -> * sr
        let decay_sec = (-6.0 + decay * 12.0).exp().clamp(0.1, 100.0);
        let decay_samples = decay_sec * sr;

        // High gain (damping): invert damp so high damp = more HF absorption.
        let high_gain_db = (1.0 - damp) * -24.0;
        let high_gain_linear = db2linear(high_gain_db);

        // Low gain in feedback path.
        let low_gain_db = lowgain * -24.0;
        let low_gain_linear = db2linear(low_gain_db);

        // Size multiplier: 2^lerp(size, -3, 1).
        let size_exp = -3.0 + size * 4.0;
        let size_mult = 2.0f32.powf(size_exp);

        // Pre-delay in samples.
        let predelay_samples = predelay * MAX_PREDELAY_SEC * sr;

        // Pre-filter frequencies.
        let prelow_freq = param_to_freq(prelow);
        let prehigh_freq = param_to_freq(prehigh);
        let prelow_coeff = freq_to_coeff(prelow_freq, sr);
        let prehigh_coeff = freq_to_coeff(prehigh_freq, sr);

        // Shelf filter frequencies in feedback path.
        let lowcut_freq = param_to_freq(lowcut);
        let highcut_freq = param_to_freq(highcut);
        let lowcut_coeff = freq_to_coeff(lowcut_freq, sr);
        let highcut_coeff = freq_to_coeff(highcut_freq, sr);

        // Chorus: x^2 * 2500 * sr_ratio * size_mult.
        let chorus_depth = chorus_amt * chorus_amt * 2500.0 * sr_ratio * size_mult;

        // Chorus frequency: exp(remap(0,1,-8,3)) Hz, clamp 16Hz.
        let chorus_hz = (-8.0 + chorus_freq * 11.0).exp().min(16.0);
        let lfo_inc = chorus_hz / sr;

        // --- Step 1: Write input to predelay, read back ---
        self.predelay_buf[wp & self.predelay_mask] = ftz(input, 1e-18);
        let predelayed = lagrange_read(
            &self.predelay_buf,
            self.predelay_mask,
            wp,
            predelay_samples,
        );

        // --- Step 2: Pre-filter (HP -> LP) and scale by 1/4 ---
        let hp_out = onepole_hp(&mut self.pre_hp_state, predelayed, prelow_coeff);
        let prefiltered = onepole_lp(&mut self.pre_lp_state, hp_out, prehigh_coeff) * 0.25;

        // --- Step 3: Add pre-filtered input to feedback signals ---
        let mut x = [0.0f32; NUM_LINES];
        for (xi, &fb) in x.iter_mut().zip(self.feedback.iter()) {
            *xi = fb + prefiltered;
        }

        // --- Step 4: Allpass comb filters ---
        for (line, xi) in x.iter_mut().enumerate() {
            let c = line / CONTAINER_SIZE;
            let l = line % CONTAINER_SIZE;
            let ap_delay = (ALLPASS_DELAYS[c][l] as f32 * size_mult * sr_ratio).max(1.0) as usize;
            let buf = &mut self.allpass_bufs[line];
            let mask = buf.len() - 1;
            let aw = self.allpass_write[line];

            let read_pos = (aw + buf.len() - ap_delay) & mask;
            let delayed = buf[read_pos];
            let v = *xi - ALLPASS_COEFF * delayed;
            buf[aw & mask] = v;
            self.allpass_write[line] = (aw + 1) & mask;
            *xi = delayed + ALLPASS_COEFF * v;
        }

        // --- Step 5: Feedback matrix ---
        // Decomposed as: identity + other_feedback + adjacent_feedback.
        // global_avg = sum(all 16) / 16
        // container_sum[c] = sum of 4 lines in container c
        // other_fb[i] = global_avg - 0.5 * container_sum[container_of(i)]
        // adjacent_fb[i] = -0.5 * container_sum[container_of(i)] (broadcast)
        // But the Faust code does: result = x[i] + other_fb + adjacent_fb
        // which simplifies to a specific mixing pattern.

        let global_sum: f32 = x.iter().sum();
        let global_avg = global_sum / NUM_LINES as f32;

        let mut container_sums = [0.0f32; NUM_CONTAINERS];
        for c in 0..NUM_CONTAINERS {
            for l in 0..CONTAINER_SIZE {
                container_sums[c] += x[c * CONTAINER_SIZE + l];
            }
        }

        let mut matrix_out = [0.0f32; NUM_LINES];
        for (line, (mo, &xi)) in matrix_out.iter_mut().zip(x.iter()).enumerate() {
            let c = line / CONTAINER_SIZE;
            let other_fb = global_avg - 0.5 * container_sums[c] / CONTAINER_SIZE as f32;
            let adjacent_fb = -0.5 * container_sums[c] / CONTAINER_SIZE as f32;
            *mo = xi + other_fb + adjacent_fb;
        }

        // --- Step 6: Extract stereo output from post-matrix signals ---
        // Sum to stereo with channel alternation, then swap and apply sqrt(2) gain.
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for (line, &mo) in matrix_out.iter().enumerate() {
            if line % 2 == 0 {
                left += mo;
            } else {
                right += mo;
            }
        }
        // Scale (the Faust code uses 2/4 = 0.5 per channel, then sqrt(2) makeup).
        left *= 0.5 * SQRT2 / NUM_LINES as f32 * CONTAINER_SIZE as f32;
        right *= 0.5 * SQRT2 / NUM_LINES as f32 * CONTAINER_SIZE as f32;

        // --- Step 7: Shelf filters in feedback path ---
        for (line, (lo_st, hi_st)) in self
            .shelf_low_state
            .iter_mut()
            .zip(self.shelf_high_state.iter_mut())
            .enumerate()
        {
            matrix_out[line] =
                low_shelf(lo_st, matrix_out[line], lowcut_coeff, low_gain_linear);
            matrix_out[line] =
                high_shelf(hi_st, matrix_out[line], highcut_coeff, high_gain_linear);
        }

        // --- Step 8: Per-line T60 decay ---
        for (line, mo) in matrix_out.iter_mut().enumerate() {
            let c = line / CONTAINER_SIZE;
            let l = line % CONTAINER_SIZE;
            let delay_len = FEEDBACK_DELAYS[c][l] * size_mult * sr_ratio;
            let decay_coeff = 0.001f32.powf(delay_len / decay_samples);
            *mo *= decay_coeff;
        }

        // --- Step 9: Advance LFOs ---
        self.lfo_phase1 += lfo_inc;
        if self.lfo_phase1 >= 1.0 {
            self.lfo_phase1 -= 1.0;
        }
        self.lfo_phase2 += lfo_inc * 1.0; // Same rate, 90-degree offset achieved by phase.
        if self.lfo_phase2 >= 1.0 {
            self.lfo_phase2 -= 1.0;
        }

        // --- Step 10: Write to modulated delay lines, read back as feedback ---
        for (line, &mo) in matrix_out.iter().enumerate() {
            let c = line / CONTAINER_SIZE;
            let l = line % CONTAINER_SIZE;
            let base_delay = FEEDBACK_DELAYS[c][l] * size_mult * sr_ratio;

            // Quadrature LFO: containers 0/1 use lfo1, containers 2/3 use lfo2.
            let phase_offset = line as f32 / NUM_LINES as f32;
            let lfo_base = if c < 2 {
                self.lfo_phase1
            } else {
                self.lfo_phase2
            };
            let phase = (lfo_base + phase_offset).fract() * std::f32::consts::TAU;
            let lfo_val = phase.sin() * LFO_SIGN[c];
            let mod_delay = base_delay + lfo_val * chorus_depth;

            // Write to delay line.
            let buf = &mut self.delay_bufs[line];
            let mask = self.delay_masks[line];
            buf[wp & mask] = ftz(mo, 1e-18);

            // Read with Lagrange interpolation.
            self.feedback[line] = lagrange_read(buf, mask, wp, mod_delay);
        }

        self.write_pos = wp + 1;

        // Flush denormals.
        [ftz(left, 1e-18), ftz(right, 1e-18)]
    }

    pub fn clear(&mut self) {
        self.predelay_buf.fill(0.0);
        self.pre_hp_state = 0.0;
        self.pre_lp_state = 0.0;
        for buf in &mut self.delay_bufs {
            buf.fill(0.0);
        }
        for buf in &mut self.allpass_bufs {
            buf.fill(0.0);
        }
        self.allpass_write = [0; NUM_LINES];
        self.shelf_low_state = [0.0; NUM_LINES];
        self.shelf_high_state = [0.0; NUM_LINES];
        self.lfo_phase1 = 0.0;
        self.lfo_phase2 = 0.0;
        self.feedback = [0.0; NUM_LINES];
        self.write_pos = 0;
    }
}
