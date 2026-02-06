---
title: "Ladder Filter"
slug: "ladder"
group: "effects"
order: 113
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A Moog-style ladder filter with self-oscillation and analog-modeled nonlinear saturation. Produces a warmer, more aggressive character than the standard filters. Based on the improved virtual analog model by Stefano D'Angelo and Vesa Välimäki. Available as lowpass (`llpf`), highpass (`lhpf`), and bandpass (`lbpf`). Filter envelope parameters are shared with the standard filters (`lpe`/`lpa`/`lpd`/`lps`/`lpr` for lowpass, etc.).

<CommandEntry name="llpf" type="number" min={20} max={20000} unit="Hz">

Ladder lowpass cutoff frequency in Hz.

<CodeEditor code={`/sound/saw/llpf/800/decay/0.5/dur/1`} rows={2} />

</CommandEntry>

<CommandEntry name="lhpf" type="number" min={20} max={20000} unit="Hz">

Ladder highpass cutoff frequency in Hz.

<CodeEditor code={`/sound/saw/lhpf/200/decay/0.5/dur/1`} rows={2} />

</CommandEntry>

<CommandEntry name="lbpf" type="number" min={20} max={20000} unit="Hz">

Ladder bandpass cutoff frequency in Hz.

<CodeEditor code={`/sound/saw/lbpf/500/decay/0.5/dur/1`} rows={2} />

</CommandEntry>

<CommandEntry name="llpq" type="number" min={0} max={1} default={0.2}>

Ladder lowpass resonance (0-1). At high values, the filter self-oscillates.

<CodeEditor code={`/sound/saw/llpf/800/llpq/.8/decay/0.5/dur/1`} rows={2} />

</CommandEntry>

<CommandEntry name="lhpq" type="number" min={0} max={1} default={0.2}>

Ladder highpass resonance (0-1).

<CodeEditor code={`/sound/saw/lhpf/200/lhpq/.5/decay/0.5/dur/1`} rows={2} />

</CommandEntry>

<CommandEntry name="lbpq" type="number" min={0} max={1} default={0.2}>

Ladder bandpass resonance (0-1).

<CodeEditor code={`/sound/saw/lbpf/500/lbpq/.7/decay/0.5/dur/1`} rows={2} />

</CommandEntry>
