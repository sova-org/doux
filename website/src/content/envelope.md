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

The envelope parameters control the shape of the gain envelope over time. It uses a DAHDSR envelope with exponential curves:

- **Delay**: Waits before the attack begins. The signal stays at 0 during this phase.
- **Attack**: Ramps from 0 to full amplitude. Uses <code>x²</code> (slow start, fast finish).
- **Hold**: Holds at full amplitude before the decay begins.
- **Decay**: Falls from full amplitude to the sustain level. Uses <code>1-(1-x)²</code> (fast drop, slow finish).
- **Sustain**: Holds at a constant level while the note is held.
- **Release**: Falls from the sustain level to 0 when the note ends. Uses <code>1-(1-x)²</code> (fast drop, slow finish).

<CommandEntry name="envdelay" aliases="envdly" type="number" min={0} default={0} unit="s">

The duration (seconds) of the delay phase of the gain envelope. The signal stays silent during this time.

<CodeEditor code={`/envdelay/.2/attack/.1`} rows={2} />

<CodeEditor code={`/envdelay/.5/attack/.3`} rows={2} />

</CommandEntry>

<CommandEntry name="attack" type="number" min={0} default={0.003} unit="s">

The duration (seconds) of the attack phase of the gain envelope.

<CodeEditor code={`/attack/.1`} rows={2} />

<CodeEditor code={`/attack/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="hold" aliases="hld" type="number" min={0} default={0} unit="s">

The duration (seconds) of the hold phase of the gain envelope. The signal stays at full amplitude during this time.

<CodeEditor code={`/attack/.1/hold/.2`} rows={2} />

<CodeEditor code={`/attack/.05/hold/.5`} rows={2} />

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

<CodeEditor code={`/gate/.25/release/.25`} rows={2} />

</CommandEntry>
