---
title: "Amplitude Modulation"
slug: "am"
group: "synthesis"
order: 109
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Amplitude modulation multiplies the signal by a modulating oscillator. The formula preserves the original signal at depth 0: <code>signal &#42;= 1.0 + modulator &#42; depth</code>. This creates sidebands at <code>carrier ± modulator</code> frequencies while keeping the carrier present.

<CommandEntry name="am" type="number" min={0} default={0} unit="Hz" mod>

AM oscillator frequency in Hz. When set above 0, an LFO modulates the signal amplitude.

<CodeEditor code={`/freq/300/am/4/amdepth/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="amdepth" type="number" min={0} max={1} default={0.5} mod>

Modulation depth (0-1). At 0, the signal is unchanged. At 1, the signal varies between 0 and 2x its amplitude.

<CodeEditor code={`/freq/300/am/2/amdepth/1.0`} rows={2} />

</CommandEntry>

<CommandEntry name="amshape" type="string" default="sine">

AM LFO waveform shape. Options: `sine`, `tri`, `saw`, `square`, `sh` (sample-and-hold).

<CodeEditor code={`/freq/300/am/4/amdepth/0.8/amshape/square`} rows={2} />

</CommandEntry>
