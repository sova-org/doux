use crate::effects::{
    CombParams, Compressor, DelayParams, FaustComb, FaustDelay, FaustFeedback, FaustJpVerb,
    FaustVitalRev, FeedbackParams, ReverbParams,
};
use crate::patch::VoicePatch;
use crate::types::{ReverbType, StereoFrame, CHANNELS, MAX_BLOCK};
use crate::voice::modulation::lcg;
use crate::voice::{ModChain, ParamMod};

const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF_SECS: f32 = 1.0;
pub const MAX_ORBIT_MODS: usize = 16;

/// Continuous orbit FX params addressable by an inline ModChain. Enum/index
/// params (delaytype, verbtype, comporbit) stay static-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OrbitParamId {
    Delay,
    Verb,
    Comb,
    Feedback,
    Comp,
    DelayTime,
    DelayFeedback,
    VerbDecay,
    VerbDamp,
    VerbPredelay,
    VerbDiff,
    VerbSize,
    VerbPrelow,
    VerbPrehigh,
    VerbLowcut,
    VerbHighcut,
    VerbLowgain,
    VerbHighgain,
    VerbChorus,
    VerbChorusFreq,
    CombFreq,
    CombFeedback,
    CombDamp,
    FbTime,
    FbDamp,
    FbCross,
    CompAttack,
    CompRelease,
    PatchLevel,
}

/// Per-orbit reusable block scratch. Every user fills `[..n]` before reading;
/// contents beyond the current fill are stale and never read. Persistent so
/// `process_block` does not splat `MAX_BLOCK`-sized stack arrays every chunk
/// (512+ wasted stores per orbit per chunk at the default n = 32).
struct OrbitScratch {
    /// Per-sample comb-freq trajectory: `apply_mods` fills it when a CombFreq
    /// chain is bound; otherwise the comb stage splats the static param.
    ctl_freq: [f32; MAX_BLOCK],
    /// Per-sample feedback-time trajectory (same contract, FbTime / fb stage).
    ctl_time: [f32; MAX_BLOCK],
    /// Mono FX-send staging (comb, per channel).
    send_mono: [f32; MAX_BLOCK],
    /// Stereo FX-send staging (feedback / delay / reverb, reused in sequence —
    /// each stage fully rewrites `[..n]` from the bus before its FX runs).
    send_stereo: [StereoFrame; MAX_BLOCK],
}

impl Default for OrbitScratch {
    fn default() -> Self {
        Self {
            ctl_freq: [0.0; MAX_BLOCK],
            ctl_time: [0.0; MAX_BLOCK],
            send_mono: [0.0; MAX_BLOCK],
            send_stereo: [[0.0; CHANNELS]; MAX_BLOCK],
        }
    }
}

