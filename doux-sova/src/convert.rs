use std::collections::HashMap;

use crate::time::TimeConverter;
use crate::types::{ParamValue, SyncTime};

pub fn payload_to_command(
    args: &HashMap<String, ParamValue>,
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

fn push_value(buf: &mut String, v: &ParamValue) {
    use std::fmt::Write;
    match v {
        ParamValue::Integer(i) => write!(buf, "{i}").unwrap(),
        ParamValue::Float(f) => write!(buf, "{f}").unwrap(),
        ParamValue::Decimal(sign, num, den) => {
            let f = (*num as f64) / (*den as f64);
            if *sign < 0 {
                write!(buf, "-{f}").unwrap();
            } else {
                write!(buf, "{f}").unwrap();
            }
        }
        ParamValue::Str(s) => buf.push_str(s),
        ParamValue::Bool(b) => buf.push(if *b { '1' } else { '0' }),
    }
}
