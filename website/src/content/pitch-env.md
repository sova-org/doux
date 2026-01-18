---
title: "Pitch Env"
slug: "pitch-env"
group: "synthesis"
order: 106
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

An ADSR envelope applied to pitch. The envelope runs with gate always on (no release phase during note). The frequency is multiplied by <code>2^(env &#42; penv / 12)</code>. When <code>psus = 1</code>, the envelope value is offset by -1 so sustained notes return to base pitch.

<CommandEntry name="penv" type="number" default={0} unit="semitones">

Pitch envelope depth in semitones. Positive values sweep up, negative values sweep down.

<CodeEditor code={`/penv/24/pdec/.2`} rows={2} />

</CommandEntry>

<CommandEntry name="patt" type="number" min={0} default={0.001} unit="s">

Attack time. Duration to reach peak pitch offset.

<CodeEditor code={`/patt/.2`} rows={2} />

</CommandEntry>

<CommandEntry name="pdec" type="number" min={0} default={0} unit="s">

Decay time. Duration to fall from peak to sustain level.

<CodeEditor code={`/pdec/.2`} rows={2} />

</CommandEntry>

<CommandEntry name="psus" type="number" min={0} max={1} default={1}>

Sustain level. At 1.0, the envelope returns to base pitch after decay.

</CommandEntry>

<CommandEntry name="prel" type="number" min={0} default={0.005} unit="s">

Release time. Not typically audible since pitch envelope gate stays on.

</CommandEntry>
