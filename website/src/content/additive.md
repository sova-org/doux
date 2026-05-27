---
title: "Additive"
slug: "additive"
group: "synthesis"
order: 106
---

<script lang="ts">
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import CommandEntry from '$lib/components/CommandEntry.svelte';
</script>

The `add` source sums sine partials into a single tone. These parameters shape that spectrum: how many partials, how they roll off, how they are tuned, and the balance between odd and even harmonics.

<CommandEntry name="harmonics" type="number" min={0} max={1} default={0.5} mod>

Inharmonicity. At 0 the partials form a pure integer harmonic series. Higher values stretch each partial progressively sharp (ratio <code>i &#42; (1 + stretch &#42; (i − 1))</code>), giving bell-like and metallic spectra. Aliased as <code>harm</code>.

<CodeEditor code={`/sound/add/note/48/harmonics/0.6`} rows={2} />

<CodeEditor code={`/sound/add/note/48/harmonics/0~1:3/gate/4`} rows={2} />

</CommandEntry>

<CommandEntry name="timbre" type="number" min={0} max={1} default={0.5} mod>

Spectral tilt. The amplitude of partial <code>i</code> is roughly <code>i^(−3(1 − timbre))</code>. Low values are dark, rolling off steeply toward a near-sine tone; high values are bright, with a flatter, buzzier spectrum.

<CodeEditor code={`/sound/add/note/48/timbre/0.2`} rows={2} />

<CodeEditor code={`/sound/add/note/48/timbre/1`} rows={2} />

</CommandEntry>

<CommandEntry name="morph" type="number" min={0} max={1} default={0.5} mod>

Odd/even harmonic balance. At 0.5 all partials are at full level. Below 0.5 the even harmonics fade out (hollow, clarinet-like); above 0.5 the odd harmonics fade out.

<CodeEditor code={`/sound/add/note/48/morph/0.1`} rows={2} />

<CodeEditor code={`/sound/add/note/48/morph/0~1:2/gate/3`} rows={2} />

</CommandEntry>

<CommandEntry name="partials" type="number" min={1} max={32} default={32} mod>

Number of active harmonics for the `add` source. Fractional values smoothly crossfade the last partial. Lower values produce simpler timbres, higher values produce richer spectra.

<CodeEditor code={`/sound/add/note/48/partials/4`} rows={2} />

<CodeEditor code={`/sound/add/note/48/partials/1~32:3/gate/4`} rows={2} />

</CommandEntry>
