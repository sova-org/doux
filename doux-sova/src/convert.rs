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
    let mut parts = Vec::new();

    if let Some(tt) = timetag {
        let engine_time = time_converter.sync_to_engine_time(tt);
        parts.push("time".to_string());
        parts.push(engine_time.to_string());
    }

    for (key, value) in args {
        parts.push(key.clone());
        parts.push(value_to_string(value));
    }

    format!("/{}", parts.join("/"))
}

/// Converts a Sova variable value to a string for Doux.
fn value_to_string(v: &VariableValue) -> String {
    match v {
        VariableValue::Integer(i) => i.to_string(),
        VariableValue::Float(f) => f.to_string(),
        VariableValue::Decimal(sign, num, den) => {
            let f = (*num as f64) / (*den as f64);
            if *sign < 0 {
                format!("-{f}")
            } else {
                f.to_string()
            }
        }
        VariableValue::Str(s) => s.clone(),
        VariableValue::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        _ => String::new(),
    }
}
