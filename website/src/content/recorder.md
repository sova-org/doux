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

The recorder captures the master output into a buffer. <code>/doux/rec/myloop</code> starts a recording under that name (any name works); a nameless <code>/doux/rec</code> is a no-op. Stop explicitly with <code>/doux/rec/endrec/1</code>. The buffer is registered as a sample under the chosen name and can be played back with all standard parameters. Maximum ~10 minutes. Native only.

<CommandEntry name="rec" type="source">

Start recording under an explicit name, passed as a positional argument. A nameless <code>/doux/rec</code> does nothing. Recording continues until you send <code>endrec</code>.

<CodeEditor code={`/doux/rec/myloop`} rows={2} />

<CodeEditor code={`/s/myloop`} rows={2} />

</CommandEntry>

<CommandEntry name="endrec" type="source">

Stop the active recording and register the captured buffer as a sample. This is the only way to stop — recording no longer toggles.

<CodeEditor code={`/doux/rec/endrec/1`} rows={2} />

</CommandEntry>

<CommandEntry name="overdub" type="source">

Layers new output on top of an existing recording. Wraps at buffer end. Falls back to fresh recording if the target does not exist.

<CodeEditor code={`/doux/rec/myloop/overdub/1`} rows={2} />

</CommandEntry>

Recorded samples work like any other sample: <code>begin</code>, <code>end</code>, <code>speed</code>, filters, effects all apply.

<CodeEditor code={`/s/myloop/begin/0.25/end/0.75`} rows={2} />

<CodeEditor code={`/s/myloop/speed/0.5/lpf/800/verb/0.3`} rows={2} />
