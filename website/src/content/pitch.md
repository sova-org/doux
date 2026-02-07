---
title: "Pitch"
slug: "pitch"
group: "synthesis"
order: 100
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Pitch control for all sources, including audio samples.

<CommandEntry name="freq" type="number" min={20} max={20000} default={330} unit="Hz">

The frequency of the sound. Has no effect on noise.

<CodeEditor code={`/freq/400`} rows={2} />

<CodeEditor code={`/freq/800`} rows={2} />

<CodeEditor code={`/freq/1200`} rows={2} />

</CommandEntry>

<CommandEntry name="note" type="number" min={0} max={127} unit="midi">

The note (midi number) that should be played.
If both note and freq is set, freq wins.

<CodeEditor code={`/note/60\n\n/note/67`} rows={4} />

<CodeEditor code={`/note/48\n\n/note/60\n\n/note/63\n\n/note/67`} rows={8} />

</CommandEntry>

<CommandEntry name="speed" type="number" default={1} mod>

Multiplies with the source frequency or buffer playback speed.

<CodeEditor code={`/sound/saw/freq/220/speed/0.5`} rows={2} />

<CodeEditor code={`/sound/saw/freq/220/speed/1.5`} rows={2} />

</CommandEntry>

<CommandEntry name="detune" type="number" default={0} unit="cents" mod>

Shifts the pitch by the given amount in cents. 100 cents = 1 semitone.

<CodeEditor code={`/freq/440/detune/50`} rows={2} />

<CodeEditor code={`/freq/440/detune/-50`} rows={2} />

</CommandEntry>

<CommandEntry name="glide" type="number" min={0} default={0} unit="s">

Creates a pitch slide when changing the frequency of an active voice.
Only has an effect when used with <code>voice</code>.

<CodeEditor code={`/voice/0/freq/220\n\n/voice/0/freq/330/glide/0.5/time/0.25`} rows={4} />

</CommandEntry>
