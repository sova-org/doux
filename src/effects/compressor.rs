pub struct Compressor {
    env: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self { env: 0.0 }
    }
}

impl Compressor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, sidechain_level: f32, attack_coeff: f32, release_coeff: f32) -> f32 {
        let coeff = if sidechain_level > self.env { attack_coeff } else { release_coeff };
        self.env += coeff * (sidechain_level - self.env);
        self.env
    }
}
