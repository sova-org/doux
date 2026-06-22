// Haas placement delay: a single fractional delay (interpolated), applied by the
// wrapper to one channel so the stereo image shifts toward the other. Mono
// 1-in/1-out. doux param: a_ms = delay in milliseconds.
import("stdfaust.lib");

a_ms = hslider("a_ms", 0, 0, 50, 0.001);

MAXSAMP = 8192; // >= 50 ms at 96 kHz, power of two
dels = max(2.0, min(MAXSAMP - 3.0, a_ms * ma.SR / 1000.0));

process = de.fdelay4(MAXSAMP, dels);
