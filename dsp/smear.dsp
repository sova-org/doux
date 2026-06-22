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
t = min(ma.PI * b_freq / ma.SR, ma.PI * 0.4999);
ap_a = (tan(t) - 1.0) / (tan(t) + 1.0);
ap = fi.tf1(ap_a, 1.0, ap_a); // (a + z^-1) / (1 + a z^-1)
chain = seq(i, N, ap);

wet = (+ : chain) ~ *(c_fb);
process(x) = x * (1.0 - a_mix) + (x : wet) * a_mix;
