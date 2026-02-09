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

Send-effect reverb with two algorithms. The default is a 16-line modulated FDN based on the Vital synth reverb. The second is a Dattorro plate reverb.

<CommandEntry name="verb" type="number" min={0} max={1} default={0} mod>

Send level to the reverb bus.

<CodeEditor code={`/verb/0.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbtype" type="enum" default="vital" values={["vital", "dattorro"]}>

<ul>
<li><strong>vital</strong> — 16-line modulated FDN reverb. Dense, lush tail with per-line decay and chorus modulation.</li>
<li><strong>dattorro</strong> (or <code>plate</code>) — Plate reverb with input diffusers and stereo tank. Bright, metallic shimmer.</li>
</ul>

<CodeEditor code={`/sound/saw/verb/.6/verbdecay/.7/verbdamp/.3/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/verb/.6/verbtype/plate/verbdecay/.7/verbdamp/.3/d/.05`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdecay" type="number" min={0} max={1} default={0.75}>

Controls tail length. On the vital reverb this maps exponentially from 0.1 to 100 seconds of RT60.

<CodeEditor code={`/verb/.8/verbdecay/.3/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.8/verbdecay/.95/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdamp" type="number" min={0} max={1} default={0.95}>

High-frequency absorption. Higher values darken the tail.

<CodeEditor code={`/verb/.7/verbdamp/.2/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.7/verbdamp/.9/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbpredelay" type="number" min={0} max={1} default={0.1}>

Pre-delay before the reverb tank (0-1 maps to 0-300ms).

<CodeEditor code={`/verb/.6/verbpredelay/.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdiff" type="number" min={0} max={1} default={0.7}>

Room size / diffusion. Scales all delay line lengths exponentially (1/8x to 2x).

<ul>
<li><strong>Vital</strong> — Exponential size multiplier. Low values give small, tight spaces; high values give large halls.</li>
<li><strong>Dattorro</strong> — Allpass diffusion amount. Higher values smear transients more.</li>
</ul>

<CodeEditor code={`/verb/.6/verbdiff/.1/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.6/verbdiff/.9/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbchorus" type="number" min={0} max={1} default={0.3}>

Chorus modulation depth (vital only). Adds movement to the reverb tail by modulating delay line read positions.

<CodeEditor code={`/verb/.6/verbchorus/0/duration/.1`} rows={2} />

<CodeEditor code={`/verb/.6/verbchorus/.8/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbchorusfreq" type="number" min={0} max={1} default={0.2}>

Chorus LFO rate (vital only). Maps exponentially up to 16 Hz. Aliases: <code>vchorusfreq</code>.

<CodeEditor code={`/verb/.6/verbchorus/.5/verbchorusfreq/.6/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbprelow" type="number" min={0} max={1} default={0.2}>

Pre-filter highpass cutoff (vital only). Removes low frequencies before they enter the reverb tank. Maps to 27-11175 Hz via MIDI key scaling.

<CodeEditor code={`/verb/.6/verbprelow/.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbprehigh" type="number" min={0} max={1} default={0.8}>

Pre-filter lowpass cutoff (vital only). Removes high frequencies before they enter the reverb tank.

<CodeEditor code={`/verb/.6/verbprehigh/.4/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verblowcut" type="number" min={0} max={1} default={0.5}>

Feedback path low-shelf cutoff frequency (vital only). Controls where the low-shelf filter acts in the feedback loop.

<CodeEditor code={`/verb/.6/verblowcut/.3/verblowgain/.2/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbhighcut" type="number" min={0} max={1} default={0.7}>

Feedback path high-shelf cutoff frequency (vital only). Controls where the high-shelf filter acts in the feedback loop.

<CodeEditor code={`/verb/.6/verbhighcut/.4/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verblowgain" type="number" min={0} max={1} default={0.4}>

Low-shelf gain in the feedback path (vital only). 0 = full cut (-24dB), 1 = unity. Lower values thin out bass in the reverb tail.

<CodeEditor code={`/verb/.6/verblowgain/.1/duration/.1`} rows={2} />

</CommandEntry>
