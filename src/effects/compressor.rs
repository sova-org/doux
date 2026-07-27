use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "compressor",
    description: "Orbit compressor (glue, or ducking from another orbit)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "comp",
            aliases: &[],
            description: "compression amount, dry/wet on the gain (0 = off, 1 = full)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "compthresh",
            aliases: &["cthresh"],
            description: "threshold in dB; the level compression starts at",
            default: "-20.0",
            min: -60.0,
            max: 0.0,
        },
        ParamInfo {
            name: "compratio",
            aliases: &["cratio"],
            description: "compression ratio above the threshold (1 = none)",
            default: "4.0",
            min: 1.0,
            max: 20.0,
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
            description: "sidechain source orbit index; negative = this orbit (glue)",
            default: "-1.0",
            min: -1.0,
            max: 7.0,
        },
    ],
};

/// Default threshold, in dB. Chosen against measured levels rather than studio
/// convention: doux trims every voice by `VOICE_OUTPUT_TRIM` and its sources
/// carry their own headroom, so one voice at `gain 1` peaks between about -19 dB
/// (pluck) and -10 dB (drums). A threshold inside that band, which -12 was,
/// leaves a solo voice barely clearing it and `comp` doing nothing audible.
/// Sitting just under the quietest of them means a lone quiet voice still passes
/// clean while anything loud or stacked actually compresses.
///
/// Deliberately no makeup gain to go with it: the detector may be another orbit,
/// and ducking depends on the gain being exactly unity while that orbit is
/// silent. Makeup would lift the idle state and move the level between hits.
const DEFAULT_THRESH_DB: f32 = -20.0;

#[derive(Clone, Copy)]
pub struct CompressorParams {
    pub amount: f32,
    /// Threshold in dB. Compression starts here; below it the gain is unity.
    pub thresh_db: f32,
    /// Ratio above the threshold. 1 is no compression.
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            amount: 0.0,
            thresh_db: DEFAULT_THRESH_DB,
            ratio: 4.0,
            attack: 0.01,
            release: 0.15,
        }
    }
}

impl CompressorParams {
    /// The block-constant half of the gain law: `(linear threshold, exponent)`.
    /// Hoisted out of the per-sample loop because the dB conversion is itself a
    /// `powf`. The floors keep the divide and the `powf` finite for degenerate
    /// threshold/ratio, the same guard arf's `comp` ugen uses.
    pub fn gain_coeffs(&self) -> (f32, f32) {
        let thresh = crate::dsp::powf(10.0, self.thresh_db / 20.0).max(0.001);
        (thresh, 1.0 / self.ratio.max(1.0) - 1.0)
    }

