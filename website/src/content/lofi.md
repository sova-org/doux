---
title: "Lo-Fi"
slug: "lofi"
group: "effects"
order: 205
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Sample rate reduction, bit crushing, and waveshaping distortion.

<CommandEntry name="coarse" type="number" min={1} default={1}>

Sample rate reduction. Holds each sample for <code>n</code> samples, creating stair-stepping and aliasing artifacts.

<CodeEditor code={`/penv/36/pdec/.5/coarse/8`} rows={2} />

</CommandEntry>

<CommandEntry name="crush" type="number" min={1} max={16} default={16} unit="bits">

Bit depth reduction. Quantizes amplitude to <code>2^(bits-1)</code> levels, creating stepping distortion.

<CodeEditor code={`/penv/36/pdec/.5/crush/4`} rows={2} />

</CommandEntry>

<CommandEntry name="fold" type="number" min={1} default={1}>

Sine-based wavefold (Serge-style). At 1, near-passthrough. At 2, one fold per peak. At 4, two folds.

<CodeEditor code={`/sound/sine/fold/3`} rows={2} />

</CommandEntry>

<CommandEntry name="wrap" type="number" min={1} default={1}>

Wrap distortion. Signal wraps around creating harsh digital artifacts.

<CodeEditor code={`/sound/tri/wrap/2`} rows={2} />

</CommandEntry>

<CommandEntry name="distort" type="number" min={0} default={0}>

Soft-clipping waveshaper using <code>(1+k)&#42;x / (1+k&#42;|x|)</code> where <code>k = e^amount - 1</code>. Higher values add harmonic saturation.

<CodeEditor code={`/sound/sine/distort/4`} rows={2} />

</CommandEntry>

<CommandEntry name="distortvol" type="number" min={0} default={1}>

Output gain applied after distortion to compensate for increased level.

<CodeEditor code={`/sound/sine/distort/4/distortvol/.5`} rows={2} />

</CommandEntry>
