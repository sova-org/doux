// Moog-style 4-pole ladder filter, multimode (LP/HP/BP). Zavalishin TPT ladder
// (cf. Faust `ve.lowpassLadder4`) with all four stage taps exposed, mixed with
// binomial coefficients for highpass/bandpass outputs.
//
// doux params: a_cutoff (Hz), b_q in [0,1] -> feedback k in [0,4], c_mode
// 0=lp 1=hp 2=bp. Slider names prefixed a/b/.. for stable param order.
import("stdfaust.lib");

a_cutoff = hslider("a_cutoff", 1000, 1, 20000, 0.001);
b_q      = hslider("b_q", 0.2, 0, 1, 0.001);
c_mode   = hslider("c_mode", 0, 0, 2, 1);

fc = max(20.0, min(a_cutoff, ma.SR * 0.45));
k  = 4.0 * max(0.0, min(1.0, b_q)); // feedback 0..4 (4 = self-oscillation)
md = int(c_mode);

process(x) = (loop ~ si.bus(4)) : (si.block(4), mix)
with {
    loop(s0, s1, s2, s3) = u0, u1, u2, u3, LP0, LP1, LP2, LP3
    with {
        g = tan(fc * ma.PI * ma.T);
        G = g / (1.0 + g);
        omg = 1.0 - G;
        gG = G * G * G * G;
        gS = G * (G * (G * (omg * s0) + (omg * s1)) + (omg * s2)) + (omg * s3);
        u = (x - k * gS) / (1.0 + k * gG);
        v0 = G * (u - s0);
        LP0 = v0 + s0;
        u0 = v0 + LP0;
        v1 = G * (LP0 - s1);
        LP1 = v1 + s1;
        u1 = v1 + LP1;
        v2 = G * (LP1 - s2);
        LP2 = v2 + s2;
        u2 = v2 + LP2;
        v3 = G * (LP2 - s3);
        LP3 = v3 + s3;
        u3 = v3 + LP3;
    };
    // LP = 4th stage; HP/BP from binomial tap mixing of the four stage outputs.
    mix(lp0, lp1, lp2, lp3) = select3(md,
        lp3,
        x - 4.0 * lp0 + 6.0 * lp1 - 4.0 * lp2 + lp3,
        4.0 * lp1 - 8.0 * lp2 + 4.0 * lp3);
};
