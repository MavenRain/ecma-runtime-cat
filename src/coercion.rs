//! Coercion helpers shared by the built-ins.
//!
//! `boa-cat`'s coercion module is internal; this thin shim provides the
//! ECMA-262 abstract operations the runtime's native callables need.

use boa_cat::Value;
use boa_cat::heap::Heap;

/// `ToNumber(value)`.
#[must_use]
#[allow(clippy::match_same_arms)] // Undefined and Object/Function/Native both yield NaN through different ECMA-262 paths
pub fn to_number(value: &Value) -> f64 {
    match value {
        Value::Undefined => f64::NAN,
        Value::Null | Value::Boolean(false) => 0.0,
        Value::Boolean(true) => 1.0,
        Value::Number(n) => *n,
        Value::String(s) => string_to_number(s),
        Value::Object(_) | Value::Function(_) | Value::Native(_) | Value::Promise(_) => f64::NAN,
    }
}

fn string_to_number(s: &str) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        0.0
    } else {
        trimmed.parse::<f64>().unwrap_or(f64::NAN)
    }
}

/// `ToString(value)`.  Mirrors the engine's implementation so the runtime
/// has a self-contained renderer for messages, JSON, etc.
#[must_use]
pub fn to_string(value: &Value, _heap: &Heap) -> String {
    match value {
        Value::Undefined => "undefined".to_owned(),
        Value::Null => "null".to_owned(),
        Value::Boolean(true) => "true".to_owned(),
        Value::Boolean(false) => "false".to_owned(),
        Value::Number(n) => number_to_string(*n),
        Value::String(s) => s.clone(),
        Value::Object(id) => format!("[object Object#{}]", id.raw()),
        Value::Function(id) => format!("function fn#{}() {{ [native code] }}", id.raw()),
        Value::Native(_) => "function () { [native code] }".to_owned(),
        Value::Promise(_) => "[object Promise]".to_owned(),
    }
}

/// Render a number the way ECMA-262 `ToString(Number)` does: `"NaN"`,
/// `"Infinity"`, `"-Infinity"`, `"0"`, or the shortest decimal.
#[must_use]
pub fn number_to_string(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_owned()
    } else if n.is_infinite() {
        if n.is_sign_negative() {
            "-Infinity".to_owned()
        } else {
            "Infinity".to_owned()
        }
    } else if n == 0.0 {
        "0".to_owned()
    } else {
        format!("{n}")
    }
}

/// Convenience: pull the first argument or `Undefined`.
#[must_use]
pub fn first_arg(args: &[Value]) -> Value {
    args.first().cloned().unwrap_or(Value::Undefined)
}

/// Convenience: pull the `idx`-th argument or `Undefined`.
#[must_use]
pub fn arg_at(args: &[Value], idx: usize) -> Value {
    args.get(idx).cloned().unwrap_or(Value::Undefined)
}
