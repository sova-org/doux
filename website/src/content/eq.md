---
title: "EQ"
slug: "eq"
group: "effects"
order: 206
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Per-voice equalizer with a 3-band DJ-style EQ and a single-knob tilt control.

## 3-Band EQ

Fixed-frequency shelving and peaking filters. All gains are in dB: 0 is flat, positive boosts, negative cuts.

<CommandEntry name="eqlo" type="number" default={0} unit="dB">

Low shelf gain at 200Hz.

<CodeEditor code={`/sound/saw/freq/50/eqlo/6`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/eqlo/-12`} rows={2} />

</CommandEntry>

<CommandEntry name="eqmid" type="number" default={0} unit="dB">

Mid peak gain at 1000Hz (Q 0.7).

<CodeEditor code={`/sound/saw/freq/50/eqmid/4`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/eqmid/-6`} rows={2} />

</CommandEntry>

<CommandEntry name="eqhi" type="number" default={0} unit="dB">

High shelf gain at 5000Hz.

<CodeEditor code={`/sound/saw/freq/50/eqhi/3`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/eqlo/6/eqhi/-6`} rows={2} />

</CommandEntry>

## Tilt EQ

<CommandEntry name="tilt" type="number" min={-1} max={1} default={0}>

Spectral tilt using a high shelf at 800Hz. Positive values brighten, negative values darken. Range maps to ±6dB.

<CodeEditor code={`/sound/saw/freq/50/tilt/0.5`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/tilt/-0.8`} rows={2} />

</CommandEntry>
