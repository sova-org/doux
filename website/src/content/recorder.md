---
title: "Recorder"
slug: "recorder"
group: "synthesis"
order: 115
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

The recorder captures the master output into a buffer. Send <code>/doux/rec</code> to start, send it again to stop. The buffer is registered as a sample and can be played back with all standard parameters. Maximum 60 seconds. Native only.

<CommandEntry name="rec" type="source">

Toggle recording. Auto-named <code>rec0</code>, <code>rec1</code>, etc. First send starts, second send stops and registers the sample.

<CodeEditor code={`/doux/rec`} rows={2} />

<CodeEditor code={`/s/rec0`} rows={2} />

</CommandEntry>

<CommandEntry name="rec + name" type="source">

Set an explicit name via <code>/s/name</code>. The recording is registered under that name for playback.

<CodeEditor code={`/doux/rec/s/myloop`} rows={2} />

<CodeEditor code={`/s/myloop`} rows={2} />

</CommandEntry>

<CommandEntry name="overdub" type="source">

Layers new output on top of an existing recording. Wraps at buffer end. Falls back to fresh recording if the target does not exist.

<CodeEditor code={`/doux/rec/overdub/1/s/myloop`} rows={2} />

</CommandEntry>

Recorded samples work like any other sample: <code>begin</code>, <code>end</code>, <code>speed</code>, filters, effects all apply.

<CodeEditor code={`/s/myloop/begin/0.25/end/0.75`} rows={2} />

<CodeEditor code={`/s/rec0/speed/0.5/lpf/800/verb/0.3`} rows={2} />
