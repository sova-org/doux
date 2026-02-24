---
title: "Compressor"
slug: "compressor"
group: "effects"
order: 210
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Sidechain compressor. Ducks this orbit's output based on another orbit's level — classic pumping effect.

<CommandEntry name="comp" type="number" min={0} max={1} default={0} mod>

Duck amount. 0 = off, 1 = full duck. Point it at another orbit with comporbit.

<CodeEditor code={`/sound/saw/freq/100/orbit/1/verb/0.3/comp/0.8/comporbit/0`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/80/orbit/1/comp/0.5/compattack/0.005/comprelease/0.2/comporbit/0`} rows={2} />

</CommandEntry>

<CommandEntry name="compattack" type="number" min={0.001} max={1} default={0.01} unit="s">

How fast the ducker reacts. Short = tight pumping, long = slow swell. Alias: `cattack`.

<CodeEditor code={`/sound/saw/freq/100/orbit/1/comp/0.8/compattack/0.001/comporbit/0`} rows={2} />

</CommandEntry>

<CommandEntry name="comprelease" type="number" min={0.001} max={2} default={0.15} unit="s">

Recovery time after the sidechain drops. Longer = more pronounced pump. Alias: `crelease`.

<CodeEditor code={`/sound/saw/freq/100/orbit/1/comp/0.8/comprelease/0.4/comporbit/0`} rows={2} />

</CommandEntry>

<CommandEntry name="comporbit" type="number" min={0} max={7} default={0}>

Which orbit drives the compression. Typically the orbit carrying your kick or bass. Alias: `corbit`.

<CodeEditor code={`/sound/saw/freq/100/orbit/1/comp/0.8/comporbit/0`} rows={2} />

</CommandEntry>
