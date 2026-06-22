// Stereo chorus with a selectable voicing. Stereo in/out: the input is summed to
// mono, written to delay lines, and read at LFO-modulated fractional taps (4th-
// order Lagrange) per side; output is an equal-power 50/50 dry/wet mix.
//   mode 0 = classic 3-voice (the bit-exact default): three taps phase-offset by
//            1/3 cycle, L/R modulating in opposite directions.
//   mode 1 = ensemble: four taps, 1/4-cycle offsets, wider detune (Juno-ish).
//   mode 2 = dimension: two taps in quadrature, deeper, no centre voice.
//
// doux params: a_rate=LFO Hz, b_depth=modulation intensity [0,1], c_delay=base
// delay in ms, d_type=voicing 0-2. Slider names prefixed a/b/.. so Faust's
// alphabetical param order is stable.
import("stdfaust.lib");

a_rate  = hslider("a_rate", 0, 0, 100, 0.001);
b_depth = hslider("b_depth", 0.35, 0, 1, 0.001);
c_delay = hslider("c_delay", 25, 0, 100, 0.001);
d_type  = hslider("d_type", 0, 0, 2, 1);

// Max delay in samples: >= 50 ms at MAX_SAMPLE_RATE (96 kHz), power of two.
MAXD = 8192;
MIX = 0.70710677; // 1/sqrt(2), equal-power 50/50 dry/wet

// Base delay (samples), smoothed over ~20 ms to suppress zipper on c_delay jumps.
base = c_delay * ma.SR / 1000.0 : si.smooth(ba.tau2pole(0.02));
modr = base * 0.8 * b_depth;
cl(d) = max(2.0, min(MAXD - 3.0, d));
lfo(v) = os.oscp(a_rate, 2.0 * ma.PI * float(v) / 3.0);

// mode 0 — classic 3-voice (verbatim; the default branch is bit-exact).
classicL(m) = (de.fdelay4(MAXD, cl(base + modr * lfo(0)), m)
             + de.fdelay4(MAXD, cl(base + modr * lfo(1)), m)
             + de.fdelay4(MAXD, cl(base + modr * lfo(2)), m)) / 3.0;
classicR(m) = (de.fdelay4(MAXD, cl(base - modr * lfo(0)), m)
             + de.fdelay4(MAXD, cl(base - modr * lfo(1)), m)
             + de.fdelay4(MAXD, cl(base - modr * lfo(2)), m)) / 3.0;

// mode 1 — ensemble: 4 taps, 1/4-cycle offsets, wider detune.
emodr  = modr * 1.5;
elfo(v) = os.oscp(a_rate, 2.0 * ma.PI * float(v) / 4.0);
ensembleL(m) = (de.fdelay4(MAXD, cl(base + emodr * elfo(0)), m)
              + de.fdelay4(MAXD, cl(base + emodr * elfo(1)), m)
              + de.fdelay4(MAXD, cl(base + emodr * elfo(2)), m)
              + de.fdelay4(MAXD, cl(base + emodr * elfo(3)), m)) / 4.0;
ensembleR(m) = (de.fdelay4(MAXD, cl(base - emodr * elfo(0)), m)
              + de.fdelay4(MAXD, cl(base - emodr * elfo(1)), m)
              + de.fdelay4(MAXD, cl(base - emodr * elfo(2)), m)
              + de.fdelay4(MAXD, cl(base - emodr * elfo(3)), m)) / 4.0;

// mode 2 — dimension: 2 quadrature taps, deeper, no centre.
dmodr  = modr * 2.0;
dlfo(v) = os.oscp(a_rate, 2.0 * ma.PI * float(v) / 2.0 + ma.PI / 2.0);
dimL(m) = (de.fdelay4(MAXD, cl(base + dmodr * dlfo(0)), m)
         + de.fdelay4(MAXD, cl(base + dmodr * dlfo(1)), m)) / 2.0;
dimR(m) = (de.fdelay4(MAXD, cl(base - dmodr * dlfo(0)), m)
         + de.fdelay4(MAXD, cl(base - dmodr * dlfo(1)), m)) / 2.0;

selL(m) = ba.selectn(3, int(d_type), classicL(m), ensembleL(m), dimL(m));
selR(m) = ba.selectn(3, int(d_type), classicR(m), ensembleR(m), dimR(m));

process(l, r) = l * MIX + selL(m) * MIX, r * MIX + selR(m) * MIX
with {
    m = (l + r) * 0.5;
};
