// Reflective wavefolder with a selectable fold shape. doux params `fold` in
// [0,1] and `foldmode` (0=triangle, 1=sine, 2=wrap).
//   mode 0 = the original reflective triangle fold (the bit-exact default).
//   mode 1 = sine fold sin(pi/2 * v) (rounder, fewer high harmonics).
//   mode 2 = sawtooth wrap (harsher).
// d = 2^(fold*DEPTH) drive, g = 2^(-fold*DEPTH*COMP) makeup.
// Slider names prefixed a/b so Faust's alphabetical param order is stable.
import("stdfaust.lib");
a_fold     = hslider("a_fold", 0, 0, 1, 0.001);
b_foldmode = hslider("b_foldmode", 0, 0, 2, 1);
depth = 3.0;
comp = 0.5;
d = 2.0 ^ (a_fold * depth);
g = 2.0 ^ (0.0 - a_fold * depth * comp);
fracp(x) = x - floor(x);
tri(v)   = 1.0 - abs(4.0 * fracp((v + 1.0) / 4.0) - 2.0);
sine(v)  = sin(0.5 * ma.PI * v);
wrapf(v) = 2.0 * fracp((v + 1.0) / 2.0) - 1.0;
shape(v) = ba.selectn(3, int(b_foldmode), tri(v), sine(v), wrapf(v));
process = _ : *(d) : shape : *(g);
