// Single-knob tilt EQ (high shelf). doux param `tilt` in [-1,1] -> +/-6 dB.
import("stdfaust.lib");
tilt = hslider("tilt", 0, -1, 1, 0.001);
process = fi.highshelf(1, tilt * 6.0, 800.0);
