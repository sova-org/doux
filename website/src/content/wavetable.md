---
title: 'Wavetable'
slug: 'wavetable'
group: 'sources'
order: 2
related: ["basic"]
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

You can read any audio sample as a wavetable oscillator instead of playing it back as a recording. <code>wtlen</code> is what selects that: put it on a sound and the sample is read as a row of single-cycle waveforms, looping one cycle at a time at whatever pitch you give it. Set it to <code>0</code> and the cycle length comes from the file. Use audio-rate modulation on <code>scan</code> to animate the position (e.g. <code>scan "0~1:2t"</code>).

<CommandEntry name="wtlen" type="number" default={0}>

Cycle length in samples, and the switch that selects wavetable playback. Set to 0 to read the length from the file: native builds look for the cycle length Serum-style exporters write into the WAV, and fall back to 2048 for a file whose length is an exact multiple of it. A file that declares nothing and divides into nothing is read as a single cycle. In the browser there is no file to inspect, so 0 means the whole buffer is one cycle and you should state the length yourself.

The length is quoted in the file's own samples. Resampling to the device rate is accounted for, so the same number stays correct at 44.1 kHz and 48 kHz.

<CodeEditor code={`/sound/wt_korg/wtlen/0/note/48/decay/2`} rows={2} />
<CodeEditor code={`/sound/wt_korg/wtlen/2048/note/48/decay/2`} rows={2} />

</CommandEntry>

<CommandEntry name="scan" type="number" min={0} max={1} default={0} mod>

Wavetable position. For multi-cycle wavetables, morphs between adjacent waveforms. This is an ordinary parameter: on its own it does not select wavetable playback, so a sample carrying <code>scan</code> but no <code>wtlen</code> still plays back as a recording.

<CodeEditor code={`/sound/wt_korg/wtlen/0/scan/0/note/48/decay/2`} rows={2} />
<CodeEditor code={`/sound/wt_korg/wtlen/0/scan/0.5/note/48/decay/2`} rows={2} />

<CodeEditor code={`/sound/wt_korg/wtlen/0/scan/0~1:2t/note/48/decay/2/gate/3`} rows={2} />

</CommandEntry>
