//! Payload conversion from Sova to Doux command format.
//!
//! Converts Sova's `AudioEnginePayload` (HashMap of parameters) into
//! Doux's slash-separated command strings (e.g., `/sound/sine/freq/440`).

use std::collections::HashMap;

use sova_core::clock::SyncTime;
use sova_core::vm::variable::VariableValue;

use crate::time::TimeConverter;

/// Converts a Sova payload to a Doux command string.
///
/// The resulting string has the format `/key/value/key/value/...`.
/// If a timetag is present, it's converted to engine time and prepended.
pub fn payload_to_command(
    args: &HashMap<String, VariableValue>,
    timetag: Option<SyncTime>,
    time_converter: &TimeConverter,
) -> String {
    use std::fmt::Write;
    let mut cmd = String::with_capacity(args.len() * 16 + 32);

    if let Some(tt) = timetag {
        let engine_time = time_converter.sync_to_engine_time(tt);
        write!(cmd, "/time/{engine_time}").unwrap();
    }

    for (key, value) in args {
        cmd.push('/');
        cmd.push_str(key);
        cmd.push('/');
        push_value(&mut cmd, value);
    }

    if cmd.is_empty() {
        cmd.push('/');
    }
    cmd
}

fn push_value(buf: &mut String, v: &VariableValue) {
    use std::fmt::Write;
    match v {
        VariableValue::Integer(i) => write!(buf, "{i}").unwrap(),
        VariableValue::Float(f) => write!(buf, "{f}").unwrap(),
        VariableValue::Decimal(sign, num, den) => {
            let f = (*num as f64) / (*den as f64);
            if *sign < 0 {
                write!(buf, "-{f}").unwrap();
            } else {
                write!(buf, "{f}").unwrap();
            }
        }
        VariableValue::Str(s) => buf.push_str(s),
        VariableValue::Bool(b) => buf.push(if *b { '1' } else { '0' }),
        _ => {}
    }
}
