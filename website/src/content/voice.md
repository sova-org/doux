---
title: "Voice"
slug: "voice"
group: "synthesis"
order: 103
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Doux is a polyphonic synthesizer with up to 32 simultaneous voices. By default, each event allocates a new voice automatically. When a voice finishes (envelope reaches zero), it is freed and recycled. The <code>voice</code> parameter lets you take manual control over voice allocation, enabling parameter updates on active voices (e.g., pitch slides with <code>glide</code>) or retriggering with <code>reset</code>.

<CommandEntry name="voice" type="number" min={0}>

The voice index to use. If set, voice allocation will be skipped and the selected voice will be used. If the voice is still active, the sent params will update the active voice.

<CodeEditor code={`/voice/0/freq/220\n\n/voice/0/freq/330/time/.5`} rows={4} />

</CommandEntry>

<CommandEntry name="reset" type="boolean" default={false}>

Only has an effect when used together with voice. If set to 1, the selected voice will be reset, even when it's still active. This will cause envelopes to retrigger for example.

<CodeEditor code={`/voice/0/freq/220/attack/.1\n\n/voice/0/freq/330/time/.25/reset/1`} rows={4} />

</CommandEntry>
