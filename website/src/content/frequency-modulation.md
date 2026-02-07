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

Any source can be frequency modulated. Frequency modulation (FM) is a technique where the frequency of a carrier wave is varied by an audio signal. This creates complex timbres and can produce rich harmonics, from mellow timbres to harsh digital noise. A second modulator can be enabled with `fm2` for 3-operator FM, with `fmalgo` selecting between cascade, parallel, and branch routing algorithms.

<CommandEntry name="fm" type="number" min={0} default={0} mod>

The frequency modulation index. FM multiplies the gain of the modulator, thus controls the amount of FM applied.

<CodeEditor code={`/fm/2/note/60\n\n/fm/4/note/63\n\n/fm/2/note/67`} rows={6} />

<CodeEditor code={`/voice/0/fm/2/time/0\n\n/voice/0/fm/4/time/1`} rows={4} />

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

<CommandEntry name="fmenv" type="number" default={0}>

Envelope amount of frequency envelope.

<CodeEditor code={`/fm/4/fmenv/4/fmd/0.25`} rows={2} />

</CommandEntry>

<CommandEntry name="fma" type="number" min={0} default={0} unit="s">

The duration (seconds) of the fm envelope's attack phase.

<CodeEditor code={`/fm/4/fma/0.25`} rows={2} />

</CommandEntry>

<CommandEntry name="fmd" type="number" min={0} default={0} unit="s">

The duration (seconds) of the fm envelope's decay phase.

<CodeEditor code={`/fm/4/fmd/0.25`} rows={2} />

</CommandEntry>

<CommandEntry name="fms" type="number" min={0} max={1} default={1}>

The sustain level of the fm envelope.

<CodeEditor code={`/fm/4/fmd/0.25/fms/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="fmr" type="number" min={0} default={0} unit="s">

The duration (seconds) of the fm envelope's release phase.

<CodeEditor code={`/fm/4/fmr/1/release/1/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="fm2" type="number" min={0} default={0} mod>

Modulation index for the second FM operator. When `fm2` is greater than 0, a third operator is introduced. Its routing depends on `fmalgo`. The second operator shares the same waveform (`fmshape`) and envelope (`fma`, `fmd`, `fms`, `fmr`) as the first.

<CodeEditor code={`/fm/3/fmh/1/fm2/1.5/fm2h/14/decay/1/dur/1.5/note/36`} rows={2} />

<CodeEditor code={`/fm/2/fmh/3.14/fm2/4/fm2h/0.7/decay/0.4/dur/0.8/note/72`} rows={2} />

</CommandEntry>

<CommandEntry name="fm2h" type="number" default={1} mod>

Harmonic ratio of the second FM operator. `fm2h` * carrier frequency defines the second modulator's frequency.

<CodeEditor code={`/fm/1/fmh/4/fm2/2/fm2h/0.5/decay/2/dur/2/note/60`} rows={2} />

<CodeEditor code={`/fm/0.5/fmh/1/fm2/3/fm2h/11.03/decay/0.3/dur/0.5/note/84`} rows={2} />

</CommandEntry>

<CommandEntry name="fmalgo" type="number" min={0} max={2} default={0}>

Selects the FM routing algorithm when `fm2` is active. `0` is cascade (fm2 modulates fm1, fm1 modulates carrier), `1` is parallel (fm1 and fm2 both modulate the carrier independently), `2` is branch (fm2 modulates both fm1 and the carrier).

<CodeEditor code={`/fm/5/fmh/1/fm2/2/fm2h/7/fmalgo/0/decay/0.6/dur/1/note/40`} rows={2} />

<CodeEditor code={`/fm/1.5/fmh/2/fm2/1/fm2h/5.19/fmalgo/1/decay/1.5/dur/2/note/67`} rows={2} />

<CodeEditor code={`/fm/2/fmh/0.5/fm2/6/fm2h/3/fmalgo/2/decay/0.2/dur/0.4/note/55`} rows={2} />

</CommandEntry>

<CommandEntry name="fmfb" type="number" default={0} mod>

Self-feedback amount on the topmost FM operator. Feedback progressively adds harmonics, turning a sine into a sawtooth-like waveform at moderate values and into noise at high values. When only `fm` is active, feedback applies to operator 1. When `fm2` is active, feedback applies to operator 2.

<CodeEditor code={`/fm/2/fmh/1/fmfb/0.4/decay/1/dur/1.5/note/48`} rows={2} />

<CodeEditor code={`/fm/3/fmh/2/fm2/1/fm2h/7/fmfb/1.2/decay/0.3/dur/0.6/note/36`} rows={2} />

</CommandEntry>
