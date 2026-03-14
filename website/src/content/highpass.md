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

A state variable highpass filter (TPT/SVF) that attenuates frequencies below the cutoff. The cutoff frequency supports inline modulation (`~`, `>`, `^`).

<CommandEntry name="hpf" type="number" min={20} max={20000} unit="Hz" mod>

Cutoff frequency in Hz. Frequencies below this are attenuated.

<CodeEditor code={`/sound/saw/hpf/500`} rows={2} />

<CodeEditor code={`/sound/saw/hpf/100^4000:0.01:0.1:0.5:0.3/decay/1/gate/2`} rows={2} />

</CommandEntry>

<CommandEntry name="hpq" type="number" min={0} max={1} default={0.2} mod>

Resonance (0-1). Boosts frequencies near the cutoff.

<CodeEditor code={`/sound/saw/hpf/500/hpq/.5`} rows={2} />

</CommandEntry>
