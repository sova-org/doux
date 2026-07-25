// State-variable filter (lp/hp/bp) — Zavalishin TPT core with a saturated
// resonance path. Restores doux's pre-Faust `SvfState`, whose level discipline
// was lost when this file first wrapped the linear `fi.svf`.
//
// Resonance maps q in [0,1] to Q = 0.5 + 29.5*q^2.5, damping k = 1/Q. The
// exponent keeps the lower half gentle (q=0.2 is barely peaked, matching the
// pre-Faust 2*(1-q)^3.5 curve to within a few percent) while the top reaches a
// singing Q=30 instead of running away — that curve passed Q=1500 by q=0.9,
// which made its top third unusable. The saturator on the bandpass state still
// bounds the peak, so high q sings rather than blows up.
//
// The input is pre-scaled by 1 - 0.5*q so the resonant peak does not run away
// in loudness as q rises — without it, the default q alone is +16 dB.
//
// params: a_cutoff (Hz), b_q in [0,1] (mapped to damping), c_mode 0=lp 1=hp
// 2=bp. Slider names prefixed a/b/c so Faust's alphabetical param order is
// stable.
import("stdfaust.lib");
a_cutoff = hslider("a_cutoff", 1000, 1, 20000, 0.001);
b_q      = hslider("b_q", 0, 0, 1, 0.001);
c_mode   = hslider("c_mode", 0, 0, 2, 1);

fc   = max(1.0, min(a_cutoff, ma.SR * 0.45));
b_qc = max(0.0, min(1.0, b_q));
md   = int(c_mode);

// Coefficients depend only on sliders, so Faust hoists them out of the loop.
g  = tan(fc * ma.PI * ma.T);
k  = 1.0 / (0.5 + 29.5 * b_qc * b_qc * sqrt(b_qc)); // divisor >= 0.5
a1 = 1.0 / (1.0 + g * (g + k)); // g, k > 0, so the divisor is never < 1
a2 = g * a1;
a3 = g * a2;

// Rational tanh — doux's `fast_tanh_f32` (arf/src/fastmath.rs): odd-symmetric,
// unit slope at 0, ~5 cycles against 20-50 for libm tanh. This runs once per
// sample on every voice, so the approximation is the point.
sat(x) = xc * (27.0 + x2) / (27.0 + 9.0 * x2)
with {
    xc = max(-3.0, min(3.0, x));
    x2 = xc * xc; // divisor >= 27, never zero
};

process(x) = (loop ~ si.bus(2)) : (si.block(2), sel)
with {
    loop(s1, s2) = ic1, ic2, lp, hp, bp
    with {
        xin = x * (1.0 - 0.5 * b_qc);
        v3  = xin - s2;
        v1  = sat(a1 * s1 + a2 * v3); // saturated bandpass state
        v2  = s2 + a2 * s1 + a3 * v3;
        ic1 = 2.0 * v1 - s1;
        ic2 = 2.0 * v2 - s2;
        lp  = v2;
        hp  = xin - k * v1 - v2;
        bp  = v1;
    };
    sel(lp, hp, bp) = select3(md, lp, hp, bp);
};
