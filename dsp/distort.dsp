// Soft saturation with a selectable shaper. doux params `distort` (drive),
// `distortvol` (postgain), `distortmode` (curve 0-5), `distortasym` (bias).
//   mode 0 = original (1+k)x/(1+k|x|) soft-sat with baked bias for even harmonics
//            + drive-loudness comp (the bit-exact default).
//   modes 1-5 = drive into a normalized +/-1 ADAA antialiased shaper (aanl.lib)
//            then the same drive-loudness comp: 1=tanh, 2=arctan, 3=hardclip,
//            4=parabolic, 5=sinarctan. ADAA suppresses the aliasing a naive
//            shaper would add. (cubic1 was evaluated and dropped: in f32 at high
//            drive its corner overshoot spikes on saw resets — clicky.)
// `distortasym` adds a pre-shaper DC bias for asymmetric / even-harmonic drive;
// the induced DC is removed by the voice's downstream DcBlock stage. At 0 the
// existing modes stay bit-exact.
// Slider names prefixed a/b/c/d so Faust's alphabetical param order is stable.
import("stdfaust.lib");
a_distort     = hslider("a_distort", 0, 0, 100, 0.001);
b_distortvol  = hslider("b_distortvol", 1, 0, 2, 0.001);
c_distortmode = hslider("c_distortmode", 0, 0, 5, 1);
d_asym        = hslider("d_asym", 0, -1, 1, 0.001);

bias  = 0.05;
compP = 0.12;
k = max(a_distort, 0.0);
drv = 1.0 + k;
comp = drv ^ (0.0 - compP); // drive-loudness compensation, shared by all modes

// mode 0: original soft-sat with bias injection (bit-exact default).
sat0(x) = drv * x / (1.0 + k * abs(x));
fb0 = sat0(bias);
curve0(x) = (sat0(x + bias) - fb0) * comp;

// modes 1-5: drive into a normalized +/-1 ADAA shaper, then the shared comp.
curve1(x) = aa.tanh1(drv * x) * comp;
curve2(x) = aa.arctan(drv * x) * (2.0 / ma.PI) * comp;
curve3(x) = aa.hardclip(drv * x) * comp;
curve4(x) = aa.parabolic(drv * x) * comp;
curve5(x) = aa.sinarctan(drv * x) * comp;

// distortasym shifts the shaper operating point; the downstream DcBlock removes
// the induced DC. At d_asym = 0 every curve is unchanged.
abias(x) = x + d_asym;
shaped(x) = ba.selectn(6, int(c_distortmode),
    curve0(abias(x)), curve1(abias(x)), curve2(abias(x)),
    curve3(abias(x)), curve4(abias(x)), curve5(abias(x)));

process(x) = shaped(x) * b_distortvol;
