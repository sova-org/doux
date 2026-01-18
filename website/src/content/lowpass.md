---
title: "Lowpass Filter"
slug: "lowpass"
group: "effects"
order: 110
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A lowpass filter attenuates frequencies above the cutoff. Each filter has its own ADSR envelope that modulates the cutoff frequency.

<CommandEntry name="lpf" type="number" min={20} max={20000} unit="Hz">

Cutoff frequency in Hz. Frequencies above this are attenuated.

<CodeEditor code={`/sound/saw/lpf/200`} rows={2} />

</CommandEntry>

<CommandEntry name="lpq" type="number" min={0} max={1} default={0.2}>

Resonance (0-1). Boosts frequencies near the cutoff.

<CodeEditor code={`/sound/saw/lpf/200/lpq/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="lpe" type="number" default={0}>

Envelope amount. Positive values sweep the cutoff up, negative values sweep down.

<CodeEditor code={`/sound/saw/lpf/100/lpe/5/lpd/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="lpa" type="number" min={0} default={0} unit="s">

Envelope attack time in seconds.

<CodeEditor code={`/sound/saw/lpf/100/lpa/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="lpd" type="number" min={0} default={0} unit="s">

Envelope decay time in seconds.

<CodeEditor code={`/sound/saw/lpf/100/lpd/.25`} rows={2} />

</CommandEntry>

<CommandEntry name="lps" type="number" min={0} max={1} default={1}>

Envelope sustain level (0-1).

<CodeEditor code={`/sound/saw/lpf/100/lpd/.25/lps/.4`} rows={2} />

</CommandEntry>

<CommandEntry name="lpr" type="number" min={0} default={0} unit="s">

Envelope release time in seconds.

<CodeEditor code={`/sound/saw/lpf/100/lpr/.25/duration/.1/release/.25`} rows={2} />

</CommandEntry>