// SuperDirt-style chain: voices accumulate into `bus`; each FX reads
// `bus * send_level`, adds its wet back into `bus`, in order. Order matters —
// later FX see the running signal including previous FX wet.
//
// Chain order: comb → fb → delay → verb → patch. Tonal/short → spatial/long.
// Reverb after delay so it captures the echoes (the load-bearing reason for
// chaining); the user's arf patch closes the chain so it can process the
// full mix including every native send's wet.
//
// Phase E: `bus` is block-rate. `clear_bus()` zeroes dirty buffers (flag-gated);
// `add_dry(frame, ch, v)` accumulates dry voice output per frame; `process_block(n)`
// runs the FX chain across `bus[..n]` via each FX's Phase-C block API. Sends are
// staged through small stack scratches so the block APIs (which overwrite their
// input) can return wet to be summed onto the bus.
pub struct Orbit {
    pub bus: Box<[StereoFrame; MAX_BLOCK]>,
    /// `bus` may hold nonzero data (written since the last clear). A cleared
    /// flag guarantees the FULL array is zero, so idle orbits skip the clear,
    /// the silence scans, and the final-mix pass entirely.
    pub bus_used: bool,
    /// FX input from `superpan` voices, kept OUT of `bus` so their dry never maps
    /// to the orbit's stereo pair. Merged into `bus` in `process_block` only when
    /// this orbit is room-routed.
    pub fx_send: Box<[StereoFrame; MAX_BLOCK]>,
    /// Wet-only FX result for the room, recovered by snapshot-difference.
    /// Never cleared: the final mix reads it only when `room_active`, and a
    /// room-active block always rewrites `fx_wet[..n]` before returning (the
    /// lone skipping path — the silence bypass — also drops `room_active`).
    pub fx_wet: Box<[StereoFrame; MAX_BLOCK]>,
    /// `fx_send` may hold nonzero data (same lazy-clear contract as `bus_used`).
    pub fx_send_used: bool,
    /// A `superpan` voice fed `fx_send` this chunk (per-chunk, reset in `clear_bus`).
    pub has_fx_send: bool,
    /// A pan voice accumulated dry into `bus` this chunk (per-chunk).
    pub has_pan_dry: bool,
    /// Latched room-routing: set when a superpan voice sends with no pan dry, held
    /// across chunks so the FX tail keeps flowing to the room after the source
    /// stops, released once the orbit goes silent.
    pub room_active: bool,
    pub delay: FaustDelay,
    pub delay_params: DelayParams,
    pub delay_level: f32,
    pub jpverb: FaustJpVerb,
    pub vital: FaustVitalRev,
    pub reverb_params: ReverbParams,
    pub verb_level: f32,
    pub comb: [FaustComb; CHANNELS],
    pub comb_params: CombParams,
    pub comb_level: f32,
    pub fb: FaustFeedback,
    pub fb_params: FeedbackParams,
    pub fb_level: f32,
    pub comp: Compressor,
    pub comp_orbit: usize,
    /// User arf effect (`patch/<name>`): a persistent Vm sticky on the orbit,
    /// swapped only by a `patch` event naming a different entry, returned to
    /// its pool on clear (`patch/off`) or engine drop.
    pub patch: Option<VoicePatch>,
    pub patch_level: f32,
    /// Set per block when the sticky `patch/` effect scrubbed a NaN/inf: its Vm
    /// state may be latched, so `gen_block` swaps in a fresh pooled Vm. Cleared
    /// at the patch stage's entry each `process_block`.
    pub patch_poisoned: bool,
    pub sr: f32,
    isr: f32,
    // === Param modulation (ticked once per block in `apply_mods`) ===
    param_mods: [(OrbitParamId, ParamMod); MAX_ORBIT_MODS],
    param_mod_count: u8,
    // Previous block's send level per FX, for the per-sample send-level ramp in
    // `process_block` (de-zippers a modulated send level — the staircase that
    // lives in the native pre-scale, outside the Faust DSP).
    prev_comb_level: f32,
    prev_fb_level: f32,
    prev_delay_level: f32,
    prev_patch_level: f32,
    seed: u32,
    silent_samples: u32,
    silence_holdoff: u32,
    scratch: Box<OrbitScratch>,
}

impl Orbit {
    pub fn new(sr: f32, index: usize) -> Self {
        let silence_holdoff = (sr * SILENCE_HOLDOFF_SECS) as u32;
        Self {
            bus: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            bus_used: false,
            fx_send: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            fx_wet: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            fx_send_used: false,
            has_fx_send: false,
            has_pan_dry: false,
            room_active: false,
            delay: FaustDelay::new(sr),
            delay_params: DelayParams::default(),
            delay_level: 0.0,
            jpverb: FaustJpVerb::new(sr),
            vital: FaustVitalRev::new(sr),
            reverb_params: ReverbParams::default(),
            verb_level: 0.0,
            comb: std::array::from_fn(|_| FaustComb::new(sr)),
            comb_params: CombParams::default(),
            comb_level: 0.0,
            fb: FaustFeedback::new(sr),
            fb_params: FeedbackParams::default(),
            fb_level: 0.0,
            comp: Compressor::default(),
            comp_orbit: 0,
            // 1.0 so `patch/<name>` is audible without an explicit
            // patchlevel; sticky thereafter like every orbit param.
            patch: None,
            patch_level: 1.0,
            patch_poisoned: false,
            sr,
            isr: 1.0 / sr,
            param_mods: [(OrbitParamId::Delay, ParamMod::default()); MAX_ORBIT_MODS],
            param_mod_count: 0,
            prev_comb_level: 0.0,
            prev_fb_level: 0.0,
            prev_delay_level: 0.0,
            prev_patch_level: 1.0,
            seed: lcg(index as u32 + 1),
            silent_samples: silence_holdoff + 1,
            silence_holdoff,
            scratch: Box::new(OrbitScratch::default()),
        }
    }

