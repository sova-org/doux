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

<CommandEntry name="pw" type="number" min={0} max={1} default={0.5} mod>

The pulse width (between 0 and 1) of the pulse oscillator. The default is 0.5 (square wave). Only has an effect when used with <code>/sound/pulse</code> or <code>/sound/pulze</code>.

<CodeEditor code={`/sound/pulse/pw/.1`} rows={2} />

<CodeEditor code={`/sound/pulse/pw/0.1~0.9:1/freq/100/decay/2/gate/3`} rows={2} />

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

<CommandEntry name="warp" type="number" min={-1} max={1} default={0}>

Phase asymmetry using a power curve. Positive values compress the early phase and expand the late phase. Negative values do the opposite. Creates timbral variations without changing pitch.

<CodeEditor code={`/sound/tri/warp/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="mirror" type="number" min={0} max={1} default={0}>

Reflects the phase at the specified position. At 0.5, creates symmetric waveforms (a saw becomes triangle-like). Values closer to 0 or 1 create increasingly asymmetric reflections.

<CodeEditor code={`/sound/saw/mirror/.5`} rows={2} />

</CommandEntry>

## Sub Oscillator

A secondary oscillator tuned octaves below the main oscillator. Works with all basic oscillators (sine, tri, saw, zaw, pulse, pulze) and spread mode.

<CommandEntry name="sub" type="number" min={0} max={1} default={0} mod>

Mix level of the sub oscillator. At 0 the sub is silent, at 1 it matches the main oscillator volume.

<CodeEditor code={`/sound/saw/sub/.5`} rows={2} />

<CodeEditor code={`/sound/saw/sub/0>1:2/freq/55/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="suboct" type="number" min={1} max={3} default={1}>

Octave offset below the main oscillator. 1 means one octave down, 2 means two octaves down, 3 means three octaves down.

<CodeEditor code={`/sound/saw/sub/.5/suboct/2`} rows={2} />

</CommandEntry>

<CommandEntry name="subwave" type="enum" values={["tri", "sine", "square"]} default="tri">

Waveform of the sub oscillator.

<CodeEditor code={`/sound/saw/sub/.5/subwave/sine`} rows={2} />

</CommandEntry>

## Hard Sync

Classic analog hard sync. A hidden master oscillator runs at the note frequency and forces the main oscillator's phase to reset every time it wraps. Sweeping the ratio produces the characteristic sync sweep. Works with all basic oscillators (sine, tri, saw, zaw, pulse, pulze) as well as the `add` and `osc` sources.

<CommandEntry name="sync" type="number" min={1} max={64} default={1} mod>

Sync ratio. The main oscillator runs at `freq * sync` and is reset each time the hidden master at `freq` wraps. `1` disables sync (no effect). Modulate it for the classic sweep.

<CodeEditor code={`/sound/saw/sync/3`} rows={2} />

<CodeEditor code={`/sound/saw/sync/1~8:2/freq/110/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="syncphase" type="number" min={0} max={1} default={0} mod>

Phase value the main oscillator resets to on each sync event. Non-zero values shift the reset point for additional timbral variation. Aliased as `syncph`.

<CodeEditor code={`/sound/saw/sync/4/syncphase/.25`} rows={2} />

</CommandEntry>

## Additive Partials

<CommandEntry name="partials" type="number" min={1} max={32} default={32} mod>

Number of active harmonics for the `add` source. Fractional values smoothly crossfade the last partial. Lower values produce simpler timbres, higher values produce richer spectra.

<CodeEditor code={`/sound/add/note/48/partials/4`} rows={2} />

<CodeEditor code={`/sound/add/note/48/partials/1~32:3/gate/4`} rows={2} />

</CommandEntry>
