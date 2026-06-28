// Single-sideband frequency shifter. Shifts every partial by a fixed number of
// Hz (NOT a transpose — harmonic ratios are broken), giving the classic
// inharmonic / metallic / "barber-pole" timbre: small shifts detune and phase,
// large shifts ring-modulate. Built from an analytic signal (Julius Smith's
// positive-pass Hilbert pair `fi.pospass`, whose real and imaginary outputs
// share the same lowpass so they stay phase-coherent) heterodyned by a
// quadrature oscillator (`os.quadosc`) at the shift frequency. The sign of
// a_shift selects the sideband: positive shifts up, negative shifts down.
//
// pospass halves amplitude (it discards negative frequencies), so the result is
// scaled by 2 for unity gain. Mono — the wrapper runs one instance per channel.
// Slider prefixed a_ so Faust's alphabetical param order is stable (index 0).
import("stdfaust.lib");

a_shift = hslider("a_shift", 0, -2000, 2000, 0.001);

// pospass Butterworth order and lower band edge (Hz). Higher order sharpens the
// analytic-signal accuracy near DC and Nyquist at more CPU; 10 / 20 Hz is a
// musical compromise.
ORDER = 10;
FC = 20.0;

process(x) = 2.0 * (re * c - im * s)
with {
    reim = x : fi.pospass(ORDER, FC);
    re = reim : _, !;
    im = reim : !, _;
    cs = os.quadosc(a_shift);
    c = cs : _, !;
    s = cs : !, _;
};
