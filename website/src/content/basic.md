---
title: "Basic"
slug: "basic"
group: "sources"
order: 0
related: ["oscillator", "wavetable"]
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
  import ParamTable from '$lib/components/ParamTable.svelte';
</script>

These sources provide fundamental waveforms that can be combined and manipulated to create complex sounds. They are inspired by classic substractive synthesizers.

<CommandEntry name="sine" type="source">

Pure sine wave. The simplest waveform with no harmonics.

<CodeEditor code={`/sound/sine`} rows={2} />

<CodeEditor code={`/sound/sine/note/60`} rows={2} />

</CommandEntry>

<CommandEntry name="tri" type="source">

Triangle wave. Contains only odd harmonics with gentle rolloff.

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

<CommandEntry name="osc" type="source">

Morphing oscillator. Sweeps through sine, triangle, saw, and square as `wave` goes from 0 to 1. The `wave` parameter is modulable.

<CodeEditor code={`/sound/osc/note/60`} rows={2} />

<CodeEditor code={`/sound/osc/note/48/wave/0~1:2/decay/4/gate/5`} rows={2} />

</CommandEntry>

<CommandEntry name="pluck" type="source">

Karplus-Strong plucked string. A noise burst rings through a tuned, damped feedback loop. The string is retuned every sample, so vibrato and pitch modulation bend it continuously. Aliases: `ks`, `string`.

Every source names its three tone-shaping parameters this way: semantic names per source, while the generic `timbre`, `harmonics`, and `morph` work on all of them — useful when retargeting a sounding voice without restating `sound`.

<ParamTable params={[
  { name: "bright", alias: "timbre", range: "0–1", default: "0.5", mod: true, description: "brightness (loop damping: 0 = dark thud, 1 = bright ring)" },
  { name: "ring", alias: "harmonics, harm", range: "0–1", default: "0.5", mod: true, description: "sustain (0 = dead, 0.5 ≈ half-second tail, 1 = drone)" },
  { name: "excite", alias: "morph", range: "0–1", default: "0.5", mod: true, description: "excitation color (0 = soft dark pluck, 1 = snappy attack)" },
]} />

<CodeEditor code={`/sound/pluck/note/60`} rows={2} />

<CodeEditor code={`/sound/pluck/note/48/bright/0.8/ring/0.9/gate/3`} rows={2} />

<CodeEditor code={`/sound/pluck/note/52/ring/1/vib/5/vibmod/0.3/gate/4`} rows={2} />

</CommandEntry>
