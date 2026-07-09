// State-variable filter (lp/hp/bp) replacing doux's basic SvfState.
// params: cutoff (Hz), q in [0,1] (mapped to filter Q), mode 0=lp 1=hp 2=bp.
// Slider names prefixed a/b/c so Faust's alphabetical param order is stable.
import("stdfaust.lib");
a_cutoff = hslider("a_cutoff", 1000, 1, 20000, 0.001);
b_q      = hslider("b_q", 0, 0, 1, 0.001);
c_mode   = hslider("c_mode", 0, 0, 2, 1);
fc = max(1.0, min(a_cutoff, ma.SR * 0.45));
b_qc = max(0.0, min(1.0, b_q));
Q  = 0.5 + b_qc * 30.0;
process = _ <: (fi.svf.lp(fc, Q), fi.svf.hp(fc, Q), fi.svf.bp(fc, Q)) : selector
with {
    selector(lp, hp, bp) = select3(int(c_mode), lp, hp, bp);
};
