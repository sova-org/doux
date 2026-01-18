---
title: "Filter Type"
slug: "ftype"
group: "effects"
order: 114
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

Controls the steepness of all filters. Higher dB/octave values create sharper transitions between passed and attenuated frequencies.

<CommandEntry name="ftype" type="enum" default="12db" values={["12db", "24db", "48db"]}>

Filter slope steepness. Higher dB/octave values create sharper cutoffs. Applies to all filter types (lowpass, highpass, bandpass).

<CodeEditor code={`/sound/pulse/freq/50/lpf/500/lpq/0.8/lpe/4/lpd/0.2/ftype/12db/d/.5\n\n/sound/pulse/freq/50/lpf/500/lpq/0.8/lpe/4/lpd/0.2/ftype/24db/time/1/d/.5\n\n/sound/pulse/freq/50/lpf/500/lpq/0.8/lpe/4/lpd/0.2/ftype/48db/time/2/d/.5`} rows={6} />

</CommandEntry>
