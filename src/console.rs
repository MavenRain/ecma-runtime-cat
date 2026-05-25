//! `console.log` / `console.error` / `console.warn` native callables.
//!
//! Each prints its space-separated arguments to stderr (info/warn) or
//! stdout (log) and returns `undefined`.  No formatting placeholders are
//! interpreted; the engine's `to_string` rendering applies to each
//! argument.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::to_string;

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn log_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    println!("{}", render_args(&args, &heap));
    Ok((Outcome::Normal(Value::Undefined), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn error_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    eprintln!("{}", render_args(&args, &heap));
    Ok((Outcome::Normal(Value::Undefined), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn warn_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    eprintln!("{}", render_args(&args, &heap));
    Ok((Outcome::Normal(Value::Undefined), heap, fuel))
}

fn render_args(args: &[Value], heap: &Heap) -> String {
    args.iter()
        .map(|v| to_string(v, heap))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build the `console` object and allocate it on the heap.  Returns the
/// new heap and the [`Value::Object`] handle to bind in the global env.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let mut props = BTreeMap::new();
    let _ = props.insert("log".to_owned(), Value::Native(log_impl));
    let _ = props.insert("error".to_owned(), Value::Native(error_impl));
    let _ = props.insert("warn".to_owned(), Value::Native(warn_impl));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}
