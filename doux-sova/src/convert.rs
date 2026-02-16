use sova_core::{protocol::audio_engine_proxy::AudioEnginePayload, vm::variable::VariableValue};

use crate::time::TimeConverter;

pub fn payload_to_command(
    payload: AudioEnginePayload,
    time_converter: &TimeConverter,
) -> String {
    use std::fmt::Write;
    let mut cmd = String::with_capacity(payload.args.len() * 16 + 32);

    if let Some(tt) = payload.timetag {
        let engine_time = time_converter.sync_to_engine_time(tt);
        write!(cmd, "/time/{engine_time}").unwrap();
    }

    for (key, value) in payload.args {
        cmd.push('/');
        cmd.push_str(&key);
        cmd.push('/');
        push_value(&mut cmd, value);
    }

    if cmd.is_empty() {
        cmd.push('/');
    }
    cmd
}

fn push_value(buf: &mut String, v: VariableValue) {
    use std::fmt::Write;
    match v {
        VariableValue::Integer(i) => write!(buf, "{i}").unwrap(),
        VariableValue::Float(f) => write!(buf, "{f}").unwrap(),
        VariableValue::Decimal(d) => {
            write!(buf, "{}", f64::from(d)).unwrap()
        }
        VariableValue::Str(s) => buf.push_str(&s),
        VariableValue::Bool(b) => buf.push(if b { '1' } else { '0' }),
        _ => ()
    }
}