    /// Install a ModChain on an orbit param, replacing any existing chain on
    /// the same param. Slew resolves its start from the current value, same
    /// as the voice path. Envelope chains trigger on install: re-sending the
    /// param each event retriggers, giving per-event FX envelopes. `gate` is
    /// the envelope's total time before release in seconds (0.0 = hold).
    pub fn set_mod(&mut self, id: OrbitParamId, chain: ModChain, gate: f32) {
        let chain = if let ModChain::Slew {
            target,
            freq,
            curve,
        } = chain
        {
            ModChain::Transition {
                start: self.read_param(id),
                target,
                freq,
                curve,
                looping: false,
            }
        } else {
            chain
        };
        for i in 0..self.param_mod_count as usize {
            if self.param_mods[i].0 == id {
                self.param_mods[i].1 = ParamMod::new(chain, self.seed);
                self.param_mods[i].1.trigger(gate);
                self.seed = lcg(self.seed);
                return;
            }
        }
        if (self.param_mod_count as usize) < MAX_ORBIT_MODS {
            let i = self.param_mod_count as usize;
            self.param_mods[i] = (id, ParamMod::new(chain, self.seed));
            self.param_mods[i].1.trigger(gate);
            self.seed = lcg(self.seed);
            self.param_mod_count += 1;
        }
    }

    /// Remove any active ModChain targeting `id` (swap-remove, no alloc).
    /// The param keeps its last written value.
    pub fn clear_mod(&mut self, id: OrbitParamId) {
        let mut i = 0;
        while i < self.param_mod_count as usize {
            if self.param_mods[i].0 == id {
                self.param_mod_count -= 1;
                self.param_mods.swap(i, self.param_mod_count as usize);
            } else {
                i += 1;
            }
        }
    }

    fn read_param(&self, id: OrbitParamId) -> f32 {
        match id {
            OrbitParamId::Delay => self.delay_level,
            OrbitParamId::Verb => self.verb_level,
            OrbitParamId::Comb => self.comb_level,
            OrbitParamId::Feedback => self.fb_level,
            OrbitParamId::Comp => self.comp.params.amount,
            OrbitParamId::DelayTime => self.delay_params.time,
            OrbitParamId::DelayFeedback => self.delay_params.feedback,
            OrbitParamId::VerbDecay => self.reverb_params.decay,
            OrbitParamId::VerbDamp => self.reverb_params.damp,
            OrbitParamId::VerbPredelay => self.reverb_params.predelay,
            OrbitParamId::VerbDiff => self.reverb_params.diff,
            OrbitParamId::VerbSize => self.reverb_params.size,
            OrbitParamId::VerbPrelow => self.reverb_params.prelow,
            OrbitParamId::VerbPrehigh => self.reverb_params.prehigh,
            OrbitParamId::VerbLowcut => self.reverb_params.lowcut,
            OrbitParamId::VerbHighcut => self.reverb_params.highcut,
            OrbitParamId::VerbLowgain => self.reverb_params.lowgain,
            OrbitParamId::VerbHighgain => self.reverb_params.highgain,
            OrbitParamId::VerbChorus => self.reverb_params.chorus,
            OrbitParamId::VerbChorusFreq => self.reverb_params.chorus_freq,
            OrbitParamId::CombFreq => self.comb_params.freq,
            OrbitParamId::CombFeedback => self.comb_params.feedback,
            OrbitParamId::CombDamp => self.comb_params.damp,
            OrbitParamId::FbTime => self.fb_params.time_ms,
            OrbitParamId::FbDamp => self.fb_params.damp,
            OrbitParamId::FbCross => self.fb_params.cross,
            OrbitParamId::CompAttack => self.comp.params.attack,
            OrbitParamId::CompRelease => self.comp.params.release,
            OrbitParamId::PatchLevel => self.patch_level,
        }
    }

