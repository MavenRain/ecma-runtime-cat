//! `Array` global static methods (v0.3.3): `isArray`.
//!
//! Detects the array shape this runtime represents (a
//! `Value::Object` whose own data-property keys are all numeric or
//! `length`, with `length` a non-negative number).  This mirrors
//! `JSON.stringify`'s `is_array_object` discriminator and the
//! existing array-literal evaluator's output shape.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::first_arg;

/// Build the `Array` global and allocate it on the heap.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let mut props = BTreeMap::new();
    let _ = props.insert("isArray".to_owned(), Value::Native(is_array_impl));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}

/// `Array.isArray(value)` -- returns `true` when `value` is an
/// `Object` whose properties match the runtime's array shape
/// (numeric keys plus `length`), `false` otherwise.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn is_array_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let arg = first_arg(&args);
    let is_array = match arg {
        Value::Object(id) => heap.object(id).is_some_and(looks_like_array),
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Function(_)
        | Value::Native(_)
        | Value::Promise(_) => false,
    };
    Ok((Outcome::Normal(Value::Boolean(is_array)), heap, fuel))
}

fn looks_like_array(obj: &Object) -> bool {
    obj.get("length")
        .is_some_and(|v| matches!(v, Value::Number(_)))
        && obj
            .properties()
            .keys()
            .all(|k| k == "length" || k.parse::<u32>().is_ok())
}
