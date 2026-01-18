---
title: "Oscillator"
slug: "oscillator"
group: "synthesis"
order: 104
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

These parameters are dedicated to alter the nominal behavior of each oscillator. Some parameters are specific to certain oscillators, most others can be used with all oscillators.

<CommandEntry name="pw" type="number" min={0} max={1} default={0.5}>

The pulse width (between 0 and 1) of the pulse oscillator. The default is 0.5 (square wave). Only has an effect when used with <code>/sound/pulse</code> or <code>/sound/pulze</code>.

<CodeEditor code={`/sound/pulse/pw/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="spread" type="number" min={0} max={100} default={0}>

Stereo unison. Adds 6 detuned voices (7 total) with stereo panning. Works with sine, tri, saw, zaw, pulse, pulze.

<CodeEditor code={`/sound/saw/spread/30`} rows={2} />

</CommandEntry>

Inspired by the M8 Tracker's WavSynth, these parameters transform the oscillator phase to create new timbres from basic waveforms. They work with all basic oscillators (sine, tri, saw, zaw, pulse, pulze).

<CommandEntry name="size" type="number" min={0} max={256} default={0}>

Phase quantization steps. Creates stair-step waveforms similar to 8-bit sound chips. Set to 0 to disable, or 2-256 for increasing resolution. Lower values produce more lo-fi, chiptune-like sounds.

<CodeEditor code={`/sound/sine/size/8`} rows={2} />

</CommandEntry>

<CommandEntry name="mult" type="number" min={0.25} max={16} default={1}>

Phase multiplier that wraps the waveform multiple times per cycle. Creates hard-sync-like harmonic effects. A value of 2 doubles the frequency content, 4 quadruples it, etc.

<CodeEditor code={`/sound/saw/mult/4`} rows={2} />

</CommandEntry>

<CommandEntry name="warp" type="number" min={-1} max={1} default={0}>

Phase asymmetry using a power curve. Positive values compress the early phase and expand the late phase. Negative values do the opposite. Creates timbral variations without changing pitch.

<CodeEditor code={`/sound/tri/warp/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="mirror" type="number" min={0} max={1} default={0}>

Reflects the phase at the specified position. At 0.5, creates symmetric waveforms (a saw becomes triangle-like). Values closer to 0 or 1 create increasingly asymmetric reflections.

<CodeEditor code={`/sound/saw/mirror/.5`} rows={2} />

</CommandEntry>
