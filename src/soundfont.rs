use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use soundfont::raw::GeneratorType;
use soundfont::SoundFont2;

use crate::sampling::SampleData;
use crate::types::midi2freq;

struct ZoneEntry {
    preset: u16,
    bank: u16,
    key_lo: u8,
    key_hi: u8,
    vel_lo: u8,
    vel_hi: u8,
    sample_name: String,
    root_freq: f32,
    loop_start: f32,
    loop_end: f32,
    looping: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

pub struct GmZone<'a> {
    pub sample_name: &'a str,
    pub root_freq: f32,
    pub loop_start: f32,
    pub loop_end: f32,
    pub looping: bool,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

pub struct GmBank {
    zones: Vec<ZoneEntry>,
}

impl GmBank {
    pub fn find(&self, program_str: &str, note: u8, vel: u8) -> Option<GmZone<'_>> {
        let (preset, bank) = resolve_gm_program(program_str)?;
        self.zones
            .iter()
            .find(|z| {
                z.preset == preset
                    && z.bank == bank
                    && note >= z.key_lo
                    && note <= z.key_hi
                    && vel >= z.vel_lo
                    && vel <= z.vel_hi
            })
            .map(|z| GmZone {
                sample_name: &z.sample_name,
                root_freq: z.root_freq,
                loop_start: z.loop_start,
                loop_end: z.loop_end,
                looping: z.looping,
                attack: z.attack,
                decay: z.decay,
                sustain: z.sustain,
                release: z.release,
            })
    }

    pub fn preset_count(&self) -> usize {
        let mut seen = Vec::new();
        for z in &self.zones {
            let key = (z.preset, z.bank);
            if !seen.contains(&key) {
                seen.push(key);
            }
        }
        seen.len()
    }
}

fn resolve_gm_program(s: &str) -> Option<(u16, u16)> {
    if let Ok(n) = s.parse::<u16>() {
        return if n < 128 { Some((n, 0)) } else { None };
    }
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
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
    if cb >= 1000 { return 0.0; }
    if cb <= 0 { return 1.0; }
    10.0_f32.powf(-cb as f32 / 200.0)
}

/// SF2 timecents to seconds: 2^(tc/1200)
fn timecents_to_secs(tc: i16) -> f32 {
    if tc <= -12000 {
        0.001
    } else {
        2.0_f32.powf(tc as f32 / 1200.0)
    }
}

pub fn load_sf2(path: &Path, target_sr: f32) -> Result<(Vec<(String, SampleData)>, GmBank), String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open SF2: {e}"))?;
    let sf2 = SoundFont2::load(&mut file).map_err(|e| format!("Failed to parse SF2: {e}"))?;

    let smpl = sf2
        .sample_data
        .smpl
        .ok_or("SF2 has no sample data")?;

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

    // Extract each sample into SampleData, keyed by index
    let mut samples = Vec::new();
    let mut sample_ratios: Vec<f32> = Vec::new();

    for (i, hdr) in sf2.sample_headers.iter().enumerate() {
        let start = hdr.start as usize;
        let end = hdr.end as usize;
        if start >= end || end > raw_i16.len() {
            sample_ratios.push(1.0);
            continue;
        }

        let pcm: Vec<f32> = raw_i16[start..end]
            .iter()
            .map(|&s| s as f32 / 32768.0)
            .collect();

        let needs_resample = (hdr.sample_rate as f32 - target_sr).abs() > 1.0;
        let ratio = if needs_resample {
            target_sr / hdr.sample_rate as f32
        } else {
            1.0
        };
        sample_ratios.push(ratio);

        let pcm = if needs_resample {
            resample_linear(&pcm, 1, hdr.sample_rate as f32, target_sr)
        } else {
            pcm
        };

        let root_note = hdr.origpitch as f32 + hdr.pitchadj as f32 / 100.0;
        let root_freq = midi2freq(root_note);
        let name = format!("_sf2_{i}");
        samples.push((name, SampleData::new(pcm, 1, root_freq)));
    }

    drop(raw_i16);

    // Build zone lookup table
    let zones = build_zone_table(&sf2, &sample_ratios);
    let bank = GmBank { zones };

    Ok((samples, bank))
}

