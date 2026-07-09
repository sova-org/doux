// Flanger: LFO-swept short delay (0.5–10 ms) with feedback, summed 50/50 with
// the dry signal. One mono instance per channel; the wrapper seeds d_phase per
// channel so the sweep is out of phase between channels for stereo width.
//
// e_thru selects the mode: 0 = classic (dry undelayed, the bit-exact default),
// 1 = through-zero (dry delayed by the sweep centre so the wet sweep crosses it
// and the notch passes through DC — the deep "jet" flange).
//
// doux params: a_rate=LFO Hz, b_depth=sweep depth [0,1] (squared into a 0.5–10ms
// span), c_fb=feedback, e_thru=mode. Slider names prefixed a/b/.. for stable order.
import("stdfaust.lib");

a_rate  = hslider("a_rate", 0, 0, 100, 0.001);
b_depth = hslider("b_depth", 0.7, 0, 1, 0.001);
c_fb    = hslider("c_fb", 0.35, 0, 0.95, 0.001);
d_phase = hslider("d_phase", 0, 0, 1, 0.001);
e_thru  = hslider("e_thru", 0, 0, 1, 1);

MIN_MS = 0.5;
MAX_MS = 10.0;
MAXSAMP = 1024; // >= 10 ms at 96 kHz, power of two

span = (b_depth * b_depth) * (MAX_MS - MIN_MS);
lfo = os.oscp(a_rate, 2.0 * ma.PI * d_phase) * 0.5 + 0.5; // unipolar 0..1
dels = max(2.0, min(MAXSAMP - 3.0, (MIN_MS + span * lfo) * ma.SR / 1000.0));
fb = max(0.0, min(0.95, c_fb)); // clamp both ends: large-negative fb runs the comb away

// Feedback comb: wet[n] = fdelay(x[n] + fb*wet[n-1]); the `~` supplies the loop's
// one-sample delay (matching the read-before-write of the hand-written version).
wet = (+ : de.fdelay4(MAXSAMP, dels)) ~ *(fb);

// Through-zero: delay the dry path by the sweep centre so the wet sweep crosses
// it (relative delay passes through 0). select2 keeps e_thru = 0 bit-exact —
// the dry passes straight through, undelayed.
centreD = max(2.0, min(MAXSAMP - 3.0, (MIN_MS + span * 0.5) * ma.SR / 1000.0));
dry(x) = select2(int(e_thru), x, de.fdelay4(MAXSAMP, centreD, x));

process(x) = dry(x) * 0.5 + (x : wet) * 0.5;
