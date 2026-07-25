// 24 dB/oct state-variable filter (lp/hp/bp): two TPT SVF stages cascaded.
// Stage A carries the user resonance; stage B is fixed at q=0.13 (near
// Butterworth) for a clean single-peak rolloff. Restores doux's pre-Faust
// `SvfCascade`, which was two `SvfState`s at q and 0.13 — see dsp/svf.dsp for
// the core, including the 1 - 0.5*q input scaling that keeps the resonant peak
// from running away in loudness.
//
// params: a_cutoff (Hz), b_q in [0,1] (mapped to damping like svf.dsp), c_mode
// 0=lp 1=hp 2=bp. Slider names prefixed a/b/c for stable param order.
import("stdfaust.lib");

a_cutoff = hslider("a_cutoff", 1000, 1, 20000, 0.001);
b_q      = hslider("b_q", 0, 0, 1, 0.001);
c_mode   = hslider("c_mode", 0, 0, 2, 1);

fc   = max(1.0, min(a_cutoff, ma.SR * 0.45));
b_qc = max(0.0, min(1.0, b_q));
md   = int(c_mode);
g    = tan(fc * ma.PI * ma.T);

// Rational tanh — doux's `fast_tanh_f32` (arf/src/fastmath.rs); see dsp/svf.dsp.
sat(x) = xc * (27.0 + x2) / (27.0 + 9.0 * x2)
with {
    xc = max(-3.0, min(3.0, x));
    x2 = xc * xc; // divisor >= 27, never zero
};

// One TPT SVF stage at damping k(q); identical core to dsp/svf.dsp.
stage(q, x) = (loop ~ si.bus(2)) : (si.block(2), sel)
with {
    k  = 1.0 / (0.5 + 29.5 * q * q * sqrt(q)); // divisor >= 0.5
    a1 = 1.0 / (1.0 + g * (g + k)); // g, k > 0, so the divisor is never < 1
    a2 = g * a1;
    a3 = g * a2;
    loop(s1, s2) = ic1, ic2, lp, hp, bp
    with {
        xin = x * (1.0 - 0.5 * q);
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

process = stage(b_qc) : stage(0.13);
