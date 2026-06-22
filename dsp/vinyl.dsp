// VinylSim / Cassette "character" insert: wow + flutter (pitch wobble), band-
// limiting, tape/vinyl hiss and gentle saturation — the lo-fi degrade box. One
// mono instance per channel (so the hiss is decorrelated L/R). e_type picks the
// voicing: 0 = vinyl303 (warmest), 1 = vinyl404 (brighter), 2 = cassette (mid-
// focused, more hiss).
//
// Wow (slow) + flutter (fast) modulate a fractional delay read offset — the same
// idiom as chorus/flanger — so the signal pitch-wobbles. no.noise is an integer
// LCG (links clean on wasm). Placed pre-VCA in the voice chain, so the hiss is
// enveloped by the note rather than a constant floor.
//
// doux params: a_vinyl = dry/wet [0,1] (0 = bypass), b_wow = wow+flutter depth
// [0,1], c_noise = hiss level [0,1], d_tone = tone tilt [-1,1], e_type = voicing.
// Slider names prefixed a/b/.. for stable param order.
import("stdfaust.lib");

a_vinyl = hslider("a_vinyl", 0, 0, 1, 0.001);
b_wow   = hslider("b_wow", 0.3, 0, 1, 0.001);
c_noise = hslider("c_noise", 0.2, 0, 1, 0.001);
d_tone  = hslider("d_tone", 0, -1, 1, 0.001);
e_type  = hslider("e_type", 0, 0, 2, 1);

MAXD = 4096; // wow delay line (~85 ms at 48 kHz), power of two

// Wow (~0.8 Hz) + flutter (~6.3 Hz) as a fractional-delay read offset in samples.
warble = (os.osc(0.8) * 0.7 + os.osc(6.3) * 0.3) * (b_wow * 40.0);
base   = 64.0;
del(x) = de.fdelay4(MAXD, max(2.0, min(MAXD - 3.0, base + warble)), x);

// Per-type band-limiting + tone tilt.
lpfCut = ba.selectn(3, int(e_type), 8000.0, 11000.0, 6500.0);
toned(x) = x : fi.highpass(1, 30.0) : fi.lowpass(2, lpfCut)
             : fi.highshelf(1, d_tone * 6.0, 4000.0);

// Hiss: high-passed noise bed, level scaled by type (cassette hisses most).
noiseLvl = c_noise * ba.selectn(3, int(e_type), 0.03, 0.025, 0.05);
hiss = no.noise : fi.highpass(1, 1500.0) : *(noiseLvl);

wet(x) = (del(x) : toned : aa.tanh1) + hiss;
process(x) = x * (1.0 - a_vinyl) + wet(x) * a_vinyl;
