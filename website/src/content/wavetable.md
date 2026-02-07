---
title: 'Wavetable'
slug: 'wavetable'
group: 'sources'
order: 2
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

You can use any audio sample as a wavetable oscillator. The sample is played at the specified pitch. The cycle length for each wavetable can be specified with <code>wtlen</code>. Use audio-rate modulation on <code>scan</code> to animate the wavetable position (e.g. <code>scan "0~1:2t"</code>).

<CommandEntry name="scan" type="number" min={0} max={1} default={0} mod>

Wavetable position. For multi-cycle wavetables, morphs between adjacent waveforms.

<CodeEditor code={`/sound/wt_korg/scan/0/note/48/decay/2/wtlen/1024`} rows={2} />
<CodeEditor code={`/sound/wt_korg/scan/0.5/note/48/decay/2/wtlen/1024`} rows={2} />

<CodeEditor code={`/sound/wt_korg/scan/0~1:2t/note/48/decay/2/dur/3/wtlen/1024`} rows={2} />

</CommandEntry>

<CommandEntry name="wtlen" type="number" default={0}>

Cycle length in samples. Set to 0 to use entire sample as one cycle. Common values: 256, 512, 1024, 2048 (Serum standard).

<CodeEditor code={`/sound/wt_korg/scan/0.5/note/48/decay/2/wtlen/1024`} rows={2} />

</CommandEntry>
