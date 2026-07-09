// Phaser: Julius O. Smith III's `pf.phaser2` (allpass-chain phaser with
// feedback). Mono per channel — the wrapper runs one instance per channel and
// seeds `e_phase` (LFO phase 0..1) differently per channel for stereo width.
//
// doux params map onto phaser2_mono as: a_speed=LFO Hz, b_fb=feedback
// resonance, c_sweep=notch sweep range in cents above center, d_center=base
// notch frequency in Hz. depth is fixed at 1 (classic phaser: equal dry/wet).
// Slider names are prefixed a/b/.. so Faust's alphabetical param order is stable.
import("stdfaust.lib");
pf = library("phaflangers.lib");

a_speed  = hslider("a_speed", 0, 0, 100, 0.001);
b_fb     = hslider("b_fb", 0, 0, 0.95, 0.001);
c_sweep  = hslider("c_sweep", 1200, 0, 20000, 0.001);
d_center = hslider("d_center", 800, 0, 20000, 0.001);
e_phase  = hslider("e_phase", 0, 0, 1, 0.001);

NOTCHES = 4;
WIDTH = 1000.0;
FRATIO = 1.5;
DEPTH = 1.0;
INVERT = 0.0;

frqmin = max(20.0, min(d_center, 0.45 * ma.SR));
frqmax = min(0.45 * ma.SR, frqmin * (2.0 ^ (c_sweep / 1200.0)));
fbc = max(0.0, min(0.95, b_fb)); // clamp feedback: >=1 self-oscillates to NaN

process =
    pf.phaser2_mono(NOTCHES, e_phase, WIDTH, frqmin, FRATIO, frqmax, a_speed, DEPTH, fbc, INVERT);
