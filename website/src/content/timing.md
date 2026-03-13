---
title: "Timing"
slug: "timing"
group: "synthesis"
order: 101
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

The engine clock starts at 0 and advances with each sample. Events with <code>time</code> are scheduled and fired when the clock reaches that value. The <code>gate</code> sets how long the gate stays open before triggering release.

<CommandEntry name="time" type="number" min={0} default={0} unit="s">

The time at which the voice should start. Defaults to 0.

<CodeEditor code={`/freq/330/time/0\n\n/freq/440/time/0.5`} rows={4} />

</CommandEntry>

<CommandEntry name="gate" type="number" min={0} default={1} unit="s">

The gate duration in seconds. Controls how long the note is held before triggering the release phase. A value of 0 means infinite sustain (the voice will play until released explicitly).

<CodeEditor code={`/gate/.5`} rows={2} />

</CommandEntry>
