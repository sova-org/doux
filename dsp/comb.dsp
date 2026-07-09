// Comb: feedback comb resonator with one-pole damping (Karplus-Strong style),
// mono — one instance per channel. Resonant peaks at the fundamental and its
// harmonics. Output is wet only; the orbit scales the send by the comb level
// and sums the wet back onto the bus.
//
// The fundamental frequency is a per-sample SIGNAL INPUT (input 0), not a
// slider: an orbit ModChain can sweep it at audio rate, so dl = SR/freq is
// derived per sample and the fractional delay glides continuously instead of
// jumping at block boundaries (no pitch-zipper). The gain-like params stay
// sliders, one-pole smoothed (ba.tau2pole) to de-zipper block-rate writes.
//
// doux params: freq = fundamental Hz (input 0), b_fb = feedback [-0.99,0.99],
// c_damp = HF damping [0,1]. Slider names prefixed b/c.. for stable param order.
import("stdfaust.lib");

b_fb   = hslider("b_fb", 0.9, -0.99, 0.99, 0.001) : si.smooth(ba.tau2pole(0.005));
c_damp = hslider("c_damp", 0.1, 0, 1, 0.001) : si.smooth(ba.tau2pole(0.005));

MAXD = 8192; // >= 50 ms at 96 kHz (native MAX_DELAY_MS), power of two

fb = max(-0.99, min(0.99, b_fb));

// One-pole damping in the feedback path: y = (1-dampc)*x + dampc*y'.
// c_damp = 0 reduces to identity, matching the native `if damp > 0` branch.
// Clamp < 1: fi.pole coeff >= 1 puts the pole on/outside the unit circle -> diverges.
dampc = max(0.0, min(0.99, c_damp));
damp(x) = (1.0 - dampc) * x : fi.pole(dampc);

// wet[n] = fdelay(x[n] + fb*damp(wet[n-1])); the `~` supplies the loop's
// one-sample delay (read-before-write of the hand-written version) and leaves
// the audio input. Linear fdelay matches the native DelayLine::read interp.
process(freq, x) = x : ((+ : de.fdelay(MAXD, dl)) ~ (damp : *(fb)))
with {
    dl = max(1.0, min(MAXD - 1.0, ma.SR / freq));
};
