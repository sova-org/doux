---
title: "Steep Lowpass Filter"
slug: "steep-lowpass"
group: "effects"
order: 114
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A 24 dB/oct steep lowpass filter built by cascading two state variable stages. Steeper rolloff than `lpf` (12 dB/oct), with a cleaner, more focused single resonance peak than the Moog-style `llpf`. Sits in the "Curtis/Roland" sonic family — punchy and surgical, not throaty. Cutoff supports inline modulation (`~`, `>`, `^`).

<CommandEntry name="slpf" type="number" min={20} max={20000} unit="Hz" mod>

Cutoff frequency in Hz. Frequencies above the cutoff are attenuated at 24 dB/oct.

<CodeEditor code={`/sound/saw/slpf/400`} rows={2} />

<CodeEditor code={`/sound/saw/slpf/200~5000:2/decay/2/gate/3`} rows={2} />

<CodeEditor code={`/sound/saw/slpf/200^8000:0.01:0.1:0.5:0.3/decay/1/gate/2`} rows={2} />

</CommandEntry>

<CommandEntry name="slpq" type="number" min={0} max={1} default={0.2} mod>

Resonance (0-1). Single focused peak near the cutoff. Self-oscillates cleanly at 1.0.

<CodeEditor code={`/sound/saw/slpf/400/slpq/.7`} rows={2} />

</CommandEntry>
