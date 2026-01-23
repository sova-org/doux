use crate::fastmath::ftz;

const NUM_LINES: usize = 8;

// Total delay times in seconds (from zita-rev1 by Fons Adriaensen).
const TDELAY: [f32; NUM_LINES] = [
    0.153129, 0.210389, 0.127837, 0.256891, 0.174713, 0.192303, 0.125000, 0.219991,
];

// Allpass diffuser times in seconds (from zita-rev1).
const TDIFF: [f32; NUM_LINES] = [
    0.020346, 0.024421, 0.031604, 0.027333, 0.022904, 0.029291, 0.013458, 0.019123,
];

const HADAMARD_NORM: f32 = 0.353_553_4; // sqrt(1/8)

fn scale_delay(seconds: f32, sr: f32) -> usize {
    (seconds * sr + 0.5) as usize
}

// In-place Hadamard transform for 8 elements (butterfly structure).
fn hadamard8(x: &mut [f32; NUM_LINES]) {
    for i in (0..8).step_by(2) {
        let a = x[i];
        let b = x[i + 1];
        x[i] = a + b;
        x[i + 1] = a - b;
    }
    for i in (0..8).step_by(4) {
        let a0 = x[i];
        let a1 = x[i + 1];
        let b0 = x[i + 2];
        let b1 = x[i + 3];
        x[i] = a0 + b0;
        x[i + 1] = a1 + b1;
        x[i + 2] = a0 - b0;
        x[i + 3] = a1 - b1;
    }
    let mut tmp = [0.0f32; 8];
    tmp.copy_from_slice(x);
    for i in 0..4 {
        x[i] = tmp[i] + tmp[i + 4];
        x[i + 4] = tmp[i] - tmp[i + 4];
    }
}

#[derive(Clone)]
struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    length: usize,
}

