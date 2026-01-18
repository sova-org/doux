---
title: "Chorus"
slug: "chorus"
group: "effects"
order: 202
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

A rich chorus effect that adds depth and movement to any sound.

<CommandEntry name="chorus" type="number" min={0} default={0} unit="Hz">

Chorus LFO rate in Hz.

<CodeEditor code={`/sound/saw/freq/100/chorus/0.1`} rows={2} />

<CodeEditor code={`/sound/saw/freq/100/chorus/0.05/chorusdepth/0.7`} rows={2} />

</CommandEntry>

<CommandEntry name="chorusdepth" type="number" min={0} max={1} default={0.5}>

Chorus modulation depth (0-1).

<CodeEditor code={`/sound/saw/freq/200/chorus/0.5/chorusdepth/0.3`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/100/chorus/0.2/chorusdepth/0.9`} rows={2} />

</CommandEntry>

<CommandEntry name="chorusdelay" type="number" min={0} default={20} unit="ms">

Chorus base delay time in milliseconds.

<CodeEditor code={`/sound/saw/freq/200/chorus/0.3/chorusdelay/20`} rows={2} />

<CodeEditor code={`/sound/saw/freq/200/chorus/0.3/chorusdelay/30`} rows={2} />

</CommandEntry>
