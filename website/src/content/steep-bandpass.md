---
title: "Steep Bandpass Filter"
slug: "steep-bandpass"
group: "effects"
order: 116
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A 24 dB/oct steep bandpass filter built by cascading two state variable stages. Narrower and more focused than `bpf` (12 dB/oct), distinct character from the Moog-style `lbpf`. Center frequency supports inline modulation (`~`, `>`, `^`).

<CommandEntry name="sbpf" type="number" min={20} max={20000} unit="Hz" mod>

Center frequency in Hz. Frequencies far from the center are attenuated at 24 dB/oct per side.

<CodeEditor code={`/sound/saw/sbpf/1000`} rows={2} />

<CodeEditor code={`/sound/saw/sbpf/200~5000:2/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="sbpq" type="number" min={0} max={1} default={0.2} mod>

Resonance (0-1). Tightens the bandpass peak. Self-oscillates cleanly at 1.0.

<CodeEditor code={`/sound/saw/sbpf/1000/sbpq/.7`} rows={2} />

</CommandEntry>
