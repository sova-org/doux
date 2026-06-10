---
title: "Drums"
slug: "drums"
group: "sources"
order: 2
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
  import ParamTable from '$lib/components/ParamTable.svelte';
</script>

Synthesized percussion. Each drum has percussive defaults so it sounds right without extra parameters. All tonal drums (kick, snare, tom, rim) support `wave` to change the oscillator waveform: `0` = sine (default), `0.5` = triangle, `1` = sawtooth. Values in between crossfade smoothly.

Each drum names its tone-shaping parameters semantically; the generic `timbre`, `harmonics`, and `morph` still work on every drum as aliases.

<CommandEntry name="kick" type="source">

Pitched body with sweep and optional saturation. Aliases: `bd`. Default freq: 55 Hz.

<ParamTable params={[
  { name: "sweep", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "sweep depth (subtle to boomy)" },
  { name: "punch", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "sweep speed" },
  { name: "drive", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "saturation" },
  { name: "wave", alias: "waveform", range: "0–1", default: "0", mod: true, description: "oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)" },
]} />

<CodeEditor code={`/sound/kick`} rows={2} />

<CodeEditor code={`/sound/kick/freq/45/sweep/0.6/punch/0.4/decay/0.4`} rows={2} />

<CodeEditor code={`/sound/kick/wave/0.5/sweep/0.3/decay/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="snare" type="source">

Body + noise. Aliases: `sd`. Default freq: 180 Hz.

<ParamTable params={[
  { name: "snappy", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "body/noise mix" },
  { name: "bright", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "noise brightness" },
  { name: "wave", alias: "waveform", range: "0–1", default: "0", mod: true, description: "oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)" },
]} />

<CodeEditor code={`/sound/snare`} rows={2} />

<CodeEditor code={`/sound/sd/freq/200/snappy/0.8/bright/0.7/decay/0.2`} rows={2} />

</CommandEntry>

<CommandEntry name="hat" type="source">

Phase-modulated metallic tone through a resonant lowpass. Aliases: `hh`, `hihat`. Default freq: 320 Hz.

<ParamTable params={[
  { name: "metal", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "clean to metallic (ratio spread)" },
  { name: "bright", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "dark to bright (filter cutoff)" },
  { name: "reso", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "filter resonance" },
]} />

<CodeEditor code={`/sound/hat`} rows={2} />

<CodeEditor code={`/sound/hh/freq/400/metal/0.6/bright/0.8/decay/0.15`} rows={2} />

</CommandEntry>

<CommandEntry name="tom" type="source">

Pitched body with gentle sweep and optional noise. Default freq: 120 Hz.

<ParamTable params={[
  { name: "sweep", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "sweep depth" },
  { name: "punch", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "sweep speed" },
  { name: "noise", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "stick-noise amount" },
  { name: "wave", alias: "waveform", range: "0–1", default: "0", mod: true, description: "oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)" },
]} />

<CodeEditor code={`/sound/tom`} rows={2} />

<CodeEditor code={`/sound/tom/freq/90/sweep/0.4/decay/0.3`} rows={2} />

</CommandEntry>

<CommandEntry name="rim" type="source">

Short pitched click with noise. Aliases: `rimshot`, `rs`. Default freq: 400 Hz.

<ParamTable params={[
  { name: "shift", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "upper partial shift" },
  { name: "bright", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "click brightness" },
  { name: "ring", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "ring length" },
  { name: "wave", alias: "waveform", range: "0–1", default: "0", mod: true, description: "oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)" },
]} />

<CodeEditor code={`/sound/rim`} rows={2} />

<CodeEditor code={`/sound/rs/shift/0.4/ring/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="cowbell" type="source">

Two detuned oscillators through a bandpass. Aliases: `cb`. Default freq: 540 Hz.

<ParamTable params={[
  { name: "clang", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "detune amount" },
  { name: "bright", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "brightness (bandpass center)" },
  { name: "drive", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "metallic bite (saturation)" },
]} />

<CodeEditor code={`/sound/cowbell`} rows={2} />

<CodeEditor code={`/sound/cb/clang/0.3/bright/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="cymbal" type="source">

Inharmonic metallic wash with filtered noise. Aliases: `crash`, `cy`. Default freq: 420 Hz.

<ParamTable params={[
  { name: "metal", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "ratio spread (bell-like to crash)" },
  { name: "bright", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "brightness (dark to sizzly)" },
  { name: "sizzle", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "noise tail (pure metallic to noisy crash)" },
]} />

<CodeEditor code={`/sound/cymbal`} rows={2} />

<CodeEditor code={`/sound/crash/metal/0.7/decay/0.8`} rows={2} />

</CommandEntry>
