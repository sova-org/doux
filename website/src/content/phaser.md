---
title: "Phaser"
slug: "phaser"
group: "effects"
order: 200
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Two cascaded notch filters (offset by 282Hz) with LFO-modulated center frequency.

<CommandEntry name="phaser" type="number" min={0} default={0} unit="Hz">

Phaser LFO rate in Hz. Creates sweeping notch filter effect.

<CodeEditor code={`/sound/saw/freq/50/phaser/0.5`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/phaser/2/phaserdepth/0.9`} rows={2} />

</CommandEntry>

<CommandEntry name="phaserdepth" type="number" min={0} max={1} default={0.5}>

Phaser effect intensity (0-1). Controls resonance and wet/dry mix.

<CodeEditor code={`/sound/saw/freq/50/phaser/1/phaserdepth/0.5`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/phaser/0.25/phaserdepth/1.0`} rows={2} />

</CommandEntry>

<CommandEntry name="phasersweep" type="number" min={0} default={2000} unit="Hz">

Phaser frequency sweep range in Hz. Default is 2000 (±2000Hz sweep).

<CodeEditor code={`/sound/saw/freq/50/phaser/1/phasersweep/4000`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/phaser/0.5/phasersweep/500`} rows={2} />

</CommandEntry>

<CommandEntry name="phasercenter" type="number" min={20} max={20000} default={1000} unit="Hz">

Phaser center frequency in Hz. Default is 1000Hz.

<CodeEditor code={`/sound/saw/freq/50/phaser/1/phasercenter/500`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/phaser/2/phasercenter/2000`} rows={2} />

</CommandEntry>
