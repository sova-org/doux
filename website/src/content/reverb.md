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

Send-effect reverb with two algorithms: space (default) and plate.

<CommandEntry name="verb" type="number" min={0} max={1} default={0} mod>

Send level to the reverb bus.

<CodeEditor code={`/verb/1/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbtype" type="enum" default="space" values={["space", "plate"]}>

<ul>
<li><strong>space</strong> — Lush and dense with chorus modulation.</li>
<li><strong>plate</strong> — Bright and metallic.</li>
</ul>

<CodeEditor code={`/sound/saw/verb/1.2/verbdecay/.7/verbdamp/.3/d/.05`} rows={2} />

<CodeEditor code={`/sound/saw/verb/1.2/verbtype/plate/verbdecay/.7/verbdamp/.3/d/.05`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdecay" type="number" min={0} max={1} default={0.75}>

Controls how long the reverb tail rings out.

<CodeEditor code={`/verb/1/verbdecay/.3/duration/.1`} rows={2} />

<CodeEditor code={`/verb/1/verbdecay/.95/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdamp" type="number" min={0} max={1} default={0.95}>

Higher values darken the reverb tail.

<CodeEditor code={`/verb/1/verbdamp/.2/duration/.1`} rows={2} />

<CodeEditor code={`/verb/1/verbdamp/.9/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbpredelay" type="number" min={0} max={1} default={0.1}>

Gap before the reverb starts.

<CodeEditor code={`/verb/1/verbpredelay/.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbdiff" type="number" min={0} max={1} default={0.7}>

Room size. Low values give small tight spaces, high values give large halls.

<CodeEditor code={`/verb/1/verbdiff/.1/duration/.1`} rows={2} />

<CodeEditor code={`/verb/1/verbdiff/.9/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbchorus" type="number" min={0} max={1} default={0.3}>

Adds movement to the reverb tail (space only).

<CodeEditor code={`/verb/1/verbchorus/0/duration/.1`} rows={2} />

<CodeEditor code={`/verb/1/verbchorus/.8/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbchorusfreq" type="number" min={0} max={1} default={0.2}>

Speed of the chorus modulation (space only).

<CodeEditor code={`/verb/1/verbchorus/.5/verbchorusfreq/.6/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbprelow" type="number" min={0} max={1} default={0.2}>

Cuts low frequencies before they enter the reverb (space only).

<CodeEditor code={`/verb/1/verbprelow/.5/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbprehigh" type="number" min={0} max={1} default={0.8}>

Cuts high frequencies before they enter the reverb (space only).

<CodeEditor code={`/verb/1/verbprehigh/.4/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verblowcut" type="number" min={0} max={1} default={0.5}>

Where the low-frequency shaping kicks in inside the reverb (space only).

<CodeEditor code={`/verb/1/verblowcut/.3/verblowgain/.2/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verbhighcut" type="number" min={0} max={1} default={0.7}>

Where the high-frequency shaping kicks in inside the reverb (space only).

<CodeEditor code={`/verb/1/verbhighcut/.4/duration/.1`} rows={2} />

</CommandEntry>

<CommandEntry name="verblowgain" type="number" min={0} max={1} default={0.4}>

How much bass survives in the reverb tail (space only). Lower values thin it out.

<CodeEditor code={`/verb/1/verblowgain/.1/duration/.1`} rows={2} />

</CommandEntry>
