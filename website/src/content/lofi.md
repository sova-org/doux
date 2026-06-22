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

<CommandEntry name="coarse" type="number" min={1} default={1} mod>

Sample rate reduction. Holds each sample for <code>n</code> samples, creating stair-stepping and aliasing artifacts.

<CodeEditor code={`/freq/130^8000:0.003:0.5/coarse/8`} rows={2} />

<CodeEditor code={`/sound/saw/freq/100/coarse/1>16:2/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="crush" type="number" min={1} max={16} default={16} unit="bits" mod>

Bit depth reduction. Quantizes amplitude to <code>2^(bits-1)</code> levels, creating stepping distortion.

<CodeEditor code={`/freq/130^8000:0.003:0.5/crush/4`} rows={2} />

<CodeEditor code={`/sound/saw/crush/16>2:1.5/freq/100/decay/1.5/gate/2`} rows={2} />

</CommandEntry>

<CommandEntry name="fold" type="number" min={0} max={1} default={0} mod>

Reflective triangle wavefold (Buchla/Serge-style). At 0, near-passthrough. At 0.25, subtle harmonics. At 0.5, rich harmonics. At 1, extreme density.

<CodeEditor code={`/sound/sine/fold/.8`} rows={2} />

<CodeEditor code={`/sound/sine/fold/0~1:1/freq/80/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="foldmode" type="enum" default="triangle" values={["triangle", "sine", "wrap"]}>

Fold shape. The default reproduces the original triangle fold exactly.

<ul>
<li><strong>triangle</strong> — Reflective triangle fold (the default). Rich odd harmonics.</li>
<li><strong>sine</strong> — Sine fold. Rounder, fewer high harmonics.</li>
<li><strong>wrap</strong> — Sawtooth wrap. Harsher, more digital.</li>
</ul>

<CodeEditor code={`/sound/sine/fold/.8/foldmode/sine`} rows={2} />

</CommandEntry>

<CommandEntry name="wrap" type="number" min={1} default={1} mod>

Wrap distortion. Signal wraps around creating harsh digital artifacts.

<CodeEditor code={`/sound/tri/wrap/2`} rows={2} />

</CommandEntry>

<CommandEntry name="distort" type="number" min={0} default={0} mod>

Soft-clipping waveshaper using <code>(1+k)&#42;x / (1+k&#42;|x|)</code> where <code>k = e^amount - 1</code>. Higher values add harmonic saturation.

<CodeEditor code={`/sound/sine/distort/4`} rows={2} />

<CodeEditor code={`/sound/sine/distort/0>8:2/freq/80/decay/2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="distortvol" type="number" min={0} default={1}>

Output gain applied after distortion to compensate for increased level.

<CodeEditor code={`/sound/sine/distort/4/distortvol/.5`} rows={2} />

</CommandEntry>

<CommandEntry name="distortmode" type="enum" default="soft" values={["soft", "tanh", "arctan", "hardclip", "parabolic", "sinarctan"]}>

Saturator curve. The default reproduces the original soft clip exactly; the others are antialiased (ADAA) shapers driven by the <code>distort</code> amount.

<ul>
<li><strong>soft</strong> — Original <code>(1+k)&#42;x / (1+k&#42;|x|)</code> soft clip (the default).</li>
<li><strong>tanh</strong> — Smooth hyperbolic-tangent saturation.</li>
<li><strong>arctan</strong> — Gentler arctangent knee.</li>
<li><strong>hardclip</strong> — Hard digital clip.</li>
<li><strong>parabolic</strong> — Rounded parabolic clip, between soft and hard.</li>
<li><strong>sinarctan</strong> — <code>x / sqrt(1+x&#178;)</code> sigmoid, smooth and symmetric.</li>
</ul>

<CodeEditor code={`/sound/saw/distort/8/distortmode/tanh`} rows={2} />

</CommandEntry>

<CommandEntry name="distortasym" type="number" min={-1} max={1} default={0}>

Pre-shaper bias. Pushes the signal off-centre into the saturator for asymmetric clipping and even-harmonic colour (tube-ish). The induced DC offset is removed by the downstream DC blocker. At <code>0</code> every curve is unchanged.

<CodeEditor code={`/sound/saw/distort/8/distortasym/.4`} rows={2} />

</CommandEntry>
