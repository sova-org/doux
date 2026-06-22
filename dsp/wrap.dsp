// Phase-wrap distortion. doux param `wrap` in [0,10]. The DC from the bias is
// removed by the existing Rust DcBlock stage downstream.
import("stdfaust.lib");
wraps = hslider("wrap", 0, 0, 10, 0.001);
bias = 0.04;
k = 1.0 + wraps;
wrap2(u) = u - 2.0 * floor(u * 0.5);
process = +(bias) : *(k) : +(1.0) : wrap2 : +(-1.0);
