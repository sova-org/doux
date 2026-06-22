// Tape stereo delay: per-channel feedback with a one-pole LP (DAMP=0.35) in the
// loop, for the darkening repeats of a tape echo. Split from delay.dsp so only
// the selected algorithm runs.
//
// doux params: a_time = s, b_fb = feedback [0,1].
import("stdfaust.lib");

a_time = hslider("a_time", 0.333, 0, 10, 0.0001);
b_fb   = hslider("b_fb", 0.6, 0, 1, 0.0001) : si.smooth(ba.tau2pole(0.005));

MAXSAMP = 65536;

dsraw = a_time * ma.SR : si.smooth(ba.tau2pole(0.03));
ds    = max(1.0, min(MAXSAMP - 2.0, dsraw));
fb    = max(0.0, min(0.95, b_fb));

tapeLP(x) = 0.35 * x : fi.pole(0.65);
chan      = (+ : de.fdelay(MAXSAMP, ds)) ~ (tapeLP : *(fb));
process = chan, chan;
