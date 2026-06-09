//! `JSON.stringify` native callable.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::{first_arg, number_to_string};

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn stringify_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let value = first_arg(&args);
    let outcome = stringify_value(&value, &heap).map_or(Outcome::Normal(Value::Undefined), |s| {
        Outcome::Normal(Value::String(s))
    });
    Ok((outcome, heap, fuel))
}

fn stringify_value(value: &Value, heap: &Heap) -> Option<String> {
    match value {
        Value::Undefined | Value::Function(_) | Value::Native(_) | Value::Promise(_) => None,
        Value::Null => Some("null".to_owned()),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Number(n) => {
            if n.is_finite() {
                Some(number_to_string(*n))
            } else {
                Some("null".to_owned())
            }
        }
        Value::String(s) => Some(stringify_string(s)),
        Value::Object(id) => heap.object(*id).map(|obj| stringify_object(obj, heap)),
    }
}

fn stringify_string(s: &str) -> String {
    let escaped: String = s
        .chars()
        .map(|c| match c {
            '"' => "\\\"".to_owned(),
            '\\' => "\\\\".to_owned(),
            '\n' => "\\n".to_owned(),
            '\r' => "\\r".to_owned(),
            '\t' => "\\t".to_owned(),
            ch if u32::from(ch) < 0x20 => format!("\\u{:04x}", u32::from(ch)),
            ch => ch.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}

fn stringify_object(obj: &Object, heap: &Heap) -> String {
    if is_array_object(obj) {
        stringify_array(obj, heap)
    } else {
        stringify_plain_object(obj, heap)
    }
}

fn is_array_object(obj: &Object) -> bool {
    obj.get("length")
        .is_some_and(|v| matches!(v, Value::Number(_)))
        && obj
            .properties()
            .keys()
            .all(|k| k == "length" || k.parse::<u32>().is_ok())
}

fn stringify_array(obj: &Object, heap: &Heap) -> String {
    // ECMA-262 array length is a ToUint32-bounded value; the `as u32`
    // after the finite/non-negative guard mirrors that bounded conversion.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let length = obj
        .get("length")
        .and_then(|v| match v {
            Value::Number(n) if n.is_finite() && *n >= 0.0 => Some(*n as u32),
            _other => None,
        })
        .unwrap_or(0);
    let body: Vec<String> = (0..length)
        .map(|i| {
            obj.get(&format!("{i}")).map_or("null".to_owned(), |v| {
                stringify_value(v, heap).unwrap_or_else(|| "null".to_owned())
            })
        })
        .collect();
    format!("[{}]", body.join(","))
}

fn stringify_plain_object(obj: &Object, heap: &Heap) -> String {
    let body: Vec<String> = obj
        .properties()
        .iter()
        .filter_map(|(k, v)| {
            stringify_value(v, heap).map(|rendered| format!("{}:{rendered}", stringify_string(k)))
        })
        .collect();
    format!("{{{}}}", body.join(","))
}

/// Build the `JSON` object and allocate it on the heap.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let mut props = BTreeMap::new();
    let _ = props.insert("stringify".to_owned(), Value::Native(stringify_impl));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}
