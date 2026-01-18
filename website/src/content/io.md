---
title: "Io"
slug: "io"
group: "sources"
order: 2
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

This special source allows you to create a live audio input (microphone) source. Click the 'Enable Mic' button in the nav bar first. Effects chain applies normally, envelopes are applied to the input signal too.

<CommandEntry name="live" type="source">

Live audio input (microphone). Click the 'Enable Mic' button in the nav bar first. Effects chain applies normally.

<CodeEditor code={`/sound/live`} rows={2} />

<CodeEditor code={`/sound/live/lpf/800`} rows={2} />

<CodeEditor code={`/sound/live/verb/0.5`} rows={2} />

</CommandEntry>
