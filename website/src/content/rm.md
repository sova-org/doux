---
title: "Ring Modulation"
slug: "rm"
group: "synthesis"
order: 110
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Ring modulation is a crossfade between dry signal and full multiplication: <code>signal &#42;= (1.0 - depth) + modulator &#42; depth</code>. Unlike AM, ring modulation at full depth removes the carrier entirely, leaving only sum and difference frequencies at <code>carrier ± modulator</code>.

<CommandEntry name="rm" type="number" min={0} default={0} unit="Hz" mod>

Ring modulation oscillator frequency in Hz. When set above 0, an LFO multiplies the signal.

<CodeEditor code={`/freq/300/rm/440/rmdepth/1.0`} rows={2} />

</CommandEntry>

<CommandEntry name="rmdepth" type="number" min={0} max={1} default={1} mod>

Modulation depth (0-1). At 0, the signal is unchanged. At 1, full ring modulation with no dry signal.

<CodeEditor code={`/freq/300/rm/440/rmdepth/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="rmshape" type="string" default="sine">

Ring modulation LFO waveform shape. Options: `sine`, `tri`, `saw`, `square`, `sh` (sample-and-hold).

<CodeEditor code={`/freq/300/rm/8/rmdepth/1/rmshape/sh`} rows={2} />

</CommandEntry>
