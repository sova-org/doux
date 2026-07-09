use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use soundfont::raw::GeneratorType;
use soundfont::SoundFont2;

use crate::sampling::SampleData;
use crate::types::midi2freq;

/// One flattened preset→instrument→zone, resolved off the audio thread into a
/// voice-ready entry. The sample PCM is owned here (via `Arc<SampleData>`), so a
/// note-on never touches the global sample registry — the whole bank is one
/// self-contained, atomically-published structure.
#[derive(Clone)]
struct ZoneEntry {
    program: u16,
    bank: u16,
    key_lo: u8,
    key_hi: u8,
    vel_lo: u8,
    vel_hi: u8,
    data: Arc<SampleData>,
    root_freq: f32,
    /// Native sample rate / device rate. Folded into playback speed so samples
    /// are stored at native rate (no up-front resample) and the cursor resamples
    /// at playback time, exactly like FluidSynth/RustySynth.
    sr_ratio: f32,
    loop_start: f32,
    loop_end: f32,
    looping: bool,
    /// SF2 sample mode 3: loop while held, then play the tail after release.
    loop_until_release: bool,
    /// Linear playback gain from the SF2 initialAttenuation generator (EMU-scaled).
    attenuation: f32,
    pan: f32,
    /// Base filter cutoff in absolute cents (before the per-note velocity term).
    filter_fc_cents: f32,
    filter_q: f32,
    scale_tuning: f32,
    /// Vibrato LFO rate in Hz (0 if no vibrato generator present).
    vib_rate: f32,
    /// Vibrato depth in semitones (0 = no vibrato).
    vib_depth: f32,
    exclusive_class: u8,
    delay: f32,
    attack: f32,
    /// Raw volume-envelope hold/decay in timecents plus their key-scaling
    /// coefficients; resolved to seconds at note-on (keynum is known there).
    hold_tc: i16,
    decay_tc: i16,
    keynum_to_hold: i16,
    keynum_to_decay: i16,
    sustain: f32,
    release: f32,
}

/// A resolved zone for one note-on. Owns a cheap `Arc` clone of the sample;
/// the velocity-dependent terms (amplitude, cutoff) are applied by the caller.
pub struct GmZone {
    pub data: Arc<SampleData>,
    pub root_freq: f32,
    pub sr_ratio: f32,
    pub loop_start: f32,
    pub loop_end: f32,
    pub looping: bool,
    pub loop_until_release: bool,
    pub attenuation: f32,
    pub pan: f32,
    pub filter_fc_cents: f32,
    pub filter_q: f32,
    pub scale_tuning: f32,
    pub vib_rate: f32,
    pub vib_depth: f32,
    pub exclusive_class: u8,
    pub delay: f32,
    pub attack: f32,
    pub hold: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

/// Contiguous run of zones sharing one `(program, bank)`. Lets a note-on binary
/// search to its program then scan only that program's zones — no full scan.
struct Bucket {
    program: u16,
    bank: u16,
    start: u32,
    len: u32,
}

pub struct GmBank {
    zones: Vec<ZoneEntry>,
    buckets: Vec<Bucket>,
}

impl GmBank {
    fn new(mut zones: Vec<ZoneEntry>) -> Self {
        zones.sort_by_key(|z| (z.program, z.bank));
        let mut buckets: Vec<Bucket> = Vec::new();
        for (i, z) in zones.iter().enumerate() {
            match buckets.last_mut() {
                Some(b) if b.program == z.program && b.bank == z.bank => b.len += 1,
                _ => buckets.push(Bucket {
                    program: z.program,
                    bank: z.bank,
                    start: i as u32,
                    len: 1,
                }),
            }
        }
        Self { zones, buckets }
    }

