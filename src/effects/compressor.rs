use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "compressor",
    description: "Sidechain compressor (ducking)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "comp",
            aliases: &[],
            description: "duck amount (0 = off, 1 = full)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "compattack",
            aliases: &["cattack"],
            description: "attack time in seconds",
            default: "0.01",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "comprelease",
            aliases: &["crelease"],
            description: "release time in seconds",
            default: "0.15",
            min: 0.0,
            max: 2.0,
        },
        ParamInfo {
            name: "comporbit",
            aliases: &["corbit"],
            description: "sidechain source orbit index",
            default: "0.0",
            min: 0.0,
            max: 7.0,
        },
    ],
};

#[derive(Clone, Copy)]
pub struct CompressorParams {
    pub amount: f32,
    pub attack: f32,
    pub release: f32,
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            amount: 0.0,
            attack: 0.01,
            release: 0.15,
        }
    }
}

#[derive(Default)]
pub struct Compressor {
    env: f32,
    pub params: CompressorParams,
}

impl Compressor {
    pub fn process(&mut self, sidechain_level: f32, attack_coeff: f32, release_coeff: f32) -> f32 {
        let coeff = if sidechain_level > self.env {
            attack_coeff
        } else {
            release_coeff
        };
        self.env += coeff * (sidechain_level - self.env);
        self.env
    }
}
