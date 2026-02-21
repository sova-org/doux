---
title: "Complex"
slug: "plaits"
group: "sources"
order: 1
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Complex oscillator engines based on Mutable Instruments Plaits. All engines share three parameters (0 to 1):

- **harmonics** — harmonic content, structure, detuning, etc.
- **timbre** — brightness, tonal color, etc.
- **morph** — smooth transitions between variations, etc.

Each engine interprets these differently.

<CommandEntry name="modal" type="source">

Modal resonator (physical modeling). Simulates struck/plucked resonant bodies. harmonics: structure, timbre: brightness, morph: damping/decay.

<CodeEditor code={`/sound/modal/note/48`} rows={2} />

<CodeEditor code={`/sound/modal/note/36/harmonics/0.8/morph/0.2`} rows={2} />

<CodeEditor code={`/sound/modal/note/60/timbre/0.7/morph/0.5/verb/0.3`} rows={2} />

</CommandEntry>

<CommandEntry name="va" type="source">

Virtual analog. Classic waveforms with sync and crossfading. harmonics: detuning, timbre: variable square, morph: variable saw.

<CodeEditor code={`/sound/va/note/36`} rows={2} />

<CodeEditor code={`/sound/va/note/48/harmonics/0.3/morph/0.8`} rows={2} />

<CodeEditor code={`/sound/va/note/36/timbre/0.2/lpf/1000`} rows={2} />

</CommandEntry>

<CommandEntry name="ws" type="source">

Waveshaping oscillator. Asymmetric triangle through waveshaper and wavefolder. harmonics: waveshaper shape, timbre: fold amount, morph: waveform asymmetry.

<CodeEditor code={`/sound/ws/note/36`} rows={2} />

<CodeEditor code={`/sound/ws/note/48/timbre/0.7/harmonics/0.5`} rows={2} />

<CodeEditor code={`/sound/ws/note/36/morph/0.8/timbre/0.9`} rows={2} />

</CommandEntry>

<CommandEntry name="fm2" type="source">

Two-operator FM synthesis. harmonics: frequency ratio, timbre: modulation index, morph: feedback.

<CodeEditor code={`/sound/fm2/note/48`} rows={2} />

<CodeEditor code={`/sound/fm2/note/60/timbre/0.5/harmonics/0.3`} rows={2} />

<CodeEditor code={`/sound/fm2/note/36/morph/0.7/timbre/0.8`} rows={2} />

</CommandEntry>

<CommandEntry name="grain" type="source">

Granular formant oscillator. Simulates formants through windowed sines. harmonics: formant ratio, timbre: formant frequency, morph: formant width.

<CodeEditor code={`/sound/grain/note/48`} rows={2} />

<CodeEditor code={`/sound/grain/note/36/timbre/0.6/harmonics/0.4`} rows={2} />

<CodeEditor code={`/sound/grain/note/60/morph/0.3/timbre/0.8`} rows={2} />

</CommandEntry>

<CommandEntry name="additive" type="source">

Harmonic oscillator. Additive mixture of sine harmonics. harmonics: number of bumps, timbre: prominent harmonic index, morph: bump shape.

<CodeEditor code={`/sound/additive/note/48`} rows={2} />

<CodeEditor code={`/sound/additive/note/36/timbre/0.5/harmonics/0.3`} rows={2} />

<CodeEditor code={`/sound/additive/note/60/morph/0.8/timbre/0.7`} rows={2} />

</CommandEntry>

<CommandEntry name="wavetable" type="source">

Wavetable oscillator. Four banks of 8x8 waveforms. harmonics: bank selection, timbre: row index, morph: column index.

<CodeEditor code={`/sound/wavetable/note/48`} rows={2} />

<CodeEditor code={`/sound/wavetable/note/36/timbre/0.5/morph/0.5`} rows={2} />

<CodeEditor code={`/sound/wavetable/note/60/harmonics/0.3/timbre/0.7/morph/0.2`} rows={2} />

</CommandEntry>

<CommandEntry name="chord" type="source">

Four-note chord engine. Virtual analog or wavetable chords. harmonics: chord type, timbre: inversion/transposition, morph: waveform.

<CodeEditor code={`/sound/chord/note/48`} rows={2} />

<CodeEditor code={`/sound/chord/note/36/harmonics/0.5`} rows={2} />

<CodeEditor code={`/sound/chord/note/48/harmonics/0.3/morph/0.7/verb/0.2`} rows={2} />

</CommandEntry>

<CommandEntry name="swarm" type="source">

Granular cloud of 8 enveloped sawtooth oscillators. harmonics: pitch randomization, timbre: grain density, morph: grain duration/overlap.

<CodeEditor code={`/sound/swarm/note/36`} rows={2} />

<CodeEditor code={`/sound/swarm/note/48/harmonics/0.5/morph/0.3`} rows={2} />

<CodeEditor code={`/sound/swarm/note/36/timbre/0.4/harmonics/0.7`} rows={2} />

</CommandEntry>

<CommandEntry name="pnoise" type="source">

Filtered noise. Clocked noise through multimode filter. harmonics: filter type (LP/BP/HP), timbre: clock frequency, morph: filter resonance.

<CodeEditor code={`/sound/pnoise/note/48`} rows={2} />

<CodeEditor code={`/sound/pnoise/note/36/harmonics/0.5/morph/0.7`} rows={2} />

<CodeEditor code={`/sound/pnoise/note/60/timbre/0.8/morph/0.9`} rows={2} />

</CommandEntry>
