//! `Math.*` native callables and constants.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::{first_arg, to_number};

fn unary(f: fn(f64) -> f64) -> impl Fn(Vec<Value>, Value, Heap, Fuel) -> EvalResult {
    move |args, _this, heap, fuel| {
        let n = to_number(&first_arg(&args));
        Ok((Outcome::Normal(Value::Number(f(n))), heap, fuel))
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn abs_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.abs())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn floor_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.floor())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn ceil_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.ceil())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn round_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    // ECMA-262: Math.round rounds half away from positive infinity, i.e.
    // 0.5 -> 1, -0.5 -> 0.  Rust's round() rounds half away from zero,
    // which differs on negatives.
    let rounded = (n + 0.5).floor();
    Ok((Outcome::Normal(Value::Number(rounded)), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn sqrt_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.sqrt())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn pow_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let base = to_number(&first_arg(&args));
    let exp = to_number(&args.get(1).cloned().unwrap_or(Value::Undefined));
    Ok((Outcome::Normal(Value::Number(base.powf(exp))), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn min_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let result = args.iter().map(to_number).fold(f64::INFINITY, |acc, n| {
        if n.is_nan() { f64::NAN } else { acc.min(n) }
    });
    Ok((Outcome::Normal(Value::Number(result)), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn max_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let result = args
        .iter()
        .map(to_number)
        .fold(f64::NEG_INFINITY, |acc, n| {
            if n.is_nan() { f64::NAN } else { acc.max(n) }
        });
    Ok((Outcome::Normal(Value::Number(result)), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn log_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.ln())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn exp_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.exp())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn sin_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.sin())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn cos_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.cos())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn tan_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.tan())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn atan_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Number(n.atan())), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn atan2_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let y = to_number(&first_arg(&args));
    let x = to_number(&args.get(1).cloned().unwrap_or(Value::Undefined));
    Ok((Outcome::Normal(Value::Number(y.atan2(x))), heap, fuel))
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn random_impl(_args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    // Deterministic placeholder for v0 (we have no entropy source we want
    // to introduce; embedders that need real entropy can override).
    Ok((Outcome::Normal(Value::Number(0.0)), heap, fuel))
}

/// Build the `Math` object and allocate it on the heap.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let _ = unary; // silence unused-import warning; kept for the v0.2 generalisation
    let mut props = BTreeMap::new();
    let _ = props.insert("abs".to_owned(), Value::Native(abs_impl));
    let _ = props.insert("floor".to_owned(), Value::Native(floor_impl));
    let _ = props.insert("ceil".to_owned(), Value::Native(ceil_impl));
    let _ = props.insert("round".to_owned(), Value::Native(round_impl));
    let _ = props.insert("sqrt".to_owned(), Value::Native(sqrt_impl));
    let _ = props.insert("pow".to_owned(), Value::Native(pow_impl));
    let _ = props.insert("min".to_owned(), Value::Native(min_impl));
    let _ = props.insert("max".to_owned(), Value::Native(max_impl));
    let _ = props.insert("log".to_owned(), Value::Native(log_impl));
    let _ = props.insert("exp".to_owned(), Value::Native(exp_impl));
    let _ = props.insert("sin".to_owned(), Value::Native(sin_impl));
    let _ = props.insert("cos".to_owned(), Value::Native(cos_impl));
    let _ = props.insert("tan".to_owned(), Value::Native(tan_impl));
    let _ = props.insert("atan".to_owned(), Value::Native(atan_impl));
    let _ = props.insert("atan2".to_owned(), Value::Native(atan2_impl));
    let _ = props.insert("random".to_owned(), Value::Native(random_impl));
    let _ = props.insert("PI".to_owned(), Value::Number(std::f64::consts::PI));
    let _ = props.insert("E".to_owned(), Value::Number(std::f64::consts::E));
    let _ = props.insert("LN2".to_owned(), Value::Number(std::f64::consts::LN_2));
    let _ = props.insert("LN10".to_owned(), Value::Number(std::f64::consts::LN_10));
    let _ = props.insert("SQRT2".to_owned(), Value::Number(std::f64::consts::SQRT_2));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}
