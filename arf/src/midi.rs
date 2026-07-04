//! MIDI input: shared message parsing.
//!
//! [`parse`] and [`MidiMsg`] are target-independent — every host feeds them raw MIDI bytes
//! so all front-ends react to MIDI identically. Omni — the MIDI channel (the status byte's
//! low nibble) is ignored.

/// A parsed MIDI message — the subset arf reacts to. The channel is dropped (omni).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiMsg {
    /// A key was pressed: `note` (0..=127) at `vel` (1..=127).
    NoteOn { note: u8, vel: u8 },
    /// A key was released: `note` (0..=127). A note-on with velocity 0 parses as this.
    NoteOff { note: u8 },
    /// A control change: controller `num` (0..=127) set to `val` (0..=127).
    Cc { num: u8, val: u8 },
}

/// Parse one raw MIDI message into the subset arf reacts to, or `None` for anything else
/// (system messages, aftertouch, program change, …). Omni: the status low nibble (channel)
/// is ignored. A note-on with velocity 0 is the conventional note-off and parses as one.
pub fn parse(bytes: &[u8]) -> Option<MidiMsg> {
    let [status, data1, data2, ..] = bytes else {
        return None;
    };
    // Data bytes are 7-bit by spec; mask the high bit off so a non-conformant or corrupt
    // stream can never drive a `num`/`note` out of 0..=127 (a high CC byte indexed past the
    // control plane and panicked the audio thread).
    let (data1, data2) = (data1 & 0x7F, data2 & 0x7F);
    match status & 0xF0 {
        0x90 if data2 > 0 => Some(MidiMsg::NoteOn { note: data1, vel: data2 }),
        0x90 => Some(MidiMsg::NoteOff { note: data1 }), // velocity 0 = note-off
        0x80 => Some(MidiMsg::NoteOff { note: data1 }),
        0xB0 => Some(MidiMsg::Cc { num: data1, val: data2 }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_handled_messages() {
        assert_eq!(parse(&[0x90, 60, 100]), Some(MidiMsg::NoteOn { note: 60, vel: 100 }));
        // A note-on with velocity 0 is the conventional note-off.
        assert_eq!(parse(&[0x90, 60, 0]), Some(MidiMsg::NoteOff { note: 60 }));
        assert_eq!(parse(&[0x80, 60, 64]), Some(MidiMsg::NoteOff { note: 60 }));
        assert_eq!(parse(&[0xB0, 74, 32]), Some(MidiMsg::Cc { num: 74, val: 32 }));
    }

    #[test]
    fn channel_is_ignored_omni() {
        // The status low nibble is the channel; the same message on channel 5 still parses.
        assert_eq!(parse(&[0x95, 60, 100]), Some(MidiMsg::NoteOn { note: 60, vel: 100 }));
        assert_eq!(parse(&[0x8F, 60, 64]), Some(MidiMsg::NoteOff { note: 60 }));
    }

    #[test]
    fn data_bytes_are_masked_to_seven_bits() {
        // A non-conformant or corrupt stream can set the high bit on a data byte. Mask every
        // data byte to 0..=127 at the boundary so a CC number can never index past the control
        // plane (it did: `control[CC_BASE + num]` panicked the audio thread for num >= 128).
        assert_eq!(parse(&[0xB0, 0xFF, 0xFF]), Some(MidiMsg::Cc { num: 0x7F, val: 0x7F }));
        assert_eq!(parse(&[0x90, 0x80, 0x81]), Some(MidiMsg::NoteOn { note: 0, vel: 1 }));
    }

    #[test]
    fn unhandled_and_short_messages_are_none() {
        assert_eq!(parse(&[0xC0, 5]), None, "program change ignored");
        assert_eq!(parse(&[0xE0, 0, 64]), None, "pitch bend ignored in v1");
        assert_eq!(parse(&[0x90, 60]), None, "short message ignored");
        assert_eq!(parse(&[]), None);
    }
}
