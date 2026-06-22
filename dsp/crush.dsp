// Bit-depth reduction (bitcrusher). doux param `crush` = target bit depth.
// y = round(x * q) / q, q = 2^(bits-1), bits = max(crush, 1).
import("stdfaust.lib");
crush = hslider("crush", 0, 0, 16, 0.001);
bits = max(crush, 1.0);
q = 2.0 ^ (bits - 1.0);
roundn(x) = floor(x + 0.5);
process = _ : *(q) : roundn : /(q);
