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

Send-effect reverb with two algorithms.

<CommandEntry name="verb" type="number" min={0} max={1} default={0}>

Send level to the reverb bus.

<CodeEditor code={`/verb/0.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbtype" type="enum" default="dattorro" values={["dattorro", "fdn"]}>

<ul>
<li><strong>dattorro</strong> (or <code>plate</code>) — Plate reverb with input diffusers and stereo tank. Bright, metallic shimmer.</li>
<li><strong>fdn</strong> (or <code>hall</code>) — Hall reverb. Dense, smooth, diffuse tail.</li>
</ul>

<CodeEditor code={`/sound/saw/verb/.6/verbtype/plate/verbdecay/.7/verbdamp/.3/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/verb/.6/verbtype/fdn/verbdecay/.7/verbdamp/.3/d/.05`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdecay" type="number" min={0} max={1} default={0.5}>

Controls tail length.

<CodeEditor code={`/verb/.8/verbtype/plate/verbdecay/.9/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.8/verbtype/fdn/verbdecay/.9/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdamp" type="number" min={0} max={1} default={0.5}>

High-frequency absorption. Higher values darken the tail. Both algorithms use a one-pole lowpass in the feedback path.

<CodeEditor code={`/verb/.7/verbdamp/.7/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbpredelay" type="number" min={0} max={1} default={0}>

<ul>
<li><strong>Dattorro</strong> — Pre-delay before the diffusers (0–1 scales up to ~100ms).</li>
<li><strong>FDN</strong> — Modulation depth. Adds chorus-like movement to the tail.</li>
</ul>

<CodeEditor code={`/verb/.6/verbtype/plate/verbpredelay/.4/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.6/verbtype/fdn/verbpredelay/.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdiff" type="number" min={0} max={1} default={0.7}>

<ul>
<li><strong>Dattorro</strong> — Allpass diffusion amount. Higher values smear transients more.</li>
<li><strong>FDN</strong> — Room size. Scales all delay line lengths (0.2–1.5x).</li>
</ul>

<CodeEditor code={`/verb/.6/verbtype/plate/verbdiff/.9/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.6/verbtype/fdn/verbdiff/.4/duration/.1`} rows={2} />

</CommandEntry>
