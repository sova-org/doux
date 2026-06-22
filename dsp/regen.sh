#!/usr/bin/env bash
# Regenerate the committed Rust DSP from the Faust sources.
#
# Requires the `faust` compiler (https://faust.grame.fr) on PATH. Only needed
# when a .dsp changes — normal `cargo build` uses the committed *_gen.rs and
# does NOT need faust installed.
set -euo pipefail
cd "$(dirname "$0")/.."
out=src/effects/faust_dsp

# dsp file stem -> generated struct name
specs=(
  "crush:CrushDsp"
  "fold:FoldDsp"
  "svf:SvfDsp"
  "coarse:CoarseDsp"
  "wrap:WrapDsp"
  "distort:DistortDsp"
  "tilt:TiltDsp"
  "eq:EqDsp"
  "wah:WahDsp"
  "vinyl:VinylDsp"
  "phaser:PhaserDsp"
  "chorus:ChorusDsp"
  "flanger:FlangerDsp"
  "smear:SmearDsp"
  "haas:HaasDsp"
  "ladder:LadderDsp"
  "svf24:Svf24Dsp"
  "vital_rev:VitalRevDsp"
  "jpverb:JpverbDsp"
  "comb:CombDsp"
  "feedback:FeedbackDsp"
  "delay_standard:DelayStandardDsp"
  "delay_pingpong:DelayPingpongDsp"
  "delay_tape:DelayTapeDsp"
  "delay_multitap:DelayMultitapDsp"
)

# DSPs with feedback loops / long decaying tails get a software denormal flush
# (-ftz 1, fabs-based): wasm32 has no hardware FTZ (enable_flush_to_zero is a
# no-op there), so their state can settle into denormals and spike CPU on old
# cores. Memoryless effects keep the default -ftz 0.
ftz_stems=" comb feedback delay_standard delay_pingpong delay_tape delay_multitap smear jpverb vital_rev "

for spec in "${specs[@]}"; do
  name="${spec%%:*}"; cn="${spec##*:}"
  gen="$out/${name}_gen.rs"
  ftz=0; [[ "$ftz_stems" == *" $name "* ]] && ftz=1
  faust -lang rust -ftz "$ftz" -cn "$cn" "dsp/$name.dsp" -o "$gen"
  # 1. Drop the `default-boxed` derive: the feature is never enabled and the
  #    unknown cfg value trips a check-cfg warning module-level allow can't catch.
  sed -i '' '/cfg_attr(feature = "default-boxed"/d' "$gen"
  # 2. Replace the libm FFI (`rintf`/`remainderf` — the only C-math symbols
  #    Faust's Rust backend emits; everything else uses Rust methods) with pure
  #    Rust std math. This removes the `#[link(name = "m")]` that breaks the
  #    wasm32-unknown-unknown link (no libm), and is portable across every OS.
  perl -i -pe '
    s/^\s*#\[cfg_attr\(not\(target_os = "windows"\), link\(name = "m"\)\)\]\s*$//;
    s{unsafe \{ ffi::remainderf\(from, to\) \}}{from - to * (from / to).round_ties_even()};
    s{unsafe \{ ffi::rintf\(val\) \}}{val.round_ties_even()};
  ' "$gen"
  # 3. Delete the now-dead `mod ffi { unsafe extern "C" { .. } }` block. Step 2
  #    rewrote its only callers (the `*_f32` helpers) to pure Rust, so the FFI
  #    declarations are unreferenced — dropping them removes the last
  #    `unsafe extern "C"` from the generated code.
  perl -i -ne 'print unless /^mod ffi \{/../^\}$/' "$gen"
done

# Guard: the libm/ffi stripping above is keyed to this faust version's output.
# If a future .dsp emits a libm primitive (or faust changes its codegen) and the
# strip misses it, the wasm32 link breaks silently with an unresolved symbol.
# Fail loudly here instead. Pinned: faust 2.81.2.
if grep -lE 'extern "C"|link\(name|ffi::' "$out"/*_gen.rs >/dev/null 2>&1; then
  echo "ERROR: libm/ffi leak in generated code — regen.sh stripping drifted from this faust version" >&2
  exit 1
fi

echo "regenerated $out/*_gen.rs"
