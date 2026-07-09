// 24 dB/oct state-variable filter (lp/hp/bp): two `fi.svf` stages cascaded.
// Stage A carries the user resonance; stage B is fixed near-Butterworth for a
// clean single-peak rolloff (replaces doux's hand-written `SvfCascade`).
//
// params: a_cutoff (Hz), b_q in [0,1] (mapped to Q like the 12 dB svf), c_mode
// 0=lp 1=hp 2=bp. Slider names prefixed a/b/.. for stable param order.
import("stdfaust.lib");

a_cutoff = hslider("a_cutoff", 1000, 1, 20000, 0.001);
b_q      = hslider("b_q", 0, 0, 1, 0.001);
c_mode   = hslider("c_mode", 0, 0, 2, 1);

fc = max(1.0, min(a_cutoff, ma.SR * 0.45));
b_qc = max(0.0, min(1.0, b_q));
Qa = 0.5 + b_qc * 30.0; // stage A: user resonance (same mapping as svf.dsp)
Qb = 0.707;            // stage B: Butterworth, clean 24 dB rolloff
md = int(c_mode);

stage(Q) = _ <: (fi.svf.lp(fc, Q), fi.svf.hp(fc, Q), fi.svf.bp(fc, Q)) : sel
with {
    sel(lp, hp, bp) = select3(md, lp, hp, bp);
};

process = stage(Qa) : stage(Qb);
