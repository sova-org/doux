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

The engine clock starts at 0 and advances with each sample. Events with <code>time</code> are scheduled and fired when the clock reaches that value. The <code>duration</code> sets how long the gate stays open before triggering release.

<CommandEntry name="time" type="number" min={0} default={0} unit="s">

The time at which the voice should start. Defaults to 0.

<CodeEditor code={`/freq/330/time/0\n\n/freq/440/time/0.5`} rows={4} />

</CommandEntry>

<CommandEntry name="duration" type="number" min={0} unit="s">

The duration (seconds) of the gate phase. If not set, the voice will play indefinitely, until released explicitly.

<CodeEditor code={`/duration/.5`} rows={2} />

</CommandEntry>
