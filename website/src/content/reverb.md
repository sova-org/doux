---
title: "Reverb"
slug: "reverb"
group: "effects"
order: 204
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Dattorro plate reverb with 4 input diffusers and a cross-fed stereo tank.

<CommandEntry name="verb" type="number" min={0} max={1} default={0}>

Send level to the reverb bus.

<CodeEditor code={`/verb/0.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdecay" type="number" min={0} max={1} default={0.5}>

Tank feedback amount (clamped to 0.99 max). Controls tail length.

<CodeEditor code={`/verb/0.8/verbdecay/0.9/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdamp" type="number" min={0} max={1} default={0.5}>

One-pole lowpass in the tank feedback path. Higher values darken the tail.

<CodeEditor code={`/verb/0.7/verbdamp/0.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbpredelay" type="number" min={0} max={1} default={0}>

Delay before the diffusers (0-1 of max ~100ms). Creates space before reverb onset.

<CodeEditor code={`/verb/0.6/verbpredelay/0.3/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdiff" type="number" min={0} max={1} default={0.7}>

Allpass coefficients in both input and tank diffusers. Higher values smear transients.

<CodeEditor code={`/verb/0.7/verbdiff/0.9/duration/.1`} rows={2} />

</CommandEntry>