impl DelayLine {
    fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length],
            write_pos: 0,
            length,
        }
    }

    fn read(&self) -> f32 {
        let read_pos = if self.write_pos >= self.length {
            self.write_pos - self.length
        } else {
            self.buffer.len() - (self.length - self.write_pos)
        };
        self.buffer[read_pos]
    }

    fn write(&mut self, value: f32) {
        self.buffer[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

#[derive(Clone)]
struct Allpass {
    buffer: Vec<f32>,
    write_pos: usize,
    coeff: f32,
}

impl Allpass {
    fn new(length: usize, coeff: f32) -> Self {
        Self {
            buffer: vec![0.0; length.max(1)],
            write_pos: 0,
            coeff,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let len = self.buffer.len();
        let read_pos = (self.write_pos + 1) % len;
        let delayed = self.buffer[read_pos];
        let v = input - self.coeff * delayed;
        self.buffer[self.write_pos] = v;
        self.write_pos = (self.write_pos + 1) % len;
        delayed + self.coeff * v
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

// Per-line damping filter (simplified zita-rev1 Filt1).
// Mid-frequency gain `gmf` controls RT60; one-pole lowpass adds HF absorption.
#[derive(Clone)]
struct DampingFilter {
    state: f32,
    delay_time: f32,
    cached_gmf: f32,
    cached_t60: f32,
}

impl DampingFilter {
    fn new(delay_time: f32) -> Self {
        Self {
            state: 0.0,
            delay_time,
            cached_gmf: 1.0,
            cached_t60: 0.0,
        }
    }

    fn process(&mut self, x: f32, t60: f32, damping: f32) -> f32 {
        if t60 != self.cached_t60 {
            self.cached_t60 = t60;
            self.cached_gmf = (0.001f32).powf(self.delay_time / t60);
        }
        let whi = 1.0 - damping * 0.7;
        self.state = ftz(whi * x + (1.0 - whi) * self.state, 0.0001);
        self.cached_gmf * self.state
    }

    fn clear(&mut self) {
        self.state = 0.0;
        self.cached_t60 = 0.0;
    }
}

#[derive(Clone)]
pub struct FdnVerb {
    delays: Vec<DelayLine>,
    diffusers: Vec<Allpass>,
    filters: Vec<DampingFilter>,
    mod_phase: [f32; NUM_LINES],
    base_lengths: [usize; NUM_LINES],
    sr: f32,
}

impl FdnVerb {
    pub fn new(sr: f32) -> Self {
        let mut delays = Vec::with_capacity(NUM_LINES);
        let mut diffusers = Vec::with_capacity(NUM_LINES);
        let mut filters = Vec::with_capacity(NUM_LINES);
        let mut base_lengths = [0usize; NUM_LINES];

        for i in 0..NUM_LINES {
            let diff_len = scale_delay(TDIFF[i], sr);
            let total_len = scale_delay(TDELAY[i], sr);
            let delay_len = total_len - diff_len;
            base_lengths[i] = delay_len;
            let max_len = (delay_len as f32 * 1.5) as usize + 32;
            delays.push(DelayLine::new(max_len));
            delays[i].length = delay_len;
            let coeff = if i & 1 == 0 { 0.6 } else { -0.6 };
            diffusers.push(Allpass::new(diff_len, coeff));
            filters.push(DampingFilter::new(TDELAY[i]));
        }

        let mut mod_phase = [0.0f32; NUM_LINES];
        for (i, phase) in mod_phase.iter_mut().enumerate() {
            *phase = i as f32 / NUM_LINES as f32;
        }

        Self {
            delays,
            diffusers,
            filters,
            mod_phase,
            base_lengths,
            sr,
        }
    }

    pub fn process(
        &mut self,
        input: f32,
        decay: f32,
        damping: f32,
        size: f32,
        modulation: f32,
    ) -> [f32; 2] {
        let damping = damping.clamp(0.0, 0.99);
        let size = size.clamp(0.2, 1.5);
        let mod_depth = modulation.clamp(0.0, 1.0) * 16.0;
        let mod_rate = 0.6 / self.sr;
        let input = ftz(input, 0.0001);

        // Map decay (0-1) to RT60: 0.4s to 15s (exponential)
        let t60 = 0.4 * (37.5f32).powf(decay.clamp(0.0, 1.0));

        // Phase 1: Read from delays, inject input, diffuse
        let mut x = [0.0f32; NUM_LINES];
        for (i, (delay, diff)) in self
            .delays
            .iter_mut()
            .zip(self.diffusers.iter_mut())
            .enumerate()
        {
            // Modulate delay length
            self.mod_phase[i] += mod_rate * (1.0 + 0.12 * i as f32);
            if self.mod_phase[i] >= 1.0 {
                self.mod_phase[i] -= 1.0;
            }
            let phase = self.mod_phase[i] * std::f32::consts::TAU;
            let mod_offset = (phase.sin() * mod_depth) as isize;
            let target_len = (self.base_lengths[i] as f32 * size) as usize;
            let modulated = (target_len as isize + mod_offset).max(1) as usize;
            delay.length = modulated.min(delay.buffer.len() - 1);

            // Read + inject input (zita-rev1 style: input added before diffuser)
            let from_delay = delay.read();
            x[i] = diff.process(from_delay + 0.3 * input);
        }

        // Phase 2: Hadamard mixing
        hadamard8(&mut x);

        // Phase 3: Normalize, filter, write back
        for (i, (delay, filt)) in self
            .delays
            .iter_mut()
            .zip(self.filters.iter_mut())
            .enumerate()
        {
            let scaled = x[i] * HADAMARD_NORM;
            let filtered = filt.process(scaled, t60 * size, damping);
            delay.write(filtered);
        }

        // Stereo output from pre-filter Hadamard outputs
        let left = x[0] + x[2] + x[4] + x[6];
        let right = x[1] + x[3] + x[5] + x[7];

        [left * 0.12, right * 0.12]
    }

    pub fn clear(&mut self) {
        for delay in &mut self.delays {
            delay.clear();
        }
        for diff in &mut self.diffusers {
            diff.clear();
        }
        for filt in &mut self.filters {
            filt.clear();
        }
    }
}
