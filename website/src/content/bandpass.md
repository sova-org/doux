---
title: "Bandpass Filter"
slug: "bandpass"
group: "effects"
order: 112
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A state variable bandpass filter (TPT/SVF) that attenuates frequencies outside a band around the center frequency. Each filter has its own ADSR envelope that modulates the center frequency.

<CommandEntry name="bpf" type="number" min={20} max={20000} unit="Hz" mod>

Center frequency in Hz. Frequencies outside the band are attenuated.

<CodeEditor code={`/sound/saw/bpf/800`} rows={2} />

</CommandEntry>

<CommandEntry name="bpq" type="number" min={0} max={1} default={0.2} mod>

Resonance (0-1). Higher values narrow the passband.

<CodeEditor code={`/sound/saw/bpf/800/bpq/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="bpe" type="number" default={0}>

Envelope amount. Positive values sweep the center up, negative values sweep down.

<CodeEditor code={`/sound/saw/bpf/800/bpe/5/bpd/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="bpa" type="number" min={0} default={0} unit="s">

Envelope attack time in seconds.

<CodeEditor code={`/sound/saw/bpf/800/bpa/.2`} rows={2} />

</CommandEntry>

<CommandEntry name="bpd" type="number" min={0} default={0} unit="s">

Envelope decay time in seconds.

<CodeEditor code={`/sound/saw/bpf/800/bpd/.2`} rows={2} />

</CommandEntry>

<CommandEntry name="bps" type="number" min={0} max={1} default={1}>

Envelope sustain level (0-1).

<CodeEditor code={`/sound/saw/bpf/800/bpd/.2/bps/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="bpr" type="number" min={0} default={0} unit="s">

Envelope release time in seconds.

<CodeEditor code={`/sound/saw/bpf/800/bpr/.25/duration/.1/release/.25`} rows={2} />

</CommandEntry>
