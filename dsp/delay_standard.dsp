// Standard stereo delay: independent per-channel feedback delay. One of the four
// delay algorithms, each split into its own DSP so the FaustDelay wrapper runs
// ONLY the selected one (the old single delay.dsp ran all four every block, ~4x
// the cost). Linear fdelay matches the native DelayLine read interpolation.
//
// doux params: a_time = s, b_fb = feedback [0,1].
import("stdfaust.lib");

a_time = hslider("a_time", 0.333, 0, 10, 0.0001);
b_fb   = hslider("b_fb", 0.6, 0, 1, 0.0001) : si.smooth(ba.tau2pole(0.005));

MAXSAMP = 65536; // native MAX_DELAY_SAMPLES; ~1.36 s at 48 kHz, ~0.68 s at 96 kHz

// 30 ms one-pole time slew (native TIME_SLEW_SECS), then clamp to the buffer.
dsraw = a_time * ma.SR : si.smooth(ba.tau2pole(0.03));
ds    = max(1.0, min(MAXSAMP - 2.0, dsraw));
fb    = max(0.0, min(0.95, b_fb));

chan = (+ : de.fdelay(MAXSAMP, ds)) ~ *(fb);
process = chan, chan;
