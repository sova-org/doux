---
title: "Comb Filter"
slug: "comb"
group: "effects"
order: 113
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Send effect with feedback comb filter. Creates pitched resonance, metallic timbres, and Karplus-Strong plucked sounds. Tail persists after voice ends.

<CommandEntry name="comb" type="number" min={0} max={1} default={0} mod>

Send amount to comb filter.

<CodeEditor code={`/sound/white/comb/1/combfreq/110/decay/.5/sustain/0`} rows={2} />

Noise into a tuned comb creates plucked string sounds (Karplus-Strong).

</CommandEntry>

<CommandEntry name="combfreq" type="number" min={20} max={20000} default={220} unit="Hz">

Resonant frequency. All voices share the same orbit comb.

<CodeEditor code={`/sound/saw/comb/0.5/combfreq/880/decay/.5/sustain/0`} rows={2} />

</CommandEntry>

<CommandEntry name="combfeedback" type="number" min={0} max={0.99} default={0.9}>

Feedback amount. Higher values create longer resonance.

<CodeEditor code={`/sound/white/comb/1/combfeedback/0.99/combfreq/220/decay/.5/sustain/0`} rows={2} />

</CommandEntry>

<CommandEntry name="combdamp" type="number" min={0} max={1} default={0.1}>

High-frequency damping. Higher values darken the sound over time.

<CodeEditor code={`/sound/white/comb/1/combfeedback/0.95/combdamp/0.4/combfreq/220/decay/.5/sustain/0`} rows={2} />

</CommandEntry>
