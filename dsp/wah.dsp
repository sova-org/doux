// Auto-wah: a resonant bandpass whose cutoff tracks the input envelope (an
// envelope-follower / "touch" wah). doux's ModChain modulators cannot follow the
// live audio signal, so the follower has to live inside the DSP. One mono
// instance per channel.
//
// doux params: a_wah = dry/wet [0,1] (0 = bypass), b_peak = resonance [0,1],
// c_sens = envelope sensitivity [0,1], d_manual = base cutoff Hz. Slider names
// prefixed a/b/.. so Faust's alphabetical param order is stable.
import("stdfaust.lib");

a_wah    = hslider("a_wah", 0, 0, 1, 0.001);
b_peak   = hslider("b_peak", 0.5, 0, 1, 0.001);
c_sens   = hslider("c_sens", 0.5, 0, 1, 0.001);
d_manual = hslider("d_manual", 400, 100, 4000, 0.1);

Q   = 2.0 + b_peak * 18.0;
env = an.amp_follower(0.01); // 10 ms release envelope of |x|
// Cutoff rides up from the manual base with the envelope, clamped below Nyquist.
fc(x)  = min(0.45 * ma.SR, d_manual + env(x) * c_sens * 4000.0);
wet(x) = fi.svf.bp(fc(x), Q, x);

process(x) = x * (1.0 - a_wah) + wet(x) * a_wah;