    /// Resolve a `(program, bank, note, velocity)` to the first matching zone.
    /// RT-safe: a binary search over buckets plus a short scan, no allocation.
    pub fn find(&self, program: u16, bank: u16, note: u8, vel: u8) -> Option<GmZone> {
        let bi = self
            .buckets
            .binary_search_by(|b| (b.program, b.bank).cmp(&(program, bank)))
            .ok()?;
        let b = &self.buckets[bi];
        let lo = b.start as usize;
        let hi = lo + b.len as usize;
        let z = self.zones[lo..hi]
            .iter()
            .find(|z| note >= z.key_lo && note <= z.key_hi && vel >= z.vel_lo && vel <= z.vel_hi)?;

        // Key-scale hold/decay (SF2 gens 39/40): tc' = tc + (60 - key) * coeff.
        let kn = 60i32 - note as i32;
        let hold = timecents_to_secs(z.hold_tc as i32 + kn * z.keynum_to_hold as i32);
        let decay = timecents_to_secs(z.decay_tc as i32 + kn * z.keynum_to_decay as i32);

        Some(GmZone {
            data: Arc::clone(&z.data),
            root_freq: z.root_freq,
            sr_ratio: z.sr_ratio,
            loop_start: z.loop_start,
            loop_end: z.loop_end,
            looping: z.looping,
            loop_until_release: z.loop_until_release,
            attenuation: z.attenuation,
            pan: z.pan,
            filter_fc_cents: z.filter_fc_cents,
            filter_q: z.filter_q,
            scale_tuning: z.scale_tuning,
            vib_rate: z.vib_rate,
            vib_depth: z.vib_depth,
            exclusive_class: z.exclusive_class,
            delay: z.delay,
            attack: z.attack,
            hold,
            decay,
            sustain: z.sustain,
            release: z.release,
        })
    }

