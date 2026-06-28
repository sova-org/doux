---
title: "Pitch Shifter"
slug: "pshift"
group: "effects"
order: 201.6
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Granular (delay-line) pitch shifter — transposes by **semitones**, preserving harmonic ratios. The musical counterpart to the inharmonic [frequency shifter](#fshift): octaves, fifths, sub-octaves, subtle detune thickening, or dive-bombs and risers when you modulate it. The characteristic granular warble grows past ~±7 semitones.

<CommandEntry name="pshift" aliases="psh" type="number" min={-24} max={24} default={0} unit="st" mod>

Transposition in semitones. Positive shifts up, negative down, 0 bypasses.

<CodeEditor code={`/sound/saw/freq/100/pshift/12`} rows={2} />

<CodeEditor code={`/sound/tri/freq/200/pshift/-12`} rows={2} />

<CodeEditor code={`/sound/saw/freq/100/pshift/0>-24:1/gate/2`} rows={2} />

</CommandEntry>

<CommandEntry name="pshiftwin" aliases="pwin" type="number" min={5} max={200} default={40} unit="ms" mod>

Grain window in ms — the character knob. Short windows (5-20 ms) are grainy and robotic with a faster warble; long windows (80-200 ms) are smoother but add latency. Only audible while `pshift` is non-zero.

<CodeEditor code={`/sound/saw/freq/100/pshift/7/pshiftwin/12`} rows={2} />

<CodeEditor code={`/sound/saw/freq/100/pshift/7/pshiftwin/120`} rows={2} />

</CommandEntry>
