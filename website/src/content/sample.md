---
title: "Sample"
slug: "sample"
group: "synthesis"
order: 111
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Doux can play back audio samples organized in folders. Point to a samples directory using the <code>--samples</code> flag. Each subfolder becomes a sample bank accessible via <code>/s/folder_name</code>. Use <code>/n/</code> to index into a folder.

<CommandEntry name="n" type="number" min={0} default={0}>

Sample index within the folder. If the index exceeds the number of samples, it wraps around using modulo. Samples in a folder are indexed starting from 0.

<CodeEditor code={`/s/crate_rd/n/0`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/2`} rows={2} />

</CommandEntry>

<CommandEntry name="begin" type="number" min={0} max={1} default={0}>

Sample start position (0-1). 0 = beginning, 0.5 = middle, 1 = end. Only works with samples.

<CodeEditor code={`/s/crate_rd/n/2/begin/0.0`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/2/begin/0.25`} rows={2} />

</CommandEntry>

<CommandEntry name="end" type="number" min={0} max={1} default={1}>

Sample end position (0-1). 0 = beginning, 0.5 = middle, 1 = end. Only works with samples.

<CodeEditor code={`/s/crate_rd/n/2/end/0.05`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/3/end/0.1/speed/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="cut" type="number" min={0}>

Choke group. Voices with the same cut value silence each other. Use for hi-hats where open should be cut by closed.

<CodeEditor code={`/s/crate_hh/n/0/cut/1\n\n/s/crate_hh/n/1/cut/1/time/.25`} rows={4} />

</CommandEntry>

<CommandEntry name="stretch" type="number" min={0} default={1} mod>

Time stretch factor. Controls playback duration independently from pitch.
1 = normal speed, 2 = twice as long (same pitch), 0.5 = half as long (same pitch), 0 = freeze.

<CodeEditor code={`/s/crate_rd/n/0/stretch/2`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/stretch/0.5`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/stretch/0`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/stretch/0.5~2:4`} rows={2} />

</CommandEntry>

<CommandEntry name="grain" type="number" min={0} max={1000} default={0} mod>

Granular grain size in ms. 0 = off, and the sample plays through the ordinary reader.
Any positive value switches playback to a cloud of short overlapping grains, each one windowed so it fades in and out.
This also re-points <code>stretch</code>: instead of driving the phase vocoder it now drives the cloud's scan head, keeping its meaning (1 = normal, 4 = four times as long, 0 = freeze) while the algorithm changes underneath.
<code>speed</code> and pitch still set the grain pitch, so time and pitch stay independent.

Large grains carry a recognisable piece of the source; grains of a few ms are too short to, and what you hear instead is the launch rate as a tone.

<CodeEditor code={`/s/crate_rd/n/0/grain/60`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/grain/3`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/grain/40/stretch/4`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/grain/120/stretch/0`} rows={2} />

</CommandEntry>

<CommandEntry name="spray" type="number" min={0} max={1} default={0} mod>

Grain scatter. Moves each grain's start position within the <code>begin</code>/<code>end</code> region and its placement across the stereo field, both by the same amount.
0 = grains launch from the scan head in order and all sit dead centre, 1 = they land anywhere in the region and anywhere in the image.
Placement is equal-power and normalized to unity at the centre, so <code>spray/0</code> is bit-identical to no placement at all.
Only meaningful with <code>grain</code> set.

<CodeEditor code={`/s/crate_rd/n/0/grain/80/stretch/0/spray/0.15`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/grain/40/spray/1`} rows={2} />

</CommandEntry>

<CommandEntry name="dens" type="number" min={1} max={8} default={2} mod>

Grains overlapping at once. A grain lives <code>grain</code> ms and one launches every <code>grain/dens</code> ms.
2 is the even tiling, where two windows at 50% overlap sum flat. 1 stops them overlapping, so the level notches to silence at every grain boundary and the result is gated. Higher counts thicken the cloud.
Only meaningful with <code>grain</code> set.

<CodeEditor code={`/s/crate_rd/n/0/grain/25/stretch/8/spray/0.4/dens/8`} rows={2} />

<CodeEditor code={`/s/crate_rd/n/0/grain/60/dens/1`} rows={2} />

</CommandEntry>
