use crate::types::{ModuleInfo, ModuleGroup, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "compressor",
    description: "Sidechain compressor (ducking)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo { name: "comp", aliases: &[], description: "duck amount (0 = off, 1 = full)", default: "0.0", min: 0.0, max: 1.0 },
        ParamInfo { name: "compattack", aliases: &["cattack"], description: "attack time in seconds", default: "0.01", min: 0.0, max: 1.0 },
        ParamInfo { name: "comprelease", aliases: &["crelease"], description: "release time in seconds", default: "0.15", min: 0.0, max: 2.0 },
        ParamInfo { name: "comporbit", aliases: &["corbit"], description: "sidechain source orbit index", default: "0.0", min: 0.0, max: 7.0 },
    ],
};

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
