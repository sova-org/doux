// Multitap stereo delay: single line per channel, 4 fractional taps, fixed 0.5
// feedback from tap1; b_fb sets the tap-spacing "swing". Split from delay.dsp so
// only the selected algorithm runs.
//
// doux params: a_time = s, b_fb = feedback / tap-swing [0,1].
import("stdfaust.lib");

a_time = hslider("a_time", 0.333, 0, 10, 0.0001);
b_fb   = hslider("b_fb", 0.6, 0, 1, 0.0001) : si.smooth(ba.tau2pole(0.005));

MAXSAMP = 65536;

dsraw = a_time * ma.SR : si.smooth(ba.tau2pole(0.03));
ds    = max(1.0, min(MAXSAMP - 2.0, dsraw));
fb    = max(0.0, min(0.95, b_fb));

mt_t1 = ds;
mt_t2 = max(1.0, ds * (0.5 + fb * 0.167));
mt_t3 = max(1.0, ds * (0.25 + fb * 0.083));
mt_t4 = max(1.0, ds * (0.125 + fb * 0.042));
// (fbk, l) -> (tap1, sum); fbk is tap1*0.5 supplied by the `~` loop.
mtraw(fbk, l) = tap1, total
with {
    written = l + fbk;
    tap1 = de.fdelay(MAXSAMP, mt_t1, written);
    total = tap1
          + de.fdelay(MAXSAMP, mt_t2, written) * 0.7
          + de.fdelay(MAXSAMP, mt_t3, written) * 0.5
          + de.fdelay(MAXSAMP, mt_t4, written) * 0.35;
};
chan = (mtraw ~ *(0.5)) : (!, _); // feed tap1*0.5 back, output total only
process = chan, chan;
