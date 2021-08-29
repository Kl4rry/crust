use std::collections::HashMap;

use thin_string::ThinString;
use thin_vec::ThinVec;

#[allow(dead_code)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(ThinString),
    List(ThinVec<Value>),
    Map(Box<HashMap<Value, Value>>),
    Range(Box<Range>),
    ExitStatus(i32),
}

#[allow(dead_code)]
pub struct Range {
    start: i64,
    end: i64,
    current: i64,
}
