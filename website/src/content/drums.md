---
title: "Drums"
slug: "drums"
group: "sources"
order: 2
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Synthesized percussion. Each drum has percussive defaults so it sounds right without extra parameters. All tonal drums (kick, snare, tom, rim) support `wave` to change the oscillator waveform: `0` = sine (default), `0.5` = triangle, `1` = sawtooth. Values in between crossfade smoothly.

<CommandEntry name="kick" type="source">

Pitched body with sweep and optional saturation. Aliases: `bd`. Default freq: 55 Hz.

- `morph` — sweep depth (subtle to boomy)
- `harmonics` — sweep speed
- `timbre` — saturation
- `wave` — oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)

<CodeEditor code={`/sound/kick`} rows={2} />

<CodeEditor code={`/sound/kick/freq/45/morph/0.6/harmonics/0.4/decay/0.4`} rows={2} />

<CodeEditor code={`/sound/kick/wave/0.5/morph/0.3/decay/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="snare" type="source">

Body + noise. Aliases: `sd`. Default freq: 180 Hz.

- `timbre` — body/noise mix
- `harmonics` — noise brightness
- `wave` — oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)

<CodeEditor code={`/sound/snare`} rows={2} />

<CodeEditor code={`/sound/sd/freq/200/timbre/0.8/harmonics/0.7/decay/0.2`} rows={2} />

</CommandEntry>

<CommandEntry name="hat" type="source">

Phase-modulated metallic tone through a resonant lowpass. Aliases: `hh`, `hihat`. Default freq: 320 Hz.

- `morph` — clean to metallic
- `harmonics` — dark to bright
- `timbre` — filter resonance

<CodeEditor code={`/sound/hat`} rows={2} />

<CodeEditor code={`/sound/hh/freq/400/morph/0.6/harmonics/0.8/decay/0.15`} rows={2} />

</CommandEntry>

<CommandEntry name="tom" type="source">

Pitched body with gentle sweep and optional noise. Default freq: 120 Hz.

- `morph` — sweep depth
- `harmonics` — sweep speed
- `timbre` — noise amount
- `wave` — oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)

<CodeEditor code={`/sound/tom`} rows={2} />

<CodeEditor code={`/sound/tom/freq/90/morph/0.4/decay/0.3`} rows={2} />

</CommandEntry>

<CommandEntry name="rim" type="source">

Short pitched click with noise. Aliases: `rimshot`, `rs`. Default freq: 400 Hz.

- `morph` — pitch sweep
- `harmonics` — noise brightness
- `timbre` — body/noise mix
- `wave` — oscillator waveform (0 sine, 0.5 triangle, 1 sawtooth)

<CodeEditor code={`/sound/rim`} rows={2} />

<CodeEditor code={`/sound/rs/morph/0.4/timbre/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="cowbell" type="source">

Two detuned oscillators through a bandpass. Aliases: `cb`. Default freq: 540 Hz.

- `morph` — detune amount
- `harmonics` — brightness
- `timbre` — metallic bite

<CodeEditor code={`/sound/cowbell`} rows={2} />

<CodeEditor code={`/sound/cb/morph/0.3/harmonics/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="cymbal" type="source">

Inharmonic metallic wash with filtered noise. Aliases: `crash`, `cy`. Default freq: 420 Hz.

- `morph` — ratio spread (bell-like to crash)
- `harmonics` — brightness (dark to sizzly)
- `timbre` — noise amount (pure metallic to noisy crash)

<CodeEditor code={`/sound/cymbal`} rows={2} />

<CodeEditor code={`/sound/crash/morph/0.7/decay/0.8`} rows={2} />

</CommandEntry>
