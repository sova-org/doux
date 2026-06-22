// Ping-pong stereo delay: mono-summed input to L, swapped feedback, R is
// feedback-only. Split from delay.dsp so only the selected algorithm runs.
//
// doux params: a_time = s, b_fb = feedback [0,1].
import("stdfaust.lib");

a_time = hslider("a_time", 0.333, 0, 10, 0.0001);
b_fb   = hslider("b_fb", 0.6, 0, 1, 0.0001) : si.smooth(ba.tau2pole(0.005));

MAXSAMP = 65536;

dsraw = a_time * ma.SR : si.smooth(ba.tau2pole(0.03));
ds    = max(1.0, min(MAXSAMP - 2.0, dsraw));
fb    = max(0.0, min(0.95, b_fb));

ppback(outl, outr) = outr * fb, outl * fb; // feedback for L = R's tail, R = L's tail
ppfwd(fl, fr, l, r) = outl, outr
with {
    mono = (l + r) * 0.5;
    outl = de.fdelay(MAXSAMP, ds, mono + fl);
    outr = de.fdelay(MAXSAMP, ds, fr);
};
process = ppfwd ~ ppback;