    /// Feed-forward gain for one detector level, given `gain_coeffs()`. Above
    /// the threshold the output follows `t·(env/t)^(1/ratio)`, which is a gain
    /// of `(env/t)^(1/ratio - 1)`; below it, unity, and the early return keeps
    /// an idle compressor off the `powf`.
    #[inline]
    pub fn gain_for(&self, env: f32, thresh: f32, exponent: f32) -> f32 {
        if env <= thresh {
            return 1.0;
        }
        let gain = crate::dsp::powf(env / thresh, exponent);
        // `amount` is a dry/wet on the gain, so 0 is bypass and 1 the full law.
        1.0 + self.amount * (gain - 1.0)
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

    /// Zero the envelope follower — used when the orbit crosses into silence so a
    /// frozen or denormal duck state flushes to true zero.
    pub fn clear_env(&mut self) {
        self.env = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(amount: f32, thresh_db: f32, ratio: f32) -> CompressorParams {
        CompressorParams {
            amount,
            thresh_db,
            ratio,
            ..Default::default()
        }
    }

    /// The `INFO` table is a string and the struct default is a float, so they
    /// can drift silently. A host reads the first and the engine runs the second.
    #[test]
    fn declared_defaults_match_the_struct() {
        let d = CompressorParams::default();
        for p in INFO.params {
            let declared: f32 = p.default.parse().expect("numeric default");
            let actual = match p.name {
                "comp" => d.amount,
                "compthresh" => d.thresh_db,
                "compratio" => d.ratio,
                "compattack" => d.attack,
                "comprelease" => d.release,
                // -1 is the "sidechain from this orbit" sentinel, not a field.
                "comporbit" => continue,
                other => panic!("undeclared compressor param {other}"),
            };
            assert_eq!(declared, actual, "{} default drifted", p.name);
            assert!(
                (p.min..=p.max).contains(&actual),
                "{} default sits outside its own declared range",
                p.name
            );
        }
    }

    #[test]
    fn below_threshold_is_unity() {
        let p = params(1.0, -12.0, 4.0);
        let (t, e) = p.gain_coeffs();
        // -12 dB is ~0.251 linear.
        assert_eq!(p.gain_for(0.1, t, e), 1.0);
        assert_eq!(p.gain_for(0.25, t, e), 1.0);
    }

    #[test]
    fn above_threshold_follows_the_ratio() {
        let p = params(1.0, -20.0, 4.0);
        let (t, e) = p.gain_coeffs();
        // -20 dB = 0.1 linear. At 10x over, a 4:1 ratio should let through
        // 10^(1/4) = ~1.78x of the threshold, i.e. a gain of ~0.178.
        let gain = p.gain_for(1.0, t, e);
        assert!(
            (gain - 0.1778).abs() < 1e-3,
            "4:1 at 20 dB over should give ~0.178, got {gain}"
        );
    }

    #[test]
    fn ratio_one_is_transparent() {
        let p = params(1.0, -40.0, 1.0);
        let (t, e) = p.gain_coeffs();
        assert!((p.gain_for(0.9, t, e) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn amount_blends_between_bypass_and_the_full_law() {
        let full = params(1.0, -20.0, 4.0);
        let half = params(0.5, -20.0, 4.0);
        let (t, e) = full.gain_coeffs();
        let (gf, gh) = (full.gain_for(1.0, t, e), half.gain_for(1.0, t, e));
        assert!(
            (gh - (1.0 + 0.5 * (gf - 1.0))).abs() < 1e-6,
            "amount must be a straight dry/wet on the gain"
        );
        assert!(gh > gf, "half the amount must duck less");
    }

    // `1 + amount*(g-1)` is unbounded below, so an amount above 1 would return a
    // NEGATIVE gain: the bus phase-inverts and amplifies instead of compressing.
    // The old exponent law was bounded in [0,1] for any positive amount, so the
    // clamp is what keeps the reformulation safe. Enforced in `write_param`,
    // which both the static and ModChain paths now go through.
    #[test]
    fn amount_above_one_cannot_invert_the_bus() {
        let mut orbit = crate::orbit::Orbit::new(48_000.0, 0);
        orbit.write_param(crate::orbit::OrbitParamId::Comp, 5.0);
        assert_eq!(orbit.comp.params.amount, 1.0);
        let p = CompressorParams {
            amount: orbit.comp.params.amount,
            thresh_db: -30.0,
            ratio: 8.0,
            ..Default::default()
        };
        let (t, e) = p.gain_coeffs();
        for env in [0.05, 0.5, 1.0, 4.0, 40.0] {
            let g = p.gain_for(env, t, e);
            assert!(
                (0.0..=1.0).contains(&g),
                "env {env}: gain {g} left the unit range"
            );
        }
    }

    #[test]
    fn a_degenerate_ratio_cannot_expand() {
        // `ratio` below 1 would flip the exponent positive and boost instead.
        let p = params(1.0, -20.0, 0.0);
        let (t, e) = p.gain_coeffs();
        assert!(p.gain_for(1.0, t, e) <= 1.0);
        assert!(p.gain_for(1.0, t, e).is_finite());
    }
}
