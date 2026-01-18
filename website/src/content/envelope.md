---
title: "Envelope"
slug: "envelope"
group: "synthesis"
order: 102
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

The envelope parameters control the shape of the gain envelope over time. It uses a typical ADSR envelope with exponential curves:

- **Attack**: Ramps from 0 to full amplitude. Uses <code>x²</code> (slow start, fast finish).
- **Decay**: Falls from full amplitude to the sustain level. Uses <code>1-(1-x)²</code> (fast drop, slow finish).
- **Sustain**: Holds at a constant level while the note is held.
- **Release**: Falls from the sustain level to 0 when the note ends. Uses <code>1-(1-x)²</code> (fast drop, slow finish). 

<CommandEntry name="attack" type="number" min={0} default={0.001} unit="s">

The duration (seconds) of the attack phase of the gain envelope.

<CodeEditor code={`/attack/.1`} rows={2} />

<CodeEditor code={`/attack/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="decay" type="number" min={0} default={0} unit="s">

The duration (seconds) of the decay phase of the gain envelope.

<CodeEditor code={`/decay/.1`} rows={2} />

<CodeEditor code={`/decay/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="sustain" type="number" min={0} max={1} default={1}>

The sustain level (0-1) of the gain envelope.

<CodeEditor code={`/decay/.1/sustain/.2`} rows={2} />

<CodeEditor code={`/decay/.1/sustain/.6`} rows={2} />

</CommandEntry>

<CommandEntry name="release" type="number" min={0} default={0.005} unit="s">

The duration (seconds) of the release phase of the gain envelope.

<CodeEditor code={`/duration/.25/release/.25`} rows={2} />

</CommandEntry>
