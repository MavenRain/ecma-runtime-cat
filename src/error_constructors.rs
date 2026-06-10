//! Standard `Error` constructors (v0.3.5): `Error`, `TypeError`,
//! `RangeError`, `SyntaxError`, `EvalError`, `ReferenceError`,
//! `URIError`.
//!
//! Each constructor produces a `Value::Object` carrying two data
//! properties: `name` (the constructor's name, e.g. `"TypeError"`)
//! and `message` (the first argument coerced via `to_string`, or
//! `""` when called with no arguments).  Both `Error("msg")` and
//! `new Error("msg")` produce the same shape: the engine's `new`
//! path discards the empty `this` Object and uses the returned
//! Object directly because every constructor here always returns
//! an Object (per the `eval_new::construct` semantics in
//! boa-cat 0.7).
//!
//! `instanceof Error` semantics are NOT implemented because this
//! engine has no prototype chain; the only observable Error
//! shape is the `name` / `message` property pair, which matches
//! how most error-handling code reads errors.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::{first_arg, to_string};

/// `Error(message?)` -- generic base error.
///
/// # Errors
///
/// Never returns `Err`; bad inputs surface as a coerced string
/// message on the returned Error object.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("Error", &args, heap, fuel))
}

/// `TypeError(message?)`.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn type_error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("TypeError", &args, heap, fuel))
}

/// `RangeError(message?)`.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn range_error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("RangeError", &args, heap, fuel))
}

/// `SyntaxError(message?)`.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn syntax_error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("SyntaxError", &args, heap, fuel))
}

/// `ReferenceError(message?)`.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn reference_error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("ReferenceError", &args, heap, fuel))
}

/// `EvalError(message?)` -- legacy spec constructor kept for
/// completeness; engines typically never throw it themselves.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn eval_error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("EvalError", &args, heap, fuel))
}

/// `URIError(message?)`.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn uri_error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    Ok(build_error("URIError", &args, heap, fuel))
}

fn build_error(name: &str, args: &[Value], heap: Heap, fuel: Fuel) -> (Outcome, Heap, Fuel) {
    let message_value = first_arg(args);
    let message = match message_value {
        Value::Undefined => String::new(),
        Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Object(_)
        | Value::Function(_)
        | Value::Native(_)
        | Value::Promise(_) => to_string(&message_value, &heap),
    };
    let mut props = BTreeMap::new();
    let _ = props.insert("name".to_owned(), Value::String(name.to_owned()));
    let _ = props.insert("message".to_owned(), Value::String(message));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Outcome::Normal(Value::Object(id)), heap, fuel)
}
