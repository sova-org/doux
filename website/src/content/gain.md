---
title: "Gain"
slug: "gain"
group: "synthesis"
order: 105
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

The signal path is: oscillator → <code>gain &#42; velocity</code> → filters → distortion → modulation → phaser/flanger → <code>envelope &#42; postgain</code> → chorus → <code>pan</code>.

<CommandEntry name="gain" type="number" min={0} default={1}>

Pre-filter gain multiplier. Applied before filters and distortion, combined with <code>velocity</code> as <code>gain &#42; velocity</code>.

<CodeEditor code={`/sound/saw/gain/0.2`} rows={2} />

</CommandEntry>

<CommandEntry name="postgain" type="number" min={0} default={1}>

Post-effects gain multiplier. Applied after phaser/flanger, combined with the envelope as <code>envelope &#42; postgain</code>.

<CodeEditor code={`/sound/saw/postgain/0.2\n\n/sound/saw/postgain/1/time/0.25`} rows={4} />

</CommandEntry>

<CommandEntry name="velocity" type="number" min={0} max={1} default={1}>

Multiplied with <code>gain</code> before filters. Also passed as <code>accent</code> to Plaits engines.

<CodeEditor code={`/sound/saw/velocity/0.2\n\n/sound/saw/velocity/1/time/0.25`} rows={4} />

</CommandEntry>

<CommandEntry name="pan" type="number" min={0} max={1} default={0.5}>

Stereo position using constant-power panning: <code>left = cos(pan &#42; π/2)</code>, <code>right = sin(pan &#42; π/2)</code>. 0 = left, 0.5 = center, 1 = right.

<CodeEditor code={`/pan/0/freq/329\n\n/pan/1/freq/331`} rows={4} />

</CommandEntry>

<CommandEntry name="width" type="number" min={0} max={2} default={1}>

Stereo width using mid-side processing. At 0 the signal collapses to mono, at 1 it is unchanged, above 1 the stereo image is exaggerated.

<CodeEditor code={`/sound/saw/freq/50/spread/5/width/0`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/spread/5/width/2`} rows={2} />

</CommandEntry>

<CommandEntry name="haas" type="number" min={0} max={35} default={0} unit="ms">

Haas effect. Delays the right channel by a short amount (1-35ms) to create spatial placement without changing volume. Small values (1-10ms) widen the image, larger values (10-35ms) create a distinct echo.

<CodeEditor code={`/sound/saw/freq/50/haas/8`} rows={2} />

<CodeEditor code={`/sound/saw/freq/50/haas/25`} rows={2} />

</CommandEntry>
