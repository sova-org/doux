// Feedback: stereo re-injection delay with one-pole damping and cross-channel
// blend. Replaces effects::feedback::Feedback. The orbit's send level is passed
// in as g_fb and doubles as the re-injection coefficient (matching the native
// call). Output is wet only.
//
// The delay is a per-sample SIGNAL INPUT (input 0), not a slider, so an orbit
// ModChain can sweep it at audio rate: ds derives per sample and the fractional
// delay glides continuously (no click). This replaces the old built-in LFO.
//
// Input 0 is the delay FREQUENCY (Hz = 1000/ms), reciprocated in the Rust
// wrapper, so the line length is `ds = ma.SR / dfreq` — comb.dsp's form, the
// ONLY one Faust can statically size. Deriving `ds = time * ma.SR / 1000` (or
// `ma.SR / (1000/time)`) from a signal input defeats Faust's interval analysis
// and silently shrinks the delay line to a 256-sample default — reads above
// ~5 ms then alias. The gain-like params are sliders, one-pole smoothed
// (ba.tau2pole) to de-zipper block-rate writes.
//
// doux params: dfreq = 1000/ms (input 0), b_damp [0,1], c_cross (0 self .. 1
// ping-pong), g_fb = re-injection amount (= orbit send level).
import("stdfaust.lib");

b_damp  = hslider("b_damp", 0, 0, 1, 0.001) : si.smooth(ba.tau2pole(0.005));
c_cross = hslider("c_cross", 0, 0, 1, 0.001) : si.smooth(ba.tau2pole(0.005));
g_fb    = hslider("g_fb", 0, 0, 0.99, 0.001) : si.smooth(ba.tau2pole(0.005));

MAXSAMP = 65536; // ds <= MAXSAMP-1 = 0.68 s at 96 kHz (covers fbtime max 680 ms);
                 // the ma.SR/dfreq line sizes to 2*MAXSAMP = 131072 samples.

cross = max(0.0, min(1.0, c_cross));
fb    = max(0.0, min(0.99, g_fb));

// Clamp < 1: fi.pole coeff >= 1 puts the pole on/outside the unit circle -> diverges.
dampc = max(0.0, min(0.99, b_damp));
damp(x) = (1.0 - dampc) * x : fi.pole(dampc);

// Stereo feedback: yl/yr are the (pre-damp) delayed wet outputs; the cross
// matrix mixes the damped outputs into each channel's feedback; `~` supplies
// the loop's one-sample delay. `dfreq` (input 0, 1000/ms) feeds the per-sample
// delay as ma.SR/dfreq (see header on why the reciprocal is formed in Rust).
process(dfreq, l, r) = (l, r : fwd ~ back)
with {
    ds = max(1.0, min(MAXSAMP - 1.0, ma.SR / dfreq));
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