    pub fn preset_count(&self) -> usize {
        self.buckets.len()
    }
}

pub fn resolve_gm_program(s: &str) -> Option<(u16, u16)> {
    if let Ok(n) = s.parse::<u16>() {
        return if n < 128 { Some((n, 0)) } else { None };
    }
    // Case-insensitive alias match without allocating: lowercase into a stack
    // buffer (GM alias names are short). Runs off-RT at parse time and on the
    // audio thread at note-on, so it must not touch the heap.
    let bytes = s.as_bytes();
    let mut buf = [0u8; 32];
    if bytes.len() > buf.len() {
        return None;
    }
    for (d, &b) in buf.iter_mut().zip(bytes) {
        *d = b.to_ascii_lowercase();
    }
    let lower = std::str::from_utf8(&buf[..bytes.len()]).ok()?;
    match lower {
        "drums" | "drum" | "percussion" => Some((0, 128)),
        "piano" | "grandpiano" => Some((0, 0)),
        "brightpiano" => Some((1, 0)),
        "epiano" | "electricpiano" => Some((4, 0)),
        "rhodes" => Some((4, 0)),
        "harpsichord" => Some((6, 0)),
        "clavinet" | "clav" => Some((7, 0)),
        "celesta" => Some((8, 0)),
        "glockenspiel" | "glock" => Some((9, 0)),
        "musicbox" => Some((10, 0)),
        "vibraphone" | "vibes" => Some((11, 0)),
        "marimba" => Some((12, 0)),
        "xylophone" | "xylo" => Some((13, 0)),
        "bells" | "tubularbells" => Some((14, 0)),
        "organ" => Some((16, 0)),
        "churchorgan" => Some((19, 0)),
        "accordion" => Some((21, 0)),
        "harmonica" => Some((22, 0)),
        "guitar" | "nylon" | "nylonguitar" => Some((24, 0)),
        "steelguitar" | "steel" => Some((25, 0)),
        "jazzguitar" => Some((26, 0)),
        "cleangt" | "clean" => Some((27, 0)),
        "overdrive" | "overdriven" => Some((29, 0)),
        "distgt" | "distortionguitar" => Some((30, 0)),
        "bass" | "fingerbass" => Some((33, 0)),
        "pickbass" => Some((34, 0)),
        "fretless" => Some((35, 0)),
        "slapbass" | "slap" => Some((36, 0)),
        "synthbass" => Some((38, 0)),
        "violin" => Some((40, 0)),
        "viola" => Some((41, 0)),
        "cello" => Some((42, 0)),
        "contrabass" => Some((43, 0)),
        "pizzicato" | "pizz" => Some((45, 0)),
        "harp" => Some((46, 0)),
        "timpani" => Some((47, 0)),
        "strings" | "ensemble" => Some((48, 0)),
        "slowstrings" => Some((49, 0)),
        "choir" => Some((52, 0)),
        "trumpet" => Some((56, 0)),
        "trombone" => Some((57, 0)),
        "tuba" => Some((58, 0)),
        "horn" | "frenchhorn" => Some((60, 0)),
        "brass" => Some((61, 0)),
        "sopranosax" => Some((64, 0)),
        "altosax" | "alto" => Some((65, 0)),
        "tenorsax" | "tenor" => Some((66, 0)),
        "barisax" | "bari" => Some((67, 0)),
        "oboe" => Some((68, 0)),
        "bassoon" => Some((70, 0)),
        "clarinet" => Some((71, 0)),
        "piccolo" => Some((72, 0)),
        "flute" => Some((73, 0)),
        "recorder" => Some((74, 0)),
        "panflute" | "pan" => Some((75, 0)),
        "whistle" => Some((79, 0)),
        "ocarina" => Some((80, 0)),
        "lead" | "squarelead" => Some((81, 0)),
        "sawlead" | "sawsynth" => Some((82, 0)),
        "pad" | "newage" => Some((89, 0)),
        "warmpad" | "warm" => Some((90, 0)),
        "polysynth" => Some((91, 0)),
        "sitar" => Some((104, 0)),
        "banjo" => Some((105, 0)),
        "kalimba" => Some((108, 0)),
        "steeldrum" => Some((114, 0)),
        _ => None,
    }
}

/// Returns all named GM preset mappings for documentation/completion.
/// Each entry: (canonical_name, aliases, GM program number).
pub fn gm_preset_docs() -> Vec<GmPresetDoc> {
    // Group aliases by (program, bank)
    let entries: &[(&str, u16, u16)] = &[
        ("piano", 0, 0),
        ("grandpiano", 0, 0),
        ("brightpiano", 1, 0),
        ("epiano", 4, 0),
        ("electricpiano", 4, 0),
        ("rhodes", 4, 0),
        ("harpsichord", 6, 0),
        ("clavinet", 7, 0),
        ("clav", 7, 0),
        ("celesta", 8, 0),
        ("glockenspiel", 9, 0),
        ("glock", 9, 0),
        ("musicbox", 10, 0),
        ("vibraphone", 11, 0),
        ("vibes", 11, 0),
        ("marimba", 12, 0),
        ("xylophone", 13, 0),
        ("xylo", 13, 0),
        ("bells", 14, 0),
        ("tubularbells", 14, 0),
        ("organ", 16, 0),
        ("churchorgan", 19, 0),
        ("accordion", 21, 0),
        ("harmonica", 22, 0),
        ("guitar", 24, 0),
        ("nylon", 24, 0),
        ("nylonguitar", 24, 0),
        ("steelguitar", 25, 0),
        ("steel", 25, 0),
        ("jazzguitar", 26, 0),
        ("cleangt", 27, 0),
        ("clean", 27, 0),
        ("overdrive", 29, 0),
        ("overdriven", 29, 0),
        ("distgt", 30, 0),
        ("distortionguitar", 30, 0),
        ("bass", 33, 0),
        ("fingerbass", 33, 0),
        ("pickbass", 34, 0),
        ("fretless", 35, 0),
        ("slapbass", 36, 0),
        ("slap", 36, 0),
        ("synthbass", 38, 0),
        ("violin", 40, 0),
        ("viola", 41, 0),
        ("cello", 42, 0),
        ("contrabass", 43, 0),
        ("pizzicato", 45, 0),
        ("pizz", 45, 0),
        ("harp", 46, 0),
        ("timpani", 47, 0),
        ("strings", 48, 0),
        ("ensemble", 48, 0),
        ("slowstrings", 49, 0),
        ("choir", 52, 0),
        ("trumpet", 56, 0),
        ("trombone", 57, 0),
        ("tuba", 58, 0),
        ("horn", 60, 0),
        ("frenchhorn", 60, 0),
        ("brass", 61, 0),
        ("sopranosax", 64, 0),
        ("altosax", 65, 0),
        ("alto", 65, 0),
        ("tenorsax", 66, 0),
        ("tenor", 66, 0),
        ("barisax", 67, 0),
        ("bari", 67, 0),
        ("oboe", 68, 0),
        ("bassoon", 70, 0),
        ("clarinet", 71, 0),
        ("piccolo", 72, 0),
        ("flute", 73, 0),
        ("recorder", 74, 0),
        ("panflute", 75, 0),
        ("pan", 75, 0),
        ("whistle", 79, 0),
        ("ocarina", 80, 0),
        ("lead", 81, 0),
        ("squarelead", 81, 0),
        ("sawlead", 82, 0),
        ("sawsynth", 82, 0),
        ("pad", 89, 0),
        ("newage", 89, 0),
        ("warmpad", 90, 0),
        ("warm", 90, 0),
        ("polysynth", 91, 0),
        ("sitar", 104, 0),
        ("banjo", 105, 0),
        ("kalimba", 108, 0),
        ("steeldrum", 114, 0),
        ("drums", 0, 128),
        ("drum", 0, 128),
        ("percussion", 0, 128),
    ];

    // Group by (program, bank) to find canonical name + aliases
    let mut grouped: std::collections::BTreeMap<(u16, u16), Vec<&str>> =
        std::collections::BTreeMap::new();
    for &(name, program, bank) in entries {
        grouped.entry((program, bank)).or_default().push(name);
    }

    grouped
        .into_iter()
        .map(|((program, bank), names)| {
            let canonical = format!("gm{}", names[0]);
            let aliases: Vec<String> = names[1..].iter().map(|a| format!("gm{a}")).collect();
            let family = gm_family(program, bank);
            GmPresetDoc {
                name: canonical,
                aliases,
                program,
                bank,
                family,
            }
        })
        .collect()
}

fn gm_family(program: u16, bank: u16) -> &'static str {
    if bank == 128 {
        return "Percussion";
    }
    match program {
        0..=7 => "Piano",
        8..=15 => "Chromatic Percussion",
        16..=23 => "Organ",
        24..=31 => "Guitar",
        32..=39 => "Bass",
        40..=47 => "Strings",
        48..=55 => "Ensemble",
        56..=63 => "Brass",
        64..=71 => "Reed",
        72..=79 => "Pipe",
        80..=95 => "Synth",
        96..=103 => "SFX",
        104..=111 => "World",
        112..=119 => "Percussion",
        _ => "Other",
    }
}

