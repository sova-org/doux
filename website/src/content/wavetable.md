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

You can use any audio sample as a wavetable oscillator. The sample is played at the specified pitch with an optional LFO modulation over the scanned position. The cycle length for each wavetable can be specified with <code>wtlen</code>.

<CommandEntry name="scan" type="number" min={0} max={1} default={0}>

Wavetable position. For multi-cycle wavetables, morphs between adjacent waveforms.

<CodeEditor code={`/sound/wt_korg/scan/0/note/48/decay/2/wtlen/1024`} rows={2} />
<CodeEditor code={`/sound/wt_korg/scan/0.5/note/48/decay/2/wtlen/1024`} rows={2} />

</CommandEntry>

<CommandEntry name="wtlen" type="number" default={0}>

Cycle length in samples. Set to 0 to use entire sample as one cycle. Common values: 256, 512, 1024, 2048 (Serum standard).

<CodeEditor code={`/sound/wt_korg/scan/0.5/note/48/decay/2/wtlen/1024`} rows={2} />

</CommandEntry>

<CommandEntry name="scanlfo" type="number" min={0} default={0} unit="Hz">

Scan LFO rate. Set to 0 to disable.

<CodeEditor code={`/sound/wt_korg/scan/0.0/scanlfo/0.2/scandepth/0.4/note/48/decay/2/wtlen/1024`} rows={2} />

</CommandEntry>

<CommandEntry name="scandepth" type="number" min={0} max={1} default={0}>

Scan LFO depth. Controls how much the LFO affects the scan position.

<CodeEditor code={`/sound/wt_korg/scan/0.3/scanlfo/0.5/scandepth/0.6/note/36/decay/2/wtlen/1024`} rows={2} />

</CommandEntry>

<CommandEntry name="scanshape" type="enum" default="sine" values={["sine", "tri", "saw", "square", "sh"]}>

Scan LFO waveform.

<ul>
  <li><strong>sine</strong> — Smooth cyclic sweep</li>
  <li><strong>tri</strong> — Linear up/down sweep</li>
  <li><strong>saw</strong> — Ramp up, snap down</li>
  <li><strong>square</strong> — Alternates between two positions</li>
  <li><strong>sh</strong> — Sample & hold (random steps)</li>
</ul>

<CodeEditor code={`/sound/wt_korg/scan/0.5/scanlfo/0.3/scandepth/0.5/scanshape/tri/note/48/decay/2/wtlen/1024`} rows={2} />

</CommandEntry>
