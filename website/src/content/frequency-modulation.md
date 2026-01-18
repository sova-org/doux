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

Any source can be frequency modulated. Frequency modulation (FM) is a technique where the frequency of a carrier wave is varied by an audio signal. This creates complex timbres and can produce rich harmonics, from mellow timbres to harsh digital noise.

<CommandEntry name="fm" type="number" min={0} default={0}>

The frequency modulation index. FM multiplies the gain of the modulator, thus controls the amount of FM applied.

<CodeEditor code={`/fm/2/note/60\n\n/fm/4/note/63\n\n/fm/2/note/67`} rows={6} />

<CodeEditor code={`/voice/0/fm/2/time/0\n\n/voice/0/fm/4/time/1`} rows={4} />

</CommandEntry>

<CommandEntry name="fmh" type="number" default={1}>

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
