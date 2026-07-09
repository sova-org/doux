// 3-band EQ: low shelf, mid peak, high shelf. Gains in dB, freqs in Hz.
// Slider names prefixed a/b/c so Faust's alphabetical param order is stable.
import("stdfaust.lib");
a_lo_db  = hslider("a_lo_db", 0, -24, 24, 0.001);
b_lo_f   = hslider("b_lo_f", 200, 20, 2000, 0.001);
c_mid_db = hslider("c_mid_db", 0, -24, 24, 0.001);
d_mid_f  = hslider("d_mid_f", 1000, 100, 10000, 0.001);
e_hi_db  = hslider("e_hi_db", 0, -24, 24, 0.001);
f_hi_f   = hslider("f_hi_f", 5000, 1000, 20000, 0.001);
g_mid_q  = hslider("g_mid_q", 0.7, 0.2, 8, 0.001);
// Clamp freqs into (0, Nyquist) and q > 0: the coefficient math divides by
// tan(pi*f/SR), sin(2pi*f/SR) and q, all of which reach 0 at the band edges
// and blow the recursion to NaN (latches in the master DC-blocker till restart).
lo_f  = max(1.0, min(b_lo_f, ma.SR * 0.45));
mid_f = max(1.0, min(d_mid_f, ma.SR * 0.45));
hi_f  = max(1.0, min(f_hi_f, ma.SR * 0.45));
mid_q = max(0.2, min(8.0, g_mid_q));
process = fi.lowshelf(1, a_lo_db, lo_f)
        : fi.peak_eq_cq(c_mid_db, mid_f, mid_q)
        : fi.highshelf(1, e_hi_db, hi_f);
