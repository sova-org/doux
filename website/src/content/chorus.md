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

<CommandEntry name="chorus" type="number" min={0} default={0} unit="Hz" mod>

Chorus LFO rate in Hz.

<CodeEditor code={`/sound/saw/freq/100/chorus/0.1`} rows={2} />

<CodeEditor code={`/sound/saw/freq/100/chorus/0.05/chorusdepth/0.7`} rows={2} />

</CommandEntry>

<CommandEntry name="chorusdepth" type="number" min={0} max={1} default={0.5} mod>

Chorus modulation depth (0-1).

<CodeEditor code={`/sound/saw/freq/200/chorus/0.5/chorusdepth/0.3`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/100/chorus/0.2/chorusdepth/0.9`} rows={2} />

</CommandEntry>

<CommandEntry name="chorusdelay" type="number" min={0} default={20} unit="ms" mod>

Chorus base delay time in milliseconds.

<CodeEditor code={`/sound/saw/freq/200/chorus/0.3/chorusdelay/20`} rows={2} />

<CodeEditor code={`/sound/saw/freq/200/chorus/0.3/chorusdelay/30`} rows={2} />

</CommandEntry>

<CommandEntry name="chorustype" type="enum" default="classic" values={["classic", "ensemble", "dimension"]}>

Chorus voicing. The default reproduces the original 3-voice chorus exactly.

<ul>
<li><strong>classic</strong> — 3-voice chorus (the default).</li>
<li><strong>ensemble</strong> — 4 voices, wider detune. Juno-style lushness.</li>
<li><strong>dimension</strong> — 2 quadrature voices, deeper, no centre.</li>
</ul>

<CodeEditor code={`/sound/saw/freq/100/chorus/0.3/chorustype/ensemble`} rows={2} />

</CommandEntry>