pub struct GmPresetDoc {
    pub name: String,
    pub aliases: Vec<String>,
    pub program: u16,
    pub bank: u16,
    pub family: &'static str,
}

fn gen_i16(zone: &soundfont::Zone, ty: GeneratorType) -> Option<i16> {
    zone.gen_list
        .iter()
        .find(|g| g.ty == ty)
        .and_then(|g| g.amount.as_i16().copied())
}

fn gen_range(zone: &soundfont::Zone, ty: GeneratorType) -> Option<(u8, u8)> {
    zone.gen_list
        .iter()
        .find(|g| g.ty == ty)
        .and_then(|g| g.amount.as_range())
        .map(|r| (r.low, r.high))
}

fn gen_u16(zone: &soundfont::Zone, ty: GeneratorType) -> Option<u16> {
    zone.gen_list
        .iter()
        .find(|g| g.ty == ty)
        .and_then(|g| g.amount.as_u16().copied())
}

/// SF2 sustain centibels to linear level: 10^(-cb/200)
fn centibels_to_linear(cb: i16) -> f32 {
    if cb >= 1000 {
        return 0.0;
    }
    if cb <= 0 {
        return 1.0;
    }
    10.0_f32.powf(-cb as f32 / 200.0)
}

/// SF2 timecents to seconds: 2^(tc/1200). Takes i32 so key-scaled envelope
/// times (which can exceed i16) don't overflow.
fn timecents_to_secs(tc: i32) -> f32 {
    if tc <= -12000 {
        0.001
    } else {
        2.0_f32.powf(tc as f32 / 1200.0)
    }
}

/// Absolute cents to Hz: 8.176 * 2^(cents/1200). Used for LFO rates.
fn abscents_to_hz(cents: i16) -> f32 {
    8.176 * 2.0_f32.powf(cents as f32 / 1200.0)
}

