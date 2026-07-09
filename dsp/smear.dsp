// Smear: a cascade of 12 first-order allpass sections (shared break frequency)
// with a feedback path — a phase-diffusion / transient-smear effect. Mono per
// channel. doux params: a_mix=wet/dry, b_freq=allpass break freq in Hz,
// c_fb=feedback for resonance. Slider names prefixed a/b/.. for stable order.
import("stdfaust.lib");

a_mix  = hslider("a_mix", 0, 0, 1, 0.001);
b_freq = hslider("b_freq", 1000, 20, 20000, 0.001);
c_fb   = hslider("c_fb", 0, 0, 0.95, 0.001);

N = 12;
// First-order allpass coefficient a = (tan(t)-1)/(tan(t)+1), t = pi*f/SR.
// Clamp f into (0, Nyquist): tan(t)+1 hits 0 at t=-pi/4 (negative f) -> divide by
// zero -> NaN. Clamp feedback < 1 so the unity-gain allpass loop can't diverge.
bf = max(1.0, min(b_freq, ma.SR * 0.4999));
t = ma.PI * bf / ma.SR;
ap_a = (tan(t) - 1.0) / (tan(t) + 1.0);
ap = fi.tf1(ap_a, 1.0, ap_a); // (a + z^-1) / (1 + a z^-1)
chain = seq(i, N, ap);

fb = max(0.0, min(0.95, c_fb));
wet = (+ : chain) ~ *(fb);
process(x) = x * (1.0 - a_mix) + (x : wet) * a_mix;
