// Soft saturation with a selectable shaper. doux params `distort` (drive),
// `distortvol` (postgain), `distortmode` (curve 0-3).
//   mode 0 = original (1+k)x/(1+k|x|) soft-sat with baked bias for even harmonics
//            + drive-loudness comp (the bit-exact default).
//   modes 1-3 = drive into a normalized +/-1 ADAA antialiased shaper (aanl.lib)
//            then the same drive-loudness comp: 1=tanh, 2=arctan, 3=hardclip.
//            ADAA suppresses the aliasing a naive shaper would add. (cubic1 was
//            evaluated and dropped: in f32 at high drive its corner overshoot
//            spikes on saw resets — clicky, not a clean curve.)
// Slider names prefixed a/b/c so Faust's alphabetical param order is stable.
import("stdfaust.lib");
a_distort     = hslider("a_distort", 0, 0, 100, 0.001);
b_distortvol  = hslider("b_distortvol", 1, 0, 2, 0.001);
c_distortmode = hslider("c_distortmode", 0, 0, 3, 1);

bias  = 0.05;
compP = 0.12;
k = max(a_distort, 0.0);
drv = 1.0 + k;
comp = drv ^ (0.0 - compP); // drive-loudness compensation, shared by all modes

// mode 0: original soft-sat with bias injection (bit-exact default).
sat0(x) = drv * x / (1.0 + k * abs(x));
fb0 = sat0(bias);
curve0(x) = (sat0(x + bias) - fb0) * comp;

// modes 1-3: drive into a normalized +/-1 ADAA shaper, then the shared comp.
curve1(x) = aa.tanh1(drv * x) * comp;
curve2(x) = aa.arctan(drv * x) * (2.0 / ma.PI) * comp;
curve3(x) = aa.hardclip(drv * x) * comp;

shaped(x) = ba.selectn(4, int(c_distortmode),
    curve0(x), curve1(x), curve2(x), curve3(x));

process(x) = shaped(x) * b_distortvol;
