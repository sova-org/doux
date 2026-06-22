// Julian Parker's lush, modulated ambient reverb (`re.jpverb`). Stereo 2-in/2-out.
// The orbit's 0..1 ReverbParams are remapped here to jpverb's native ranges. The
// mid band gain is fixed at 1.0 (the orbit exposes only low/high). jpverb is wet
// by nature; the orbit adds its output back onto the dry bus.
// Slider names prefixed a/b/.. for stable param order.
import("stdfaust.lib");
re = library("reverbs.lib");

a_decay    = hslider("a_decay", 0.55, 0, 1, 0.001);
b_damp     = hslider("b_damp", 0.7, 0, 1, 0.001);
c_size     = hslider("c_size", 0.75, 0, 1, 0.001);
d_diff     = hslider("d_diff", 0.6, 0, 1, 0.001);
e_moddepth = hslider("e_moddepth", 0.3, 0, 1, 0.001);
f_modfreq  = hslider("f_modfreq", 0.65, 0, 1, 0.001);
g_low      = hslider("g_low", 0.4, 0, 1, 0.001);
h_high     = hslider("h_high", 0.5, 0, 1, 0.001);
i_lowcut   = hslider("i_lowcut", 0.5, 0, 1, 0.001);
j_highcut  = hslider("j_highcut", 0.7, 0, 1, 0.001);

t60     = 0.1 * (200.0 ^ a_decay);        // 0.1 .. 20 s
size    = 0.5 + c_size * 4.5;             // 0.5 .. 5
modfreq = f_modfreq * 10.0;              // 0 .. 10 Hz
lowco   = 100.0 + i_lowcut * 5900.0;     // 100 .. 6000 Hz
highco  = 1000.0 + j_highcut * 9000.0;   // 1000 .. 10000 Hz

process = re.jpverb(t60, b_damp, size, d_diff, e_moddepth, modfreq,
    g_low, 1.0, h_high, lowco, highco);
