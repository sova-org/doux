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
  "delay:DelayDsp"
)

for spec in "${specs[@]}"; do
  name="${spec%%:*}"; cn="${spec##*:}"
  gen="$out/${name}_gen.rs"
  faust -lang rust -cn "$cn" "dsp/$name.dsp" -o "$gen"
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

echo "regenerated $out/*_gen.rs"
