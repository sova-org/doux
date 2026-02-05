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
//! # Protocol
//!
//! - Transport: UDP
//! - Default bind: `0.0.0.0:<port>` (all interfaces)
//! - Supports both single messages and bundles (bundles are flattened)

use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

use crate::Engine;

/// Maximum UDP packet size for incoming OSC messages.
const BUFFER_SIZE: usize = 4096;

/// Starts the OSC receiver loop on the specified port.
///
/// Binds to all interfaces (`0.0.0.0`) and blocks indefinitely, processing
/// incoming messages. Intended to be spawned in a dedicated thread.
///
/// # Panics
///
/// Panics if the UDP socket cannot be bound (e.g., port already in use).
pub fn run(engine: Arc<Mutex<Engine>>, port: u16) {
    let addr = format!("0.0.0.0:{port}");
    let socket = UdpSocket::bind(&addr).expect("failed to bind OSC socket");

    let mut buf = [0u8; BUFFER_SIZE];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                if let Ok(packet) = rosc::decoder::decode_udp(&buf[..size]) {
                    handle_packet(&engine, &packet.1);
                }
            }
            Err(e) => {
                eprintln!("OSC recv error: {e}");
            }
        }
    }
}

/// Recursively processes an OSC packet, handling both messages and bundles.
fn handle_packet(engine: &Arc<Mutex<Engine>>, packet: &OscPacket) {
    match packet {
        OscPacket::Message(msg) => handle_message(engine, msg),
        OscPacket::Bundle(bundle) => {
            for p in &bundle.content {
                handle_packet(engine, p);
            }
        }
    }
}

/// Converts an OSC message to a path string and evaluates it on the engine.
fn handle_message(engine: &Arc<Mutex<Engine>>, msg: &OscMessage) {
    let path = osc_to_path(msg);
    if !path.is_empty() {
        if let Ok(mut e) = engine.lock() {
            e.evaluate(&path);
        }
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