/// FluidSynth `FLUID_PEAK_ATTENUATION` (src/utils/fluid_conv_tables.h).
const FLUID_PEAK_ATTENUATION: f32 = 960.0;
/// `FLUID_VEL_CB_SIZE - 1`: the concave curve spans integer indices 0..=127.
const VEL_CB_MAX_INDEX: f32 = 127.0;

/// FluidSynth's concave transform on a normalized input `x` in `[0, 1]`, ported
/// verbatim from `src/gentables/fluid_concave.cpp` (coefficient `-200*2/960`,
/// endpoints pinned to 0 and 1). The SF2 2.04 §8.2.4 formula is wrong — this is
/// the implemented curve ("according to the pictures on SF2.01 page 73").
fn concave(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    -(200.0 * 2.0 / FLUID_PEAK_ATTENUATION) * (1.0 - x).log10()
}

/// SF2 default modulator #1 (NoteOnVelocity → Initial Attenuation, concave,
/// negative-unipolar, amount 960 cB). Returns the centibels of additional
/// attenuation to add to the zone's `initialAttenuation`: ~0 cB at vel=127,
/// ~952.5 cB at vel=0 (FluidSynth's 127/128 clamp). Ported from
/// `fluid_mod.c` / `fluid_synth.c` `default_vel2att_mod`.
pub fn velocity_to_attenuation_cb(vel: u8) -> f32 {
    let v = vel.min(127) as f32;
    let inv_norm = (VEL_CB_MAX_INDEX - v) / VEL_CB_MAX_INDEX;
    FLUID_PEAK_ATTENUATION * concave(inv_norm).min(127.0 / 128.0)
}

/// Centibels of attenuation → linear amplitude gain: `10^(-cb/200)`
/// (FluidSynth `fluid_cb2amp`).
pub fn cb_to_linear_gain(cb: f32) -> f32 {
    10.0_f32.powf(-cb / 200.0)
}

/// Native PCM for one SF2 sample header, kept at its original sample rate.
struct SampleSrc {
    pcm: Vec<f32>,
    native_sr: u32,
    link: u16,
    is_left: bool,
    is_right: bool,
}

pub fn load_sf2(path: &Path, device_sr: f32) -> Result<GmBank, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Failed to open SF2: {e}"))?;
    let sf2 = SoundFont2::load(&mut file).map_err(|e| format!("Failed to parse SF2: {e}"))?;

    let smpl = sf2.sample_data.smpl.ok_or("SF2 has no sample data")?;

    file.seek(SeekFrom::Start(smpl.offset))
        .map_err(|e| format!("Failed to seek to sample data: {e}"))?;
    let mut raw_bytes = vec![0u8; smpl.len as usize];
    file.read_exact(&mut raw_bytes)
        .map_err(|e| format!("Failed to read sample data: {e}"))?;

    let raw_i16: Vec<i16> = raw_bytes
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();
    drop(raw_bytes);

    // Decode each sample header to native-rate f32 PCM (no resampling: pitch and
    // device-rate conversion both happen at playback time via the cursor speed).
    // Vorbis (SF3) and ROM samples can't be read as raw PCM, so they're skipped.
    let mut srcs: Vec<Option<SampleSrc>> = Vec::with_capacity(sf2.sample_headers.len());
    for hdr in &sf2.sample_headers {
        let start = hdr.start as usize;
        let end = hdr.end as usize;
        if start >= end
            || end > raw_i16.len()
            || hdr.sample_type.is_vorbis()
            || hdr.sample_type.is_rom()
        {
            srcs.push(None);
            continue;
        }
        let pcm: Vec<f32> = raw_i16[start..end]
            .iter()
            .map(|&s| s as f32 / 32768.0)
            .collect();
        srcs.push(Some(SampleSrc {
            pcm,
            native_sr: hdr.sample_rate,
            link: hdr.sample_link,
            is_left: hdr.sample_type.is_left(),
            is_right: hdr.sample_type.is_right(),
        }));
    }

    let zones = build_zone_table(&sf2, &srcs, device_sr);
    Ok(GmBank::new(zones))
}

