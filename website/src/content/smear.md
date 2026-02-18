---
title: "Smear"
slug: "smear"
group: "effects"
order: 207
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Allpass chain — 12 cascaded first-order allpass filters that smear transients into laser chirps and metallic sweeps. Zero buffers, pure phase manipulation. Allpass filters only shift phase, so the effect is inaudible at static frequencies — modulate `smearfreq` to hear it.

<CommandEntry name="smear" type="number" min={0} max={1} default={0} mod>

Wet/dry mix (0 = bypass, 1 = full wet). Controls the blend between dry input and the allpass-smeared signal.

<CodeEditor code={`/sound/saw/freq/100/smear/0.5/smearfreq/200~4000:2/dur/3`} rows={2} />

<CodeEditor code={`/sound/tri/freq/200/smear/0~1:2/smearfreq/300~3000:2/dur/3`} rows={2} />

</CommandEntry>

<CommandEntry name="smearfreq" type="number" min={20} default={1000} unit="Hz" mod>

Break frequency of the allpass chain. Lower values produce longer, more dramatic chirps. Higher values affect only the highest partials. Sweep it to hear the smearing.

<CodeEditor code={`/sound/saw/freq/100/smear/0.5/smearfreq/200~4000:2/dur/3`} rows={2} />

<CodeEditor code={`/sound/saw/freq/100/smear/0.8/smearfreq/sin:80/dur/3`} rows={2} />

</CommandEntry>

<CommandEntry name="smearfb" type="number" min={0} max={0.95} default={0} mod>

Feedback amount for resonance. Wraps the allpass output back to the input, creating metallic resonances and self-oscillation at high values.

<CodeEditor code={`/sound/saw/freq/100/smear/0.8/smearfreq/200~4000:2/smearfb/0.8/dur/3`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/80/smear/0.5/smearfreq/300~2000:3/smearfb/0~0.9:3/dur/4`} rows={2} />

</CommandEntry>
