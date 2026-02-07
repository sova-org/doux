---
title: "Flanger"
slug: "flanger"
group: "effects"
order: 201
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

LFO-modulated delay (0.5-10ms) with feedback and linear interpolation. Output is 50% dry, 50% wet.

<CommandEntry name="flanger" type="number" min={0} default={0} unit="Hz" mod>

Flanger LFO rate in Hz. Creates sweeping comb filter effect with short delay modulation.

<CodeEditor code={`/sound/saw/freq/100/flanger/0.5`} rows={2} />

<CodeEditor code={`/sound/tri/freq/200/flanger/2/flangerdepth/0.8`} rows={2} />

</CommandEntry>

<CommandEntry name="flangerdepth" type="number" min={0} max={1} default={0.5} mod>

Flanger modulation depth (0-1). Controls delay time sweep range.

<CodeEditor code={`/sound/saw/freq/100/flanger/1/flangerdepth/0.3`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/80/flanger/0.5/flangerdepth/0.9`} rows={2} />

</CommandEntry>

<CommandEntry name="flangerfeedback" type="number" min={0} max={0.95} default={0} mod>

Flanger feedback amount (0-0.95).

<CodeEditor code={`/sound/saw/freq/100/flanger/1/flangerfeedback/0.7`} rows={2} />

<CodeEditor code={`/sound/tri/freq/150/flanger/0.3/flangerdepth/0.5/flangerfeedback/0.9`} rows={2} />

</CommandEntry>