/// Build (or fetch from cache) the mono `SampleData` for one sample index.
fn mono_arc(
    idx: usize,
    srcs: &[Option<SampleSrc>],
    headers: &[soundfont::raw::SampleHeader],
    cache: &mut [Option<Arc<SampleData>>],
) -> Option<Arc<SampleData>> {
    if let Some(a) = &cache[idx] {
        return Some(Arc::clone(a));
    }
    let src = srcs[idx].as_ref()?;
    let hdr = &headers[idx];
    let freq = midi2freq(hdr.origpitch as f32 + hdr.pitchadj as f32 / 100.0);
    let arc = Arc::new(SampleData::new(src.pcm.clone(), 1, freq));
    cache[idx] = Some(Arc::clone(&arc));
    Some(arc)
}

/// Build (or fetch from cache) a 2-channel interleaved `SampleData` from a
/// linked L/R sample pair (SF2 §7.10). Frame count is the shorter of the two.
fn stereo_arc(
    left_idx: usize,
    right_idx: usize,
    srcs: &[Option<SampleSrc>],
    headers: &[soundfont::raw::SampleHeader],
    cache: &mut HashMap<(usize, usize), Arc<SampleData>>,
) -> Option<Arc<SampleData>> {
    let key = (left_idx, right_idx);
    if let Some(a) = cache.get(&key) {
        return Some(Arc::clone(a));
    }
    let left = srcs[left_idx].as_ref()?;
    let right = srcs[right_idx].as_ref()?;
    let frames = left.pcm.len().min(right.pcm.len());
    let mut interleaved = Vec::with_capacity(frames * 2);
    for f in 0..frames {
        interleaved.push(left.pcm[f]);
        interleaved.push(right.pcm[f]);
    }
    let hdr = &headers[right_idx];
    let freq = midi2freq(hdr.origpitch as f32 + hdr.pitchadj as f32 / 100.0);
    let arc = Arc::new(SampleData::new(interleaved, 2, freq));
    cache.insert(key, Arc::clone(&arc));
    Some(arc)
}