    /// Send levels clamp at 0 (the `set_pos!` contract in lib.rs); the rest
    /// write raw, matching the static `set!` path — each FX clamps at
    /// consumption (e.g. delay feedback 0..0.95, time to MAX_DELAY_SAMPLES).
    fn write_param(&mut self, id: OrbitParamId, v: f32) {
        match id {
            OrbitParamId::Delay => self.delay_level = v.max(0.0),
            OrbitParamId::Verb => self.verb_level = v.max(0.0),
            OrbitParamId::Comb => self.comb_level = v.max(0.0),
            OrbitParamId::Feedback => self.fb_level = v.max(0.0),
            OrbitParamId::Comp => self.comp.params.amount = v.max(0.0),
            OrbitParamId::DelayTime => self.delay_params.time = v,
            OrbitParamId::DelayFeedback => self.delay_params.feedback = v,
            OrbitParamId::VerbDecay => self.reverb_params.decay = v,
            OrbitParamId::VerbDamp => self.reverb_params.damp = v,
            OrbitParamId::VerbPredelay => self.reverb_params.predelay = v,
            OrbitParamId::VerbDiff => self.reverb_params.diff = v,
            OrbitParamId::VerbSize => self.reverb_params.size = v,
            OrbitParamId::VerbPrelow => self.reverb_params.prelow = v,
            OrbitParamId::VerbPrehigh => self.reverb_params.prehigh = v,
            OrbitParamId::VerbLowcut => self.reverb_params.lowcut = v,
            OrbitParamId::VerbHighcut => self.reverb_params.highcut = v,
            OrbitParamId::VerbLowgain => self.reverb_params.lowgain = v,
            OrbitParamId::VerbHighgain => self.reverb_params.highgain = v,
            OrbitParamId::VerbChorus => self.reverb_params.chorus = v,
            OrbitParamId::VerbChorusFreq => self.reverb_params.chorus_freq = v,
            OrbitParamId::CombFreq => self.comb_params.freq = v,
            OrbitParamId::CombFeedback => self.comb_params.feedback = v,
            OrbitParamId::CombDamp => self.comb_params.damp = v,
            OrbitParamId::FbTime => self.fb_params.time_ms = v,
            OrbitParamId::FbDamp => self.fb_params.damp = v,
            OrbitParamId::FbCross => self.fb_params.cross = v,
            OrbitParamId::CompAttack => self.comp.params.attack = v,
            OrbitParamId::CompRelease => self.comp.params.release = v,
            OrbitParamId::PatchLevel => self.patch_level = v.max(0.0),
        }
    }

    /// Advance all active param mods by `n` samples. Block-rate params write a
    /// single value (`tick_block`); the two audio-rate params (CombFreq, FbTime)
    /// fill `scratch.ctl_freq` / `scratch.ctl_time` (`tick_into`) consumed as a
    /// Faust input, and still sync their field to the block's final value. Runs
    /// before the silence bypass so a modulated send level can wake the orbit
    /// and chains keep time through silent stretches. Returns which of the two
    /// control trajectories were written; the consuming FX stage splats the
    /// static param over `[..n]` when its flag is false.
    fn apply_mods(&mut self, n: usize) -> (bool, bool) {
        let isr = self.isr;
        let mut freq_traj = false;
        let mut time_traj = false;
        for i in 0..self.param_mod_count as usize {
            let id = self.param_mods[i].0;
            match id {
                OrbitParamId::CombFreq => {
                    self.comb_params.freq = self.param_mods[i]
                        .1
                        .tick_into(isr, &mut self.scratch.ctl_freq[..n]);
                    freq_traj = true;
                }
                OrbitParamId::FbTime => {
                    self.fb_params.time_ms = self.param_mods[i]
                        .1
                        .tick_into(isr, &mut self.scratch.ctl_time[..n]);
                    time_traj = true;
                }
                _ => {
                    let v = self.param_mods[i].1.tick_block(isr, n);
                    self.write_param(id, v);
                }
            }
        }
        (freq_traj, time_traj)
    }

    #[inline]
    pub fn clear_bus(&mut self) {
        // Full-array clears (not `..n`): a clean flag must mean "all zero"
        // even when a later chunk is larger than this one. Only dirty orbits
        // pay them, and they are straight memsets. `fx_wet` is never cleared —
        // see the field invariant.
        if self.bus_used {
            *self.bus = [[0.0; CHANNELS]; MAX_BLOCK];
            self.bus_used = false;
        }
        if self.fx_send_used {
            *self.fx_send = [[0.0; CHANNELS]; MAX_BLOCK];
            self.fx_send_used = false;
        }
        // Per-chunk routing flags. `room_active` latches across chunks and is
        // released in `process_block` on silence, so it is NOT reset here.
        self.has_fx_send = false;
        self.has_pan_dry = false;
    }

