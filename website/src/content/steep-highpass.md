---
title: "Steep Highpass Filter"
slug: "steep-highpass"
group: "effects"
order: 115
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A 24 dB/oct steep highpass filter built by cascading two state variable stages. Steeper rolloff than `hpf` (12 dB/oct), distinct character from the Moog-style `lhpf`: clean single peak, punchy and focused. Cutoff supports inline modulation (`~`, `>`, `^`).

<CommandEntry name="shpf" type="number" min={20} max={20000} unit="Hz" mod>

Cutoff frequency in Hz. Frequencies below the cutoff are attenuated at 24 dB/oct.

<CodeEditor code={`/sound/saw/shpf/800`} rows={2} />

<CodeEditor code={`/sound/saw/shpf/200~5000:2/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="shpq" type="number" min={0} max={1} default={0.2} mod>

Resonance (0-1). Single focused peak near the cutoff. Self-oscillates cleanly at 1.0.

<CodeEditor code={`/sound/saw/shpf/800/shpq/.7`} rows={2} />

</CommandEntry>