fn build_zone_table(
    sf2: &SoundFont2,
    srcs: &[Option<SampleSrc>],
    device_sr: f32,
) -> Vec<ZoneEntry> {
    let mut entries = Vec::new();
    let mut mono_cache: Vec<Option<Arc<SampleData>>> = vec![None; sf2.sample_headers.len()];
    let mut stereo_cache: HashMap<(usize, usize), Arc<SampleData>> = HashMap::new();

    for preset in &sf2.presets {
        let program = preset.header.preset;
        let bank = preset.header.bank;

        // Detect global zone (first zone with no instrument reference)
        let (preset_global, preset_zones) = if !preset.zones.is_empty()
            && gen_u16(&preset.zones[0], GeneratorType::Instrument).is_none()
        {
            (Some(&preset.zones[0]), &preset.zones[1..])
        } else {
            (None, preset.zones.as_slice())
        };

        for pzone in preset_zones {
            let inst_idx = match gen_u16(pzone, GeneratorType::Instrument) {
                Some(idx) => idx as usize,
                None => continue,
            };
            let instrument = match sf2.instruments.get(inst_idx) {
                Some(i) => i,
                None => continue,
            };

            let p_key = gen_range(pzone, GeneratorType::KeyRange).unwrap_or((0, 127));
            let p_vel = gen_range(pzone, GeneratorType::VelRange).unwrap_or((0, 127));

            // Detect instrument global zone
            let (inst_global, inst_zones) = if !instrument.zones.is_empty()
                && gen_u16(&instrument.zones[0], GeneratorType::SampleID).is_none()
            {
                (Some(&instrument.zones[0]), &instrument.zones[1..])
            } else {
                (None, instrument.zones.as_slice())
            };

            for izone in inst_zones {
                let sample_idx = match gen_u16(izone, GeneratorType::SampleID) {
                    Some(idx) => idx as usize,
                    None => continue,
                };

                let hdr = match sf2.sample_headers.get(sample_idx) {
                    Some(h) => h,
                    None => continue,
                };
                let src = match srcs.get(sample_idx).and_then(|s| s.as_ref()) {
                    Some(s) => s,
                    None => continue,
                };

                let i_key = gen_range(izone, GeneratorType::KeyRange).unwrap_or((0, 127));
                let i_vel = gen_range(izone, GeneratorType::VelRange).unwrap_or((0, 127));

                // Intersect key/vel ranges
                let key_lo = i_key.0.max(p_key.0);
                let key_hi = i_key.1.min(p_key.1);
                let vel_lo = i_vel.0.max(p_vel.0);
                let vel_hi = i_vel.1.min(p_vel.1);
                if key_lo > key_hi || vel_lo > vel_hi {
                    continue;
                }

                // SF2 spec Section 9.4: instrument-level generators use fallback
                // (zone → global), preset-level generators are additive offsets.
                let inst_val = |ty: GeneratorType| -> Option<i16> {
                    gen_i16(izone, ty).or_else(|| inst_global.and_then(|z| gen_i16(z, ty)))
                };
                let preset_offset = |ty: GeneratorType| -> i16 {
                    gen_i16(pzone, ty)
                        .or_else(|| preset_global.and_then(|z| gen_i16(z, ty)))
                        .unwrap_or(0)
                };
                let get = |ty: GeneratorType, default: i16| -> i16 {
                    inst_val(ty).unwrap_or(default) + preset_offset(ty)
                };

                // Resolve the sample data: merge a linked L/R pair into one stereo
                // SampleData (the read path is already interleaved-aware), else mono.
                let data = if (src.is_left || src.is_right) && (src.link as usize) < srcs.len() {
                    let partner = src.link as usize;
                    let partner_ok = srcs
                        .get(partner)
                        .and_then(|s| s.as_ref())
                        .map(|p| p.is_left != src.is_left)
                        .unwrap_or(false);
                    if partner_ok {
                        let (l, r) = if src.is_left {
                            (sample_idx, partner)
                        } else {
                            (partner, sample_idx)
                        };
                        stereo_arc(l, r, srcs, &sf2.sample_headers, &mut stereo_cache)
                    } else {
                        mono_arc(sample_idx, srcs, &sf2.sample_headers, &mut mono_cache)
                    }
                } else {
                    mono_arc(sample_idx, srcs, &sf2.sample_headers, &mut mono_cache)
                };
                let data = match data {
                    Some(d) => d,
                    None => continue,
                };

                // Root key (override, not additive)
                let root_key = inst_val(GeneratorType::OverridingRootKey)
                    .filter(|&k| k >= 0)
                    .map(|k| k as u8)
                    .unwrap_or(hdr.origpitch);
                let coarse_tune = get(GeneratorType::CoarseTune, 0);
                let fine_tune = get(GeneratorType::FineTune, 0) + hdr.pitchadj as i16;
                let root_freq =
                    midi2freq(root_key as f32 + coarse_tune as f32 + fine_tune as f32 / 100.0);

                let sr_ratio = src.native_sr as f32 / device_sr;

                // Loop points in native frames, relative to the sample start. No
                // resample ratio: the sample is stored at native rate. Clamp to
                // the frame count — a no-op for mono, but a merged stereo pair is
                // only as long as its shorter channel, so a malformed font can't
                // push the loop past the buffer.
                let sample_start = hdr.start;
                let frames = data.frame_count as f32;
                let loop_start = (hdr.loop_start.saturating_sub(sample_start) as f32).min(frames);
                let loop_end = (hdr.loop_end.saturating_sub(sample_start) as f32).min(frames);

                // Sample mode: 0=no loop, 1=loop continuous, 3=loop until release.
                let sample_mode = inst_val(GeneratorType::SampleModes).unwrap_or(0);
                let loops = sample_mode == 1 || sample_mode == 3;
                let valid_loop = loops && loop_end > loop_start + 1.0;
                let loop_until_release = valid_loop && sample_mode == 3;

                // Initial attenuation. The EMU8k/10k 0.4 factor on the generator
                // value matches FluidSynth/BASSMIDI — virtually every GM font is
                // authored expecting it, and spec-literal attenuation plays them far
                // too quietly. Scope is the generator only (the velocity concave and
                // sustainVolEnv stay unscaled).
                let attenuation =
                    cb_to_linear_gain(0.4 * get(GeneratorType::InitialAttenuation, 0) as f32);

                // Pan (-500..500 = -50%..+50%, default 0 = center). A sample that
                // belongs to a stereo L/R pair encodes its side in the pan
                // generator (left = -500, right = +500); since the pair is merged
                // into one 2-channel buffer, applying that generator would collapse
                // the stereo image back onto one side (a hard-panned piano). Stereo
                // zones therefore stay centered (pan 0.5 makes the Pan stage a no-op).
                let pan = if src.is_left || src.is_right {
                    0.5
                } else {
                    let pan_raw = get(GeneratorType::Pan, 0);
                    (pan_raw as f32 / 1000.0).clamp(-0.5, 0.5) + 0.5
                };

                // Filter: cutoff stays in cents (the velocity term is added per
                // note); Q in centibels maps to 0..1 resonance. No passband makeup
                // gain — doux's SVF lowpass keeps unity passband at all Q (resonance
                // only adds a peak at cutoff), so there is nothing to compensate.
                let filter_fc_cents = get(GeneratorType::InitialFilterFc, 13500) as f32;
                let fq_cb = get(GeneratorType::InitialFilterQ, 0).max(0);
                let filter_q = (fq_cb as f32 / 960.0).clamp(0.0, 1.0);

                // Scale tuning (cents/key, default 100 = normal chromatic)
                let scale_tuning = get(GeneratorType::ScaleTuning, 100) as f32 / 100.0;

                // Vibrato LFO → pitch (gens 6/24). Mapped onto the existing voice
                // vibrato; only carried when the depth generator is non-zero.
                let vib_depth_cents = get(GeneratorType::VibLfoToPitch, 0);
                let (vib_rate, vib_depth) = if vib_depth_cents != 0 {
                    (
                        abscents_to_hz(get(GeneratorType::FreqVibLFO, 0)),
                        vib_depth_cents.abs() as f32 / 100.0,
                    )
                } else {
                    (0.0, 0.0)
                };

                let exclusive_class = get(GeneratorType::ExclusiveClass, 0).clamp(0, 127) as u8;

                // Volume envelope. delay/attack/sustain/release resolve to
                // seconds now; hold/decay keep raw timecents so key-scaling
                // (gens 39/40) can be applied at note-on.
                let delay = timecents_to_secs(get(GeneratorType::DelayVolEnv, -12000) as i32);
                let attack = timecents_to_secs(get(GeneratorType::AttackVolEnv, -12000) as i32);
                let hold_tc = get(GeneratorType::HoldVolEnv, -12000);
                let decay_tc = get(GeneratorType::DecayVolEnv, -12000);
                let keynum_to_hold = get(GeneratorType::KeynumToVolEnvHold, 0);
                let keynum_to_decay = get(GeneratorType::KeynumToVolEnvDecay, 0);
                let sustain = centibels_to_linear(get(GeneratorType::SustainVolEnv, 0));
                let release = timecents_to_secs(get(GeneratorType::ReleaseVolEnv, -12000) as i32);

                entries.push(ZoneEntry {
                    program,
                    bank,
                    key_lo,
                    key_hi,
                    vel_lo,
                    vel_hi,
                    data,
                    root_freq,
                    sr_ratio,
                    loop_start: if valid_loop { loop_start } else { 0.0 },
                    loop_end: if valid_loop { loop_end } else { 0.0 },
                    looping: valid_loop,
                    loop_until_release,
                    attenuation,
                    pan,
                    filter_fc_cents,
                    filter_q,
                    scale_tuning,
                    vib_rate,
                    vib_depth,
                    exclusive_class,
                    delay,
                    attack,
                    hold_tc,
                    decay_tc,
                    keynum_to_hold,
                    keynum_to_decay,
                    sustain,
                    release,
                });
            }
        }
    }

    entries
}

/// Find the first .sf2 file in a directory.
pub fn find_sf2_file(dir: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("sf2") {
                    return Some(path);
                }
            }
        }
    }
    None
}
