//! OSC (Open Sound Control) message receiver.
//!
//! Listens for UDP packets containing OSC messages and translates them into
//! engine commands. Runs in a dedicated thread, forwarding parsed messages
//! to the audio engine for evaluation.
//!
//! # Message Format
//!
//! OSC arguments are interpreted as key-value pairs and converted to a path
//! string for the engine. Arguments are processed in pairs: odd positions are
//! keys (must be strings), even positions are values.
//!
//! ```text
//! OSC: /play ["sound", "kick", "note", 60, "amp", 0.8]
//!  →   Engine path: "sound/kick/note/60/amp/0.8"
//! ```
//!
//! # Scheduling
//!
//! OSC bundle timetags (NTP) are honored. A timetagged bundle resolves to a
//! sample-accurate engine tick via [`TimeAnchor`]; messages inside the bundle
//! inherit that tick unless they carry an explicit in-band `tick` / `time` /
//! `delta` arg, which takes precedence. The OSC "immediately" sentinel
//! `(0, 1)` falls through to fire-on-receipt.
//!
//! # Protocol
//!
//! - Transport: UDP
//! - Default bind: `0.0.0.0:<port>` (all interfaces)
//! - Supports both single messages and bundles (bundles are flattened)

use crate::time::TimeAnchor;
use crate::{AudioCmd, EngineMetrics};
use crossbeam_channel::{Sender, TrySendError};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Maximum UDP packet size for incoming OSC messages.
const BUFFER_SIZE: usize = 4096;

/// Starts the OSC receiver loop on the specified port.
///
/// Binds to all interfaces (`0.0.0.0`) and returns when `device_lost` is set.
/// Uses a 500ms socket timeout so the loop periodically checks the flag.
/// Returns `Ok(true)` if it exited due to device loss, `Ok(false)` otherwise.
pub fn run_recoverable(
    tx: Sender<AudioCmd>,
    port: u16,
    anchor: TimeAnchor,
    device_lost: &AtomicBool,
    metrics: Arc<EngineMetrics>,
) -> std::io::Result<bool> {
    let addr = format!("0.0.0.0:{port}");
    let socket = UdpSocket::bind(&addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let mut buf = [0u8; BUFFER_SIZE];

    loop {
        if device_lost.load(Ordering::Acquire) {
            return Ok(true);
        }
        match socket.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                if let Ok(packet) = rosc::decoder::decode_udp(&buf[..size]) {
                    handle_packet(&tx, &packet.1, &anchor, None, &metrics);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                eprintln!("OSC recv error: {e}");
            }
        }
    }
}

/// Recursively processes an OSC packet, handling both messages and bundles.
///
/// `parent_tick` propagates a tick resolved from an outer bundle's timetag.
/// Nested bundles override with their own timetag.
fn handle_packet(
    tx: &Sender<AudioCmd>,
    packet: &OscPacket,
    anchor: &TimeAnchor,
    parent_tick: Option<u64>,
    metrics: &EngineMetrics,
) {
    match packet {
        OscPacket::Message(msg) => handle_message(tx, msg, anchor, parent_tick, metrics),
        OscPacket::Bundle(bundle) => {
            let tick = anchor
                .ntp_to_tick(bundle.timetag.seconds, bundle.timetag.fractional)
                .or(parent_tick);
            for p in &bundle.content {
                handle_packet(tx, p, anchor, tick, metrics);
            }
        }
    }
}

/// Converts an OSC message to an `Event` and sends it as an AudioCmd.
///
/// `Event::parse` runs here on the OSC receiver thread to keep all allocation
/// off the audio callback. The bundle-timetag `parent_tick` is applied only
/// when the message itself did not carry an explicit tick.
fn handle_message(
    tx: &Sender<AudioCmd>,
    msg: &OscMessage,
    anchor: &TimeAnchor,
    parent_tick: Option<u64>,
    metrics: &EngineMetrics,
) {
    let path = osc_to_path(msg);
    if path.is_empty() {
        return;
    }
    let mut event = crate::event::Event::parse(&path, anchor.sample_rate);
    if event.tick.is_none() {
        event.tick = parent_tick;
    }
    match tx.try_send(AudioCmd::DispatchEvent(event)) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            metrics.dropped_cmds.fetch_add(1, Ordering::Relaxed);
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

/// Converts OSC message arguments to a slash-separated path string.
///
/// Arguments are processed as key-value pairs. Keys must be strings;
/// non-string keys cause the pair to be skipped. Values are written
/// directly into a single String without intermediate allocations.
fn osc_to_path(msg: &OscMessage) -> String {
    let args = &msg.args;
    let mut path = String::with_capacity(args.len() * 8);
    let mut i = 0;

    while i + 1 < args.len() {
        let key = match &args[i] {
            OscType::String(s) => s.as_str(),
            _ => {
                i += 1;
                continue;
            }
        };
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(key);
        path.push('/');
        push_osc_arg(&mut path, &args[i + 1]);
        i += 2;
    }

    path
}

fn push_osc_arg(buf: &mut String, arg: &OscType) {
    use std::fmt::Write;
    match arg {
        OscType::Int(v) => write!(buf, "{v}").unwrap(),
        OscType::Float(v) => write!(buf, "{v}").unwrap(),
        OscType::Double(v) => write!(buf, "{v}").unwrap(),
        OscType::Long(v) => write!(buf, "{v}").unwrap(),
        OscType::String(s) => buf.push_str(s),
        OscType::Bool(b) => buf.push(if *b { '1' } else { '0' }),
        _ => {}
    }
}
