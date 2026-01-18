---
title: "Delay"
slug: "delay"
group: "effects"
order: 203
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Stereo delay line with feedback (max 1 second at 48kHz, clamped to 0.95 feedback).

<CommandEntry name="delay" type="number" min={0} max={1} default={0}>

Send level to the delay bus.

<CodeEditor code={`/delay/.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="delayfeedback" type="number" min={0} max={1} default={0.5}>

Feedback amount (clamped to 0.95 max). Output is fed back into input.

<CodeEditor code={`/delay/.5/delayfeedback/.8/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="delaytime" type="number" min={0} default={0.25} unit="s">

Delay time in seconds (max ~1s at 48kHz).

<CodeEditor code={`/delay/.5/delaytime/.08/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="delaytype" type="enum" default="standard" values={["standard", "pingpong", "tape", "multitap"]}>

<ul>
<li><strong>standard</strong> — Clean digital. Precise repeats.</li>
<li><strong>pingpong</strong> — Mono in, bounces L→R→L→R.</li>
<li><strong>tape</strong> — Each repeat darker. Analog warmth.</li>
<li><strong>multitap</strong> — 4 taps. Feedback 0=straight, 1=triplet, between=swing.</li>
</ul>

<CodeEditor code={`/sound/saw/delay/.6/dtype/std/delaytime/.15/delayfeedback/.7/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/delay/.7/dtype/pp/delaytime/.12/delayfeedback/.8/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/delay/.6/dtype/tape/delaytime/.2/delayfeedback/.9/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/delay/.7/dtype/multi/delaytime/.3/delayfeedback/0/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/delay/.7/dtype/multi/delaytime/.3/delayfeedback/1/d/.05`} rows={2} />

</CommandEntry>
