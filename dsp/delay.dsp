// Delay: stereo multi-algorithm delay (standard, ping-pong, tape, multitap).
// Replaces effects::delay::Delay. Output is wet only; the orbit scales the send
// by the delay level and sums the wet back onto the bus.
//
// doux params: a_time = s, b_fb = feedback [0,1], c_type (0 standard,
// 1 ping-pong, 2 tape, 3 multitap).
//
// COST: all four algorithms are instantiated and run every block; c_type only
// selects which output is kept (Faust has no cheap runtime branch over stateful
// blocks). Multitap alone is 4 taps x 2 channels = 8 delay lines. This is ~4x
// the delay-line memory/CPU of the native version — the documented port cost.
import("stdfaust.lib");

a_time = hslider("a_time", 0.333, 0, 10, 0.0001);
b_fb   = hslider("b_fb", 0.6, 0, 1, 0.0001) : si.smooth(ba.tau2pole(0.005));
c_type = hslider("c_type", 0, 0, 3, 1);

MAXSAMP = 65536; // native MAX_DELAY_SAMPLES; ~1.36 s at 48 kHz, ~0.68 s at 96 kHz

// 30 ms one-pole time slew (native TIME_SLEW_SECS), then clamp to the buffer.
dsraw = a_time * ma.SR : si.smooth(ba.tau2pole(0.03));
ds    = max(1.0, min(MAXSAMP - 2.0, dsraw));
fb    = max(0.0, min(0.95, b_fb));

// Standard: independent per-channel feedback delay. Linear fdelay matches the
// native DelayLine read interpolation.
stdchan  = (+ : de.fdelay(MAXSAMP, ds)) ~ *(fb);
standard = stdchan, stdchan;

// Ping-pong: mono-summed input to L, swapped feedback, R is feedback-only.
ppback(outl, outr) = outr * fb, outl * fb; // feedback for L = R's tail, R = L's tail
ppfwd(fl, fr, l, r) = outl, outr
with {
    mono = (l + r) * 0.5;
    outl = de.fdelay(MAXSAMP, ds, mono + fl);
    outr = de.fdelay(MAXSAMP, ds, fr);
};
pingpong = ppfwd ~ ppback;

// Tape: per-channel feedback with a one-pole LP (DAMP=0.35) in the loop.
tapeLP(x) = 0.35 * x : fi.pole(0.65);
tapechan  = (+ : de.fdelay(MAXSAMP, ds)) ~ (tapeLP : *(fb));
tape      = tapechan, tapechan;

// Multitap: single line per channel, 4 fractional taps, fixed 0.5 feedback from
// tap1. `b_fb` acts as the tap-spacing "swing", as in the native version.
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
mtchan   = (mtraw ~ *(0.5)) : (!, _); // feed tap1*0.5 back, output total only
multitap = mtchan, mtchan;

// Run all four, select the active one per output channel.
selector(sl, sr, pl, pr, tl, tr, ml, mr) =
    ba.selectn(4, int(c_type), sl, pl, tl, ml),
    ba.selectn(4, int(c_type), sr, pr, tr, mr);

process = _, _ <: (standard, pingpong, tape, multitap) : selector;
