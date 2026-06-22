// Feedback: stereo re-injection delay with one-pole damping and cross-channel
// blend. Replaces effects::feedback::Feedback. The orbit's send level is passed
// in as g_fb and doubles as the re-injection coefficient (matching the native
// call). Output is wet only.
//
// The base delay time is a per-sample SIGNAL INPUT (input 0, ms), not a slider:
// an orbit ModChain can sweep it at audio rate, so ds derives per sample and the
// fractional delay glides continuously (no click). This replaces the old
// built-in LFO (rate/depth/shape) — sweeping `fbtime` with a ModChain now does
// the same job at audio rate. The gain-like params are sliders, one-pole
// smoothed (ba.tau2pole) to de-zipper block-rate writes.
//
// doux params: time = ms (input 0), b_damp [0,1], c_cross (0 self .. 1 ping-pong),
// g_fb = re-injection amount (= orbit send level).
import("stdfaust.lib");

b_damp  = hslider("b_damp", 0, 0, 1, 0.001) : si.smooth(ba.tau2pole(0.005));
c_cross = hslider("c_cross", 0, 0, 1, 0.001) : si.smooth(ba.tau2pole(0.005));
g_fb    = hslider("g_fb", 0, 0, 0.99, 0.001) : si.smooth(ba.tau2pole(0.005));

MAXSAMP = 131072; // >= 1 s at 96 kHz (native MAX_DELAY_SECS), power of two

cross = max(0.0, min(1.0, c_cross));
fb    = max(0.0, min(0.99, g_fb));

damp(x) = (1.0 - b_damp) * x : fi.pole(b_damp);

// Stereo feedback: yl/yr are the (pre-damp) delayed wet outputs; the cross
// matrix mixes the damped outputs into each channel's feedback; `~` supplies
// the loop's one-sample delay. `time` (input 0, ms) feeds the per-sample delay.
process(time, l, r) = (l, r : fwd ~ back)
with {
    ds = max(1.0, min(MAXSAMP - 1.0, time * ma.SR / 1000.0));
    fwd(fbl, fbr, l, r) = yl, yr
    with {
        yl = de.fdelay(MAXSAMP, ds, l + fbl);
        yr = de.fdelay(MAXSAMP, ds, r + fbr);
    };
    back(yl, yr) = fbl, fbr
    with {
        dyl = damp(yl);
        dyr = damp(yr);
        fbl = (dyl * (1.0 - cross) + dyr * cross) * fb;
        fbr = (dyr * (1.0 - cross) + dyl * cross) * fb;
    };
};
