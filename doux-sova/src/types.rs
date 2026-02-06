use std::collections::HashMap;

pub type SyncTime = u64;

pub enum ParamValue {
    Integer(i64),
    Float(f64),
    Decimal(i8, u64, u64),
    Str(String),
    Bool(bool),
}

pub struct AudioPayload {
    pub args: HashMap<String, ParamValue>,
    pub timetag: Option<SyncTime>,
}