    /// Zero every orbit-chain FX tail in place (delay lines, reverb tanks, comb
    /// lines, feedback lines, compressor env). Called once when the orbit crosses
    /// its silence holdoff (state is < −140 dB then, so this is inaudible) and by
    /// `Engine::panic`. Flushes denormal or sub-threshold frozen state to true
    /// zero on every target and shrinks the wasm per-sample denormal window to the
    /// bounded 1 s holdoff. `instance_clear` keeps coefficients — no re-init.
    pub fn clear_fx_state(&mut self) {
        self.delay.reset_in_place();
        self.jpverb.reset_in_place();
        self.vital.reset_in_place();
        for c in self.comb.iter_mut() {
            c.reset_in_place();
        }
        self.fb.reset_in_place();
        self.comp.clear_env();
    }

    /// True when any orbit FX has a non-zero send level — gate for routing
    /// `superpan` voices into the FX path.
    #[inline]
    pub fn has_any_fx(&self) -> bool {
        self.comb_level > 0.0
            || self.fb_level > 0.0
            || self.delay_level > 0.0
            || self.verb_level > 0.0
            || (self.patch_level > 0.0 && self.patch.is_some())
    }

    #[inline]
    pub fn add_dry(&mut self, frame: usize, ch: usize, value: f32) {
        self.bus[frame][ch] += value;
        self.bus_used = true;
    }

