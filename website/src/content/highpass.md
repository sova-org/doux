---
title: "Highpass Filter"
slug: "highpass"
group: "effects"
order: 111
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A highpass filter attenuates frequencies below the cutoff. Each filter has its own ADSR envelope that modulates the cutoff frequency.

<CommandEntry name="hpf" type="number" min={20} max={20000} unit="Hz">

Cutoff frequency in Hz. Frequencies below this are attenuated.

<CodeEditor code={`/sound/saw/hpf/500`} rows={2} />

</CommandEntry>

<CommandEntry name="hpq" type="number" min={0} max={1} default={0.2}>

Resonance (0-1). Boosts frequencies near the cutoff.

<CodeEditor code={`/sound/saw/hpf/500/hpq/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="hpe" type="number" default={0}>

Envelope amount. Positive values sweep the cutoff up, negative values sweep down.

<CodeEditor code={`/sound/saw/hpf/500/hpe/5/hpd/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="hpa" type="number" min={0} default={0} unit="s">

Envelope attack time in seconds.

<CodeEditor code={`/sound/saw/hpf/500/hpa/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="hpd" type="number" min={0} default={0} unit="s">

Envelope decay time in seconds.

<CodeEditor code={`/sound/saw/hpf/500/hpd/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="hps" type="number" min={0} max={1} default={1}>

Envelope sustain level (0-1).

<CodeEditor code={`/sound/saw/hpf/500/hpd/.25/hps/.4`} rows={2} />

</CommandEntry>

<CommandEntry name="hpr" type="number" min={0} default={0} unit="s">

Envelope release time in seconds.

<CodeEditor code={`/sound/saw/hpf/500/hpr/.25/duration/.1/release/.25`} rows={2} />

</CommandEntry>
