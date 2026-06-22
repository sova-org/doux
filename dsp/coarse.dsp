// Sample-rate reduction (decimation). doux param `coarse` = hold factor.
import("stdfaust.lib");
factor = hslider("coarse", 0, 0, 128, 1);
stride = max(1, int(factor));
process = ba.sAndH((ba.time % stride) == 0, _);
