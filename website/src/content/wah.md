---
title: "Auto-Wah"
slug: "wah"
group: "effects"
order: 208
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Envelope-follower auto-wah: a resonant bandpass whose cutoff rides up with the input's amplitude envelope (a "touch wah"). The follower tracks the live signal inside the DSP, so the sweep responds to dynamics rather than a fixed LFO.

<CommandEntry name="wah" type="number" min={0} max={1} default={0}>

Dry/wet mix (0 = bypass, 1 = full wet).

<CodeEditor code={`/sound/saw/freq/110/wah/0.9`} rows={2} />

<CodeEditor code={`/sound/pulse/freq/80/wah/0.8/decay/1.5/gate/1.5`} rows={2} />

</CommandEntry>

<CommandEntry name="wahpeak" type="number" min={0} max={1} default={0.5}>

Resonance / peak sharpness of the bandpass.

<CodeEditor code={`/sound/saw/freq/110/wah/0.9/wahpeak/0.9`} rows={2} />

</CommandEntry>

<CommandEntry name="wahsens" type="number" min={0} max={1} default={0.5}>

Envelope sensitivity — how far the cutoff sweeps up as the signal gets louder.

<CodeEditor code={`/sound/saw/freq/110/wah/0.9/wahsens/0.8`} rows={2} />

</CommandEntry>

<CommandEntry name="wahmanual" type="number" min={100} max={4000} default={400} unit="Hz">

Base cutoff in Hz — the resting position the envelope sweeps up from.

<CodeEditor code={`/sound/saw/freq/110/wah/0.9/wahmanual/250/wahsens/0.7`} rows={2} />

</CommandEntry>
