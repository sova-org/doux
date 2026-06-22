---
title: "VinylSim"
slug: "vinyl"
group: "effects"
order: 209
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Vinyl / cassette "character" insert: wow + flutter pitch wobble, band-limiting, tape/vinyl hiss and gentle saturation — the lo-fi degrade box. It sits after the distortion group in the voice chain, so the hiss is shaped by the note's envelope rather than a constant floor.

<CommandEntry name="vinyl" type="number" min={0} max={1} default={0}>

Dry/wet mix (0 = bypass, 1 = full wet).

<CodeEditor code={`/sound/saw/freq/110/vinyl/0.8`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/100/vinyl/0.9/vinylwow/0.6/vinylnoise/0.4`} rows={2} />

</CommandEntry>

<CommandEntry name="vinylwow" type="number" min={0} max={1} default={0.3}>

Wow + flutter depth — slow and fast pitch wobble of worn tape/vinyl.

<CodeEditor code={`/sound/saw/freq/110/vinyl/0.8/vinylwow/0.7`} rows={2} />

</CommandEntry>

<CommandEntry name="vinylnoise" type="number" min={0} max={1} default={0.2}>

Hiss level — the high-passed noise bed (most present on the cassette voicing).

<CodeEditor code={`/sound/saw/freq/110/vinyl/0.8/vinylnoise/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="vinyltone" type="number" min={-1} max={1} default={0}>

Tone tilt — negative darkens, positive brightens the high shelf.

<CodeEditor code={`/sound/saw/freq/110/vinyl/0.8/vinyltone/-0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="vinyltype" type="enum" default="dull" values={["dull", "clear", "cassette"]}>

Voicing. <strong>dull</strong> is the warmest (dullest low-pass), <strong>clear</strong> a touch brighter, <strong>cassette</strong> is mid-focused with the most hiss.

<CodeEditor code={`/sound/saw/freq/110/vinyl/0.8/vinyltype/cassette`} rows={2} />

</CommandEntry>
