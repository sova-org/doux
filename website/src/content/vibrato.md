---
title: "Vibrato"
slug: "vibrato"
group: "synthesis"
order: 107
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

The pitch of every oscillator can be modulated by a vibrato effect. Vibrato is a technique where the pitch of a note is modulated slightly around a central pitch, creating a shimmering effect.

<CommandEntry name="vib" type="number" min={0} default={0} unit="Hz" mod>

Vibrato frequency (in hertz).

<CodeEditor code={`/vib/8`} rows={2} />

</CommandEntry>

<CommandEntry name="vibmod" type="number" min={0} default={0} unit="semitones" mod>

Vibrato modulation depth (semitones).

<CodeEditor code={`/vib/8/vibmod/24`} rows={2} />

</CommandEntry>

<CommandEntry name="vibshape" type="string" default="sine">

Vibrato LFO waveform shape. Options: `sine`, `tri`, `saw`, `square`, `sh` (sample-and-hold).

<CodeEditor code={`/vib/4/vibmod/1/vibshape/tri`} rows={2} />

</CommandEntry>
