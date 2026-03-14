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

A state variable bandpass filter (TPT/SVF) that attenuates frequencies outside a band around the center frequency. The center frequency supports inline modulation (`~`, `>`, `^`).

<CommandEntry name="bpf" type="number" min={20} max={20000} unit="Hz" mod>

Center frequency in Hz. Frequencies outside the band are attenuated.

<CodeEditor code={`/sound/saw/bpf/800`} rows={2} />

<CodeEditor code={`/sound/saw/bpf/200^4000:0.01:0.2:0.5:0.3/decay/1/gate/2`} rows={2} />

</CommandEntry>

<CommandEntry name="bpq" type="number" min={0} max={1} default={0.2} mod>

Resonance (0-1). Higher values narrow the passband.

<CodeEditor code={`/sound/saw/bpf/800/bpq/.5`} rows={2} />

</CommandEntry>
