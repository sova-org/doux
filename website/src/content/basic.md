---
title: "Basic"
slug: "basic"
group: "sources"
order: 0
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

These sources provide fundamental waveforms that can be combined and manipulated to create complex sounds. They are inspired by classic substractive synthesizers.

<CommandEntry name="sine" type="source">

Pure sine wave. The simplest waveform with no harmonics.

<CodeEditor code={`/sound/sine`} rows={2} />

<CodeEditor code={`/sound/sine/note/60`} rows={2} />

</CommandEntry>

<CommandEntry name="tri" type="source">

Triangle wave. The default source. Contains only odd harmonics with gentle rolloff.

<CodeEditor code={`/sound/tri`} rows={2} />

<CodeEditor code={`/sound/tri/note/60`} rows={2} />

</CommandEntry>

<CommandEntry name="saw" type="source">

Band-limited sawtooth wave. Rich in harmonics, bright and buzzy.

<CodeEditor code={`/sound/saw`} rows={2} />

<CodeEditor code={`/sound/saw/note/60`} rows={2} />

</CommandEntry>

<CommandEntry name="zaw" type="source">

Naive sawtooth with no anti-aliasing. Cheaper but more aliasing artifacts than saw.

<CodeEditor code={`/sound/zaw`} rows={2} />

<CodeEditor code={`/sound/zaw/note/60`} rows={2} />

</CommandEntry>

<CommandEntry name="pulse" type="source">

Band-limited pulse wave. Hollow sound with only odd harmonics. Use /pw to control pulse width.

<CodeEditor code={`/sound/pulse`} rows={2} />

<CodeEditor code={`/sound/pulse/pw/0.25`} rows={2} />

</CommandEntry>

<CommandEntry name="pulze" type="source">

Naive pulse with no anti-aliasing. Cheaper but more aliasing artifacts than pulse.

<CodeEditor code={`/sound/pulze`} rows={2} />

<CodeEditor code={`/sound/pulze/pw/0.25`} rows={2} />

</CommandEntry>

<CommandEntry name="white" type="source">

White noise. Equal energy at all frequencies.

<CodeEditor code={`/sound/white`} rows={2} />

<CodeEditor code={`/sound/white/lpf/2000`} rows={2} />

</CommandEntry>

<CommandEntry name="pink" type="source">

Pink noise (1/f). Equal energy per octave, more natural sounding.

<CodeEditor code={`/sound/pink`} rows={2} />

<CodeEditor code={`/sound/pink/lpf/4000`} rows={2} />

</CommandEntry>

<CommandEntry name="brown" type="source">

Brown/red noise (1/f^2). Deep rumbling, heavily weighted toward low frequencies.

<CodeEditor code={`/sound/brown`} rows={2} />

<CodeEditor code={`/sound/brown/hpf/100`} rows={2} />

</CommandEntry>
