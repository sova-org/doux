---
title: "Multichannel"
slug: "multichannel"
group: "synthesis"
order: 116
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

`superpan` rings a voice's stereo signal around a set of output **pairs** using equal-power azimuth panning, in the style of SuperCollider's `PanAz`. It is meant for multi-speaker setups with more than two output channels. Disabled by default: when `superpan` is unset, the voice uses the normal stereo `pan` and orbit routing.

<CommandEntry name="superpan" type="number" min={0} max={1} mod>

Azimuth position around the ring of output pairs (wraps 0..1). Setting this switches the voice from stereo `pan` to multichannel panning. Aliased as <code>span</code>.

<CodeEditor code={`/sound/saw/superpan/0.5`} rows={2} />

<CodeEditor code={`/sound/saw/superpan/0~1:2/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="superwidth" type="number" min={1} default={2} mod>

Number of adjacent output pairs lit. About 2 keeps the source localised to one pair; larger values spread it wider across the ring. Gains are normalised so loudness stays constant as the source moves. Aliased as <code>swidth</code>.

<CodeEditor code={`/sound/saw/superpan/0.5/superwidth/4`} rows={2} />

</CommandEntry>

<CommandEntry name="speakers" type="source">

Ordered, 1-based list of output pairs the ring spans, e.g. <code>1,3,5,7</code>. Pair <code>p</code> drives channels <code>2p</code> and <code>2p+1</code>. Empty (omitted) means all pairs, in order. Aliased as <code>spk</code>.

<CodeEditor code={`/sound/saw/superpan/0~1:4/speakers/1,3,5,7/decay/2/gate/3`} rows={2} />

</CommandEntry>
