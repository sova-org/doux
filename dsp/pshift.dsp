// Granular (delay-line) pitch shifter — `ef.transpose`: two delay taps swept at
// the transposition rate and crossfaded at the wrap. Transposes by SEMITONES,
// preserving harmonic ratios (unlike the inharmonic fshift): octaves, fifths,
// detune thickening, and dive-bombs/risers when a_shift is modulated. The
// classic granular warble grows past ~±7 semitones — a feature for live coding.
//
// b_window sets the grain length in ms: short = grainy/robotic (the warble is
// faster and more present), long = smoother but more latency and echo. The
// crossfade tracks the window (quarter-window) to hide the grain splice. Window
// in ms uses ma.SR, so this DSP is sample-rate dependent and re-inits per rate.
// Mono — the wrapper runs one instance per channel. Sliders prefixed a_/b_ so
// Faust's alphabetical param order is stable (a_shift = 0, b_window = 1).
import("stdfaust.lib");

a_shift  = hslider("a_shift", 0, -24, 24, 0.001);
b_window = hslider("b_window", 40, 5, 200, 0.01);

win = b_window * ma.SR / 1000.0; // grain window in samples
xfd = win / 4.0;                 // crossfade in samples (quarter window)

process = ef.transpose(win, xfd, a_shift);
