---
title: "Modal Resonator"
slug: "modal"
group: "effects"
order: 116
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Eight tuned bandpasses standing in for the modes of a struck body. Whatever the voice produces is the exciter — a noise burst plucks it, a click strikes it, a saw bows it — and the bank rings at `modalfreq` and decays. It is a filter, not a voice: silence in, silence out.

The resonator is a per-voice insert sitting just after the filters, so its ring lives and dies with the voice's own envelope. For a tail that outlasts the note, give the voice a long `sustain` or `release` — or use the orbit's `comb`, which is a send and keeps ringing after the voice has ended.

The same bank is available inside a patch as the `modal` UGen: `noise 220 3 0 0.5 modal`.

<CommandEntry name="modal" type="number" min={0} max={1} default={0}>

Dry/wet mix (0 = bypass, 1 = full wet). At full wet you hear only the ring.

<CodeEditor code={`/sound/white/attack/0.001/decay/0.02/modal/1/modalfreq/220/sustain/2`} rows={2} />

<CodeEditor code={`/sound/saw/freq/55/modal/0.6/modalfreq/440/gate/1`} rows={2} />

</CommandEntry>

<CommandEntry name="modalfreq" type="number" min={20} max={20000} default={220} unit="Hz">

The fundamental — the pitch mode 1 rings at. Every other mode sits at a ratio above it.

<CodeEditor code={`/sound/white/attack/0.001/decay/0.02/modal/1/modalfreq/880/sustain/2`} rows={2} />

</CommandEntry>

<CommandEntry name="modaldecay" type="number" min={0.05} max={20} default={2} unit="s">

Ring time of mode 1, in seconds. A longer decay buys ring time, not loudness: the strike stays at the same level and simply hangs on.

<CodeEditor code={`/sound/white/attack/0.001/decay/0.02/modal/1/modaldecay/12/sustain/4`} rows={2} />

</CommandEntry>

<CommandEntry name="modalstruct" type="number" min={0} max={1} default={0}>

The partial ratios, morphing continuously: 0 is a string (the harmonic series), 0.5 a bar, 1 a bell. One knob from woody to metallic to clangorous.

<CodeEditor code={`/sound/white/attack/0.001/decay/0.02/modal/1/modalstruct/0.5/sustain/2`} rows={2} />

<CodeEditor code={`/sound/white/attack/0.001/decay/0.02/modal/1/modalstruct/1/modalfreq/660/sustain/2`} rows={2} />

</CommandEntry>

<CommandEntry name="modalbright" type="number" min={0} max={1} default={0.5}>

How long the upper modes ring relative to mode 1. At 0 they die almost at once and only the fundamental is left; at 1 they hang on with it and the body stays bright the whole way down.

<CodeEditor code={`/sound/white/attack/0.001/decay/0.02/modal/1/modalbright/1/modalstruct/0.7/sustain/3`} rows={2} />

</CommandEntry>
