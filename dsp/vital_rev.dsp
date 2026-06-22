// Vital's reverb, the real thing (`re.vital_rev`). Stereo 2-in/2-out. All inputs
// are 0..1 normalized (remapped internally by the library), so the orbit's
// normalized ReverbParams map directly. `mix` is fixed at 1.0 (fully wet): the
// orbit scales the input by the send level and adds the wet back onto the bus.
// Slider names prefixed a/b/.. for stable param order.
import("stdfaust.lib");
re = library("reverbs.lib");

a_prelow    = hslider("a_prelow", 0.2, 0, 1, 0.001);
b_prehigh   = hslider("b_prehigh", 0.9, 0, 1, 0.001);
c_lowcut    = hslider("c_lowcut", 0.5, 0, 1, 0.001);
d_highcut   = hslider("d_highcut", 0.7, 0, 1, 0.001);
e_lowgain   = hslider("e_lowgain", 0.4, 0, 1, 0.001);
f_highgain  = hslider("f_highgain", 0.5, 0, 1, 0.001);
g_chorus    = hslider("g_chorus", 0.3, 0, 1, 0.001);
h_chorusfreq = hslider("h_chorusfreq", 0.65, 0, 1, 0.001);
i_predelay  = hslider("i_predelay", 0, 0, 1, 0.001);
j_time      = hslider("j_time", 0.55, 0, 1, 0.001);
k_size      = hslider("k_size", 0.75, 0, 1, 0.001);

process = re.vital_rev(a_prelow, b_prehigh, c_lowcut, d_highcut, e_lowgain,
    f_highgain, g_chorus, h_chorusfreq, i_predelay, j_time, k_size, 1.0);
