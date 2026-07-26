---
title: "Arf Patches"
slug: "patch"
group: "effects"
order: 211
related: ["reverb", "delay"]
---

<script lang="ts">
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

User-defined arf effect patches, applied per voice or per orbit. A patch is installed into the engine's registry by the host (in cagire: `dsp`, `defdsp`, `fx`, `deffx`); events then reference it by name. A patch that reads audio input (`in`, `n input`, `n inputs`) is an effect; a patch without input is a source, playable as `/sound/arf:name`. The two roles are exclusive.

Patch installation is not available in this playground yet, so the examples below are not interactive.

<CommandEntry name="patch" type="string">

Set the orbit's arf effect patch by name — a serial insert, running after the built-in effect chain so it processes their tails as well as the dry. Its output replaces the bus, so unlike `verb` or `delay` it can subtract: a filter really filters the orbit, a saturator really overdrives it. The dry is whatever `in` reads, mixed inside the patch or with `patchlevel`. The patch is sticky on the orbit until replaced, or cleared with the reserved name `off`. A mono patch reads the bus downmix and its output feeds both channels; a stereo patch (`2 inputs`) reads and returns the pair.

```
/sound/kick/patch/shimmer
/sound/kick/patch/off
```

</CommandEntry>

<CommandEntry name="patchlevel" type="number" min={0} max={1} default={1} mod>

Dry/wet mix for the orbit patch insert: 0 is the untouched bus, 1 is the patch alone. Clamped to that range, sticky like every orbit parameter. Takes modulation strings.

```
/sound/break/patch/shimmer/patchlevel/0.5
```

</CommandEntry>

<CommandEntry name="fx" type="string">

Insert an arf effect patch into the voice's own chain, after the built-in inserts and just before the envelope VCA. Serial: the patch output replaces the voice signal (the dry is whatever `in` reads — mix inside the patch). The insert never changes the voice's width; a stereo patch on a mono voice is downmixed. `fx/off` clears the slot; an event naming a missing patch, a source-role patch, or a patch whose instance pool is exhausted is dropped.

```
/sound/break/fx/crushed
/sound/break/fx/off
```

</CommandEntry>
