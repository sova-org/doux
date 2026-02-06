---
title: "Feedback"
slug: "feedback"
group: "effects"
order: 202
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Orbit feedback delay. Sends voice signal to the orbit bus where it is re-injected with controllable delay time and damping.

<CommandEntry name="feedback" type="number" min={0} max={1} default={0}>

Feedback delay send level and re-injection amount. 0 = bypassed. Internally clamped to 0.99 to prevent runaway.

<CodeEditor code={`/sound/saw/freq/100/feedback/0.7/fbtime/120/decay/0.5`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/80/feedback/0.7/fbtime/250/decay/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="fbtime" type="number" min={0.1} max={680} default={10} unit="ms">

Feedback delay time in milliseconds. Short values produce metallic resonances, longer values give slapback echoes.

<CodeEditor code={`/sound/saw/freq/100/feedback/0.7/fbtime/500/decay/0.5`} rows={2} />

<CodeEditor code={`/sound/white/freq/200/feedback/0.8/fbtime/100/decay/0.1`} rows={2} />

</CommandEntry>

<CommandEntry name="fbdamp" type="number" min={0} max={1} default={0}>

High-frequency damping in the feedback path. Higher values roll off treble on each iteration, producing warmer repeats.

<CodeEditor code={`/sound/pulze/freq/100/feedback/1.0/fbtime/8/fbdamp/0.5/decay/0.1`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/150/feedback/0.8/fbtime/50/fbdamp/0.8/decay/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="fblfo" type="number" min={0} max={100} default={0} unit="Hz">

Feedback delay time LFO rate in Hz. Modulates the delay time to produce wobbling, warping delay tails. 0 = no modulation.

<CodeEditor code={`/sound/saw/freq/100/feedback/0.7/fbtime/120/fblfo/2/fblfodepth/0.3/decay/0.5`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/80/feedback/0.8/fbtime/250/fblfo/0.5/decay/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="fblfodepth" type="number" min={0} max={1} default={0.5}>

Depth of the feedback delay time LFO modulation. Controls how much the delay time varies around its center value.

<CodeEditor code={`/sound/saw/freq/100/feedback/0.7/fbtime/120/fblfo/3/fblfodepth/0.8/decay/0.5`} rows={2} />

</CommandEntry>

<CommandEntry name="fblfoshape" type="string" default="sine">

Waveform shape of the feedback delay time LFO. Options: sine, tri, saw, square, ramp.

<CodeEditor code={`/sound/saw/freq/100/feedback/0.7/fbtime/120/fblfo/2/fblfoshape/tri/decay/0.5`} rows={2} />

</CommandEntry>
