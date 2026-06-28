---
title: "Frequency Shifter"
slug: "fshift"
group: "effects"
order: 201.3
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Single-sideband frequency shifter — moves every partial up or down by a fixed number of **Hz**. Unlike a pitch shift it does not preserve harmonic ratios, so the spectrum turns inharmonic: small shifts phase and detune, larger shifts ring-modulate into metallic, clangorous "barber-pole" textures. Built from an analytic signal (Hilbert pair) heterodyned by a quadrature oscillator.

<CommandEntry name="fshift" aliases="fsh" type="number" min={-2000} max={2000} default={0} unit="Hz" mod>

Shift amount in Hz. Positive shifts up, negative shifts down, 0 bypasses — the sign selects the sideband.

<CodeEditor code={`/sound/saw/freq/100/fshift/60`} rows={2} />

<CodeEditor code={`/sound/tri/freq/200/fshift/-250`} rows={2} />

<CodeEditor code={`/sound/saw/freq/80/fshift/-300~300:4/gate/4`} rows={2} />

</CommandEntry>