    /// Block-rate driver for the orbit FX chain. Each enabled FX runs its
    /// Phase-C block API once across `bus[..n]`, staged through a stack
    /// scratch and summed back onto the bus.
    ///
    /// Silence accounting is block-aware (`to_do.md:302`): pre-scan for any
    /// non-zero input; if all zero AND past holdoff, bypass FX and add `n` to
    /// `silent_samples`; otherwise process, then sum-of-abs on `bus[..n]` and
    /// update `silent_samples` accordingly. At `dsp_block_size = 1` (n=1)
    /// this is bit-identical to the legacy per-sample `Orbit::process`.
    pub fn process_block(&mut self, n: usize) {
        debug_assert!(
            n <= MAX_BLOCK,
            "Orbit::process_block: n={n} > MAX_BLOCK={MAX_BLOCK}"
        );
        // Param mods first: the silence bypass below reads send levels, so a
        // modulated level must be current before `has_any_fx` is consulted.
        // The two audio-rate params (comb freq, feedback time) land in
        // `scratch.ctl_freq` / `scratch.ctl_time` when a ModChain is bound;
        // otherwise the consuming stage splats the static value over `[..n]`,
        // a constant signal identical to its old slider.
        let (freq_traj, time_traj) = self.apply_mods(n);

        // Room routing: latch on when a superpan voice sends with no pan dry; the
        // FX run on `bus + fx_send` and only the wet (recovered below) reaches the
        // room. The latch holds the tail to the room after the source stops.
        if self.has_fx_send && !self.has_pan_dry {
            self.room_active = true;
        }

        // FX-less, non-room orbit: the bus is untouched by this function, so
        // the pre-FX input scan and the post-FX energy scan would walk the
        // same data — do the silence accounting with a single pass. Decision-
        // equivalent to the general path below: sum-of-abs over the bus is
        // nonzero iff any sample is nonzero (nonnegative addition; subnormals
        // still sum positive without FTZ, NaN propagates into both compares).
        if !self.has_any_fx() && !self.room_active {
            let mut energy = 0.0_f32;
            if self.bus_used {
                for frame in self.bus.iter().take(n) {
                    energy += frame[0].abs() + frame[1].abs();
                }
            }
            if energy != 0.0 {
                self.silent_samples = 0;
            } else if self.silent_samples > self.silence_holdoff {
                self.silent_samples = self.silent_samples.saturating_add(n as u32);
                return;
            }
            if energy < SILENCE_THRESHOLD * n as f32 {
                self.silent_samples = self.silent_samples.saturating_add(n as u32);
            } else {
                self.silent_samples = 0;
            }
            return;
        }

        let dedicated = self.room_active;
        if dedicated {
            for f in 0..n {
                self.bus[f][0] += self.fx_send[f][0];
                self.bus[f][1] += self.fx_send[f][1];
            }
            // `has_fx_send` ⟺ a superpan voice wrote `fx_send` this chunk, so
            // the merge may have made the bus nonzero without any pan dry.
            if self.has_fx_send {
                self.bus_used = true;
            }
        }

        // Clean flag guarantees an all-zero bus: skip the scan, its result is known.
        let any_input =
            self.bus_used && self.bus.iter().take(n).any(|s| s[0] != 0.0 || s[1] != 0.0);

        if any_input {
            self.silent_samples = 0;
        } else if self.silent_samples > self.silence_holdoff {
            self.silent_samples = self.silent_samples.saturating_add(n as u32);
            self.room_active = false; // tail decayed; release room routing
            return;
        }

        // Past the bypass: the FX chain (or the dedicated merge) writes the bus
        // below, so it must be cleared and mixed from here on.
        self.bus_used = true;

        // Comb (per-channel mono resonator, shared params). The send level is
        // ramped per sample from the previous block's value (`lvl` below) so a
        // modulated comb level does not stair-step; constant level => step 0 =>
        // exact `frame * level` (unchanged steady state).
        if self.comb_level > 0.0 {
            let level = self.comb_level;
            let prev = self.prev_comb_level;
            let step = (level - prev) / n as f32;
            let params = self.comb_params;
            if !freq_traj {
                self.scratch.ctl_freq[..n].fill(params.freq);
            }
            for c in 0..CHANNELS {
                for (i, (slot, frame)) in self
                    .scratch
                    .send_mono
                    .iter_mut()
                    .take(n)
                    .zip(self.bus.iter().take(n))
                    .enumerate()
                {
                    *slot = frame[c] * (prev + step * (i as f32 + 1.0));
                }
                self.comb[c].process_block(
                    &mut self.scratch.send_mono[..n],
                    n,
                    &params,
                    &self.scratch.ctl_freq[..n],
                );
                for (frame, &wet) in self
                    .bus
                    .iter_mut()
                    .take(n)
                    .zip(self.scratch.send_mono.iter().take(n))
                {
                    frame[c] += wet;
                }
            }
            self.prev_comb_level = level;
        }

        // Feedback (stereo short delay with cross-channel + LFO). Input pre-scale
        // is ramped per sample (de-zippers a modulated send level); the same
        // block-level `level` is still passed as the re-injection coefficient,
        // which the DSP smooths internally (si.smooth on g_fb).
        if self.fb_level > 0.0 {
            let level = self.fb_level;
            let prev = self.prev_fb_level;
            let step = (level - prev) / n as f32;
            let p = self.fb_params;
            if !time_traj {
                self.scratch.ctl_time[..n].fill(p.time_ms);
            }
            for (i, (slot, frame)) in self
                .scratch
                .send_stereo
                .iter_mut()
                .take(n)
                .zip(self.bus.iter().take(n))
                .enumerate()
            {
                let lvl = prev + step * (i as f32 + 1.0);
                slot[0] = frame[0] * lvl;
                slot[1] = frame[1] * lvl;
            }
            self.fb.process_block(
                &mut self.scratch.send_stereo[..n],
                n,
                &p,
                level,
                &self.scratch.ctl_time[..n],
            );
            for (frame, wet) in self
                .bus
                .iter_mut()
                .take(n)
                .zip(self.scratch.send_stereo.iter().take(n))
            {
                frame[0] += wet[0];
                frame[1] += wet[1];
            }
            self.prev_fb_level = level;
        }

        // Delay (stereo). Send level ramped per sample (de-zippers a modulated
        // delay level); delay time is already si.smooth-ed inside the DSP.
        if self.delay_level > 0.0 {
            let level = self.delay_level;
            let prev = self.prev_delay_level;
            let step = (level - prev) / n as f32;
            let p = self.delay_params;
            for (i, (slot, frame)) in self
                .scratch
                .send_stereo
                .iter_mut()
                .take(n)
                .zip(self.bus.iter().take(n))
                .enumerate()
            {
                let lvl = prev + step * (i as f32 + 1.0);
                slot[0] = frame[0] * lvl;
                slot[1] = frame[1] * lvl;
            }
            self.delay
                .process_block(&mut self.scratch.send_stereo[..n], n, &p);
            self.prev_delay_level = level;
            for (frame, wet) in self
                .bus
                .iter_mut()
                .take(n)
                .zip(self.scratch.send_stereo.iter().take(n))
            {
                frame[0] += wet[0];
                frame[1] += wet[1];
            }
        }

        // Reverb — last so it captures delay echoes.
        if self.verb_level > 0.0 {
            let level = self.verb_level;
            let rp = &self.reverb_params;
            // Both reverbs are stereo (2-in/2-out) Faust effects run fully wet:
            // build a stereo send = bus * level, run the chosen reverb, and add
            // the wet back onto the bus.
            for (slot, frame) in self
                .scratch
                .send_stereo
                .iter_mut()
                .take(n)
                .zip(self.bus.iter().take(n))
            {
                slot[0] = frame[0] * level;
                slot[1] = frame[1] * level;
            }
            let send = &mut self.scratch.send_stereo[..n];
            match rp.verb_type {
                ReverbType::Cloud => self.jpverb.process_block(send, n, rp),
                ReverbType::Space => self.vital.process_block(send, n, rp),
            }
            for (frame, wet) in self
                .bus
                .iter_mut()
                .take(n)
                .zip(self.scratch.send_stereo.iter().take(n))
            {
                frame[0] += wet[0];
                frame[1] += wet[1];
            }
        }

        // arf patch (user effect) — closes the chain so it hears the full mix
        // including every native send's wet. Parallel send like the rest:
        // input = bus * level (ramped per sample from the previous block's
        // value), wet summed back onto the bus, dry always passes. An
        // effect's control plane carries only the transport lane (patch.rs
        // contract), latched per chunk; `frame_pos` only advances while the
        // orbit is awake — `now` is windowed, arf/src/vm.rs:106.
        self.patch_poisoned = false;
        if self.patch_level > 0.0 {
            if let Some(p) = self.patch.as_mut() {
                let level = self.patch_level;
                let prev = self.prev_patch_level;
                let step = (level - prev) / n as f32;
                let program = p.entry.program();
                let in_ch = program.in_channels();
                let width = program.audio_channels().min(CHANNELS);
                let mut bad = false;
                for (i, frame) in self.bus.iter_mut().take(n).enumerate() {
                    let lvl = prev + step * (i as f32 + 1.0);
                    // C3 input rule: stereo patch reads the bus pair, mono
                    // patch reads its downmix.
                    let input = if in_ch == 2 {
                        [frame[0] * lvl, frame[1] * lvl]
                    } else {
                        [(frame[0] + frame[1]) * 0.5 * lvl, 0.0]
                    };
                    let mut out = [0.0f32; CHANNELS];
                    p.vm.tick_frame(
                        program,
                        p.frame_pos,
                        &input[..in_ch],
                        &p.control[..program.control_len()],
                        &mut out[..width],
                    );
                    p.frame_pos += 1;
                    // Same scrub as run_arf_block: one non-finite sample
                    // would poison the master DC blocker. No 0.7 headroom —
                    // `{ 2 inputs out }` at patchlevel 1 must be unity.
                    bad |= crate::patch::scrub_non_finite(&mut out);
                    let (w0, w1) = (out[0], out[1]);
                    if width == 2 {
                        frame[0] += w0;
                        frame[1] += w1;
                    } else {
                        frame[0] += w0;
                        frame[1] += w0;
                    }
                }
                self.prev_patch_level = level;
                self.patch_poisoned = bad;
            }
        }

        // Recover wet-only for the room: bus now holds dry+wet, fx_send holds the
        // dry that was merged in, so the difference is the FX return. (When pan
        // dry shares a room-active orbit — misuse — it leaks here; documented.)
        if dedicated {
            for f in 0..n {
                self.fx_wet[f][0] = self.bus[f][0] - self.fx_send[f][0];
                self.fx_wet[f][1] = self.bus[f][1] - self.fx_send[f][1];
            }
        }

        // Post-block silence accounting. Sum-of-abs ≥ threshold·n keeps the
        // counter pinned to 0 while any non-trivial tail energy remains.
        let mut energy = 0.0_f32;
        for frame in self.bus.iter().take(n) {
            energy += frame[0].abs() + frame[1].abs();
        }
        if !energy.is_finite() {
            // Recovery hatch: a NaN reached a native Faust feedback path. It would
            // otherwise pin `silent_samples` to 0 forever (NaN fails both compares),
            // so flush the chain and reset the counter.
            self.clear_fx_state();
            self.silent_samples = 0;
        } else if energy < SILENCE_THRESHOLD * n as f32 {
            let old = self.silent_samples;
            let new = old.saturating_add(n as u32);
            self.silent_samples = new;
            // Crossing the holdoff: the tail is now inaudible (< 1e-7 for 1 s).
            // Flush denormal/frozen FX state to true zero, exactly once.
            if old <= self.silence_holdoff && new > self.silence_holdoff {
                self.clear_fx_state();
            }
        } else {
            self.silent_samples = 0;
        }
    }
}
