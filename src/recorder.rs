use crate::sampling::{SampleData, SampleRegistry};
use crate::types::CHANNELS;
use std::sync::Arc;

const MAX_RECORD_SECONDS: usize = 60;

enum State {
    Idle,
    Recording,
    Overdubbing,
}

pub struct Recorder {
    buffer: Vec<f32>,
    write_pos: usize,
    max_len: usize,
    state: State,
    name: String,
    counter: usize,
    target_orbit: Option<usize>,
}

impl Recorder {
    pub fn new(sr: f32) -> Self {
        let max_len = MAX_RECORD_SECONDS * sr as usize * CHANNELS;
        Self {
            buffer: Vec::with_capacity(max_len),
            write_pos: 0,
            max_len,
            state: State::Idle,
            name: String::new(),
            counter: 0,
            target_orbit: None,
        }
    }

    pub fn target_orbit(&self) -> Option<usize> {
        self.target_orbit
    }

    pub fn toggle(
        &mut self,
        name: Option<&str>,
        overdub: bool,
        target_orbit: Option<usize>,
        registry: &SampleRegistry,
    ) -> Option<String> {
        match self.state {
            State::Idle => {
                let rec_name = match name {
                    Some(n) => n.to_string(),
                    None => {
                        let n = format!("rec{}", self.counter);
                        self.counter += 1;
                        n
                    }
                };

                if overdub {
                    self.buffer.clear();
                    let key = format!("{rec_name}/0");
                    if let Some(data) = registry.get(&key) {
                        let src = &data.frames;
                        self.buffer.extend_from_slice(src);
                    }
                    self.write_pos = 0;
                    self.state = State::Overdubbing;
                } else {
                    self.buffer.clear();
                    self.state = State::Recording;
                }

                self.name = rec_name;
                self.target_orbit = target_orbit;
                None
            }
            State::Recording | State::Overdubbing => {
                self.state = State::Idle;
                self.target_orbit = None;
                Some(self.name.clone())
            }
        }
    }

    #[inline]
    pub fn capture_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        match self.state {
            State::Idle => {}
            State::Recording => self.record_block(output, block_samples, output_channels),
            State::Overdubbing => self.overdub_block(output, block_samples, output_channels),
        }
    }

    #[inline]
    fn record_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        let remaining = self.max_len - self.buffer.len();
        if remaining == 0 {
            return;
        }

        if output_channels == CHANNELS {
            let n = (block_samples * CHANNELS).min(remaining);
            self.buffer.extend_from_slice(&output[..n]);
        } else {
            let max_frames = remaining / CHANNELS;
            let frames = block_samples.min(max_frames);
            for i in 0..frames {
                let base = i * output_channels;
                self.buffer.push(output[base]);
                self.buffer.push(output[base + 1]);
            }
        }
    }

    #[inline]
    fn overdub_block(&mut self, output: &[f32], block_samples: usize, output_channels: usize) {
        let buf_len = self.buffer.len();
        if buf_len == 0 {
            self.record_block(output, block_samples, output_channels);
            return;
        }

        for i in 0..block_samples {
            let base = i * output_channels;
            let l = output[base];
            let r = output[base + 1];

            if self.write_pos >= buf_len {
                self.write_pos = 0;
            }

            self.buffer[self.write_pos] += l;
            self.buffer[self.write_pos + 1] += r;
            self.write_pos += CHANNELS;
        }
    }

    pub fn finalize(&mut self) -> Option<(String, Arc<SampleData>)> {
        if self.buffer.is_empty() {
            return None;
        }
        let data = SampleData::new(self.buffer.clone(), CHANNELS as u8, 261.626);
        Some((self.name.clone(), Arc::new(data)))
    }
}