fn build_zone_table(sf2: &SoundFont2, sample_ratios: &[f32]) -> Vec<ZoneEntry> {
    let mut entries = Vec::new();

    for preset in &sf2.presets {
        let program = preset.header.preset;
        let bank = preset.header.bank;

        // Detect global zone (first zone with no instrument reference)
        let (preset_global, preset_zones) =
            if !preset.zones.is_empty() && gen_u16(&preset.zones[0], GeneratorType::Instrument).is_none() {
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
            let (inst_global, inst_zones) =
                if !instrument.zones.is_empty() && gen_u16(&instrument.zones[0], GeneratorType::SampleID).is_none() {
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

                // Resolve generators with fallback chain: izone -> inst_global -> preset_global
                let get = |ty: GeneratorType| -> Option<i16> {
                    gen_i16(izone, ty)
                        .or_else(|| inst_global.and_then(|z| gen_i16(z, ty)))
                        .or_else(|| preset_global.and_then(|z| gen_i16(z, ty)))
                };

                // Root key (override or from sample header)
                let root_key = get(GeneratorType::OverridingRootKey)
                    .filter(|&k| k >= 0)
                    .map(|k| k as u8)
                    .unwrap_or(hdr.origpitch);
                let coarse_tune = get(GeneratorType::CoarseTune).unwrap_or(0);
                let fine_tune = get(GeneratorType::FineTune).unwrap_or(0)
                    + hdr.pitchadj as i16;
                let root_freq = midi2freq(root_key as f32 + coarse_tune as f32 + fine_tune as f32 / 100.0);

                // Loop points (adjusted for sample start offset and resampling)
                let ratio = sample_ratios.get(sample_idx).copied().unwrap_or(1.0);
                let sample_start = hdr.start;
                let loop_start_raw = hdr.loop_start.saturating_sub(sample_start);
                let loop_end_raw = hdr.loop_end.saturating_sub(sample_start);
                let loop_start = loop_start_raw as f32 * ratio;
                let loop_end = loop_end_raw as f32 * ratio;

                // Sample mode: 0=no loop, 1=loop continuous, 3=loop until release
                let sample_mode = get(GeneratorType::SampleModes).unwrap_or(0);
                let looping = sample_mode == 1 || sample_mode == 3;
                let valid_loop = looping && loop_end > loop_start + 1.0;

                // Volume envelope
                let attack = get(GeneratorType::AttackVolEnv)
                    .map(timecents_to_secs)
                    .unwrap_or(0.001);
                let decay = get(GeneratorType::DecayVolEnv)
                    .map(timecents_to_secs)
                    .unwrap_or(0.0);
                let sustain = get(GeneratorType::SustainVolEnv)
                    .map(centibels_to_linear)
                    .unwrap_or(1.0);
                let release = get(GeneratorType::ReleaseVolEnv)
                    .map(timecents_to_secs)
                    .unwrap_or(0.001);

                entries.push(ZoneEntry {
                    preset: program,
                    bank,
                    key_lo,
                    key_hi,
                    vel_lo,
                    vel_hi,
                    sample_name: format!("_sf2_{sample_idx}"),
                    root_freq,
                    loop_start: if valid_loop { loop_start } else { 0.0 },
                    loop_end: if valid_loop { loop_end } else { 0.0 },
                    looping: valid_loop,
                    attack,
                    decay,
                    sustain,
                    release,
                });
            }
        }
    }

    entries
}

fn resample_linear(samples: &[f32], channels: usize, from_sr: f32, to_sr: f32) -> Vec<f32> {
    let ratio = to_sr / from_sr;
    let in_frames = samples.len() / channels;
    let out_frames = (in_frames as f32 * ratio) as usize;
    let mut output = vec![0.0; out_frames * channels];

    for out_frame in 0..out_frames {
        let in_pos = out_frame as f32 / ratio;
        let in_frame = (in_pos as usize).min(in_frames.saturating_sub(1));
        let next_frame = (in_frame + 1).min(in_frames.saturating_sub(1));
        let frac = in_pos - in_frame as f32;

        for ch in 0..channels {
            let s0 = samples[in_frame * channels + ch];
            let s1 = samples[next_frame * channels + ch];
            output[out_frame * channels + ch] = s0 + frac * (s1 - s0);
        }
    }

    output
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
