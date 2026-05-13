---
title: "Frequency Modulation"
slug: "frequency-modulation"
group: "synthesis"
order: 108
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Any source can be frequency modulated. Frequency modulation (FM) is a technique where the frequency of a carrier wave is varied by an audio signal. This creates complex timbres and can produce rich harmonics, from mellow timbres to harsh digital noise. A second modulator can be enabled with `fm2` for 3-operator FM. Instead of a discrete algorithm selector, `fmpivot` continuously rotates op2's output between op1 and the carrier, sweeping through cascade, branch, parallel, and their phase-inverted variants on a single circle.

<CommandEntry name="fm" type="number" min={0} default={0} mod>

The frequency modulation index. FM multiplies the gain of the modulator, thus controls the amount of FM applied.

<CodeEditor code={`/fm/2/note/60\n\n/fm/4/note/63\n\n/fm/2/note/67`} rows={6} />

<CodeEditor code={`/voice/0/fm/2/time/0\n\n/voice/0/fm/4/time/1`} rows={4} />

<CodeEditor code={`/fm/0^5:0.01:0.1:0.3:0.5/fmh/3/decay/1/gate/2`} rows={2} />

</CommandEntry>

<CommandEntry name="fmh" type="number" default={1} mod>

The harmonic ratio of the frequency modulation. fmh*freq defines the modulation frequency. As a rule of thumb, numbers close to simple ratios sound more harmonic.

<CodeEditor code={`/fm/2/fmh/2/`} rows={2} />

<CodeEditor code={`/fm/0.5/fmh/1.5/`} rows={2} />

<CodeEditor code={`/fm/0.25/fmh/3/`} rows={2} />

</CommandEntry>

<CommandEntry name="fmshape" type="string" default="sine">

FM modulator waveform shape. Options: `sine`, `tri`, `saw`, `square`, `sh` (sample-and-hold). Different shapes create different harmonic spectra.

<CodeEditor code={`/fm/2/fmshape/saw`} rows={2} />

</CommandEntry>

<CommandEntry name="fm2" type="number" min={0} default={0} mod>

Modulation index for the second FM operator. When `fm2` is greater than 0, a third operator is introduced. Its routing is controlled by `fmpivot`. The second operator shares the same waveform (`fmshape`) as the first.

<CodeEditor code={`/fm/3/fmh/1/fm2/1.5/fm2h/14/decay/1/gate/1.5/note/36`} rows={2} />

<CodeEditor code={`/fm/2/fmh/3.14/fm2/4/fm2h/0.7/decay/0.4/gate/0.8/note/72`} rows={2} />

</CommandEntry>

<CommandEntry name="fm2h" type="number" default={1} mod>

Harmonic ratio of the second FM operator. `fm2h` * carrier frequency defines the second modulator's frequency.

<CodeEditor code={`/fm/1/fmh/4/fm2/2/fm2h/0.5/decay/2/gate/2/note/60`} rows={2} />

<CodeEditor code={`/fm/0.5/fmh/1/fm2/3/fm2h/11.03/decay/0.3/gate/0.5/note/84`} rows={2} />

</CommandEntry>

<CommandEntry name="fmpivot" type="number" min={0} max={1} default={0} mod>

Continuous op2 routing pivot, wraps. Replaces the old algorithm selector with a single knob that traces a circle in the (op2→op1, op2→carrier) plane. Total op2 modulation magnitude stays constant; only the destination rotates.

Named points on the circle:

- `0.000` — cascade (op2 → op1 → carrier)
- `0.125` — branch (op2 → both, equal-power split)
- `0.250` — parallel (op2 → carrier, op1 → carrier independently)
- `0.500` — inverted cascade
- `0.750` — inverted parallel
- `1.000` — wraps to cascade

Everything between is reachable; modulating `fmpivot` at LFO rate creates continuous algorithm morphing.

<CodeEditor code={`/fm/5/fmh/1/fm2/2/fm2h/7/fmpivot/0/decay/0.6/gate/1/note/40`} rows={2} />

<CodeEditor code={`/fm/1.5/fmh/2/fm2/1/fm2h/5.19/fmpivot/0.25/decay/1.5/gate/2/note/67`} rows={2} />

<CodeEditor code={`/fm/2/fmh/0.5/fm2/6/fm2h/3/fmpivot/0.125/decay/0.2/gate/0.4/note/55`} rows={2} />

<CodeEditor code={`/fm/4/fmh/2/fm2/3/fm2h/4/fmpivot/0~1:0.3/decay/4/gate/4/note/48`} rows={2} />

</CommandEntry>

<CommandEntry name="fmfb" type="number" default={0} mod>

Self-feedback amount on the topmost FM operator. Feedback progressively adds harmonics, turning a sine into a sawtooth-like waveform at moderate values and into noise at high values. When only `fm` is active, feedback applies to operator 1. When `fm2` is active, feedback applies to operator 2.

<CodeEditor code={`/fm/2/fmh/1/fmfb/0.4/decay/1/gate/1.5/note/48`} rows={2} />

<CodeEditor code={`/fm/3/fmh/2/fm2/1/fm2h/7/fmfb/1.2/decay/0.3/gate/0.6/note/36`} rows={2} />

</CommandEntry>
