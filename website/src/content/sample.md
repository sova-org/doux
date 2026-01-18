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
