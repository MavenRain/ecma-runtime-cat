//! Top-level global functions: `parseInt`, `parseFloat`, `isNaN`,
//! `isFinite`, `Number`, `String`, `Boolean`.

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};

use crate::coercion::{first_arg, to_number, to_string};

/// `parseInt(string, radix?)` native implementation.
///
/// # Errors
///
/// Never returns `Err`; bad input yields `Outcome::Normal(Value::Number(NaN))`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn parse_int_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let text = to_string(&first_arg(&args), &heap);
    let radix_arg = to_number(&args.get(1).cloned().unwrap_or(Value::Undefined));
    let parsed = resolve_radix(radix_arg).map_or(f64::NAN, |radix| parse_int_body(&text, radix));
    Ok((Outcome::Normal(Value::Number(parsed)), heap, fuel))
}

fn resolve_radix(radix_arg: f64) -> Option<u32> {
    if radix_arg.is_nan() || radix_arg == 0.0 {
        Some(10)
    } else if (2.0..=36.0).contains(&radix_arg) {
        // `radix_arg` is verified finite, non-negative, and within `2..=36`
        // before the conversion, so the `as u32` matches `TryFrom` exactly.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let r = radix_arg as u32;
        Some(r)
    } else {
        None
    }
}

fn parse_int_body(text: &str, radix: u32) -> f64 {
    let trimmed = text.trim();
    let (sign, body) = match trimmed.as_bytes().first() {
        Some(b'-') => (-1.0, &trimmed[1..]),
        Some(b'+') => (1.0, &trimmed[1..]),
        _other => (1.0, trimmed),
    };
    let body = body.trim_start_matches(|c: char| c == '0' && radix == 16);
    i64::from_str_radix(body, radix).map_or(f64::NAN, |n| sign * i64_to_f64(n))
}

/// Convert an `i64` to `f64` preserving precision when possible.
/// Values within `i32` range round-trip exactly through `f64::from`;
/// larger magnitudes go through a decimal round-trip, which yields the
/// same rounded value as `Number(string)` without a naked `as` cast.
fn i64_to_f64(n: i64) -> f64 {
    i32::try_from(n).map_or_else(
        |_| n.to_string().parse::<f64>().unwrap_or(f64::NAN),
        f64::from,
    )
}

/// `parseFloat(string)` native implementation.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn parse_float_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let text = to_string(&first_arg(&args), &heap);
    let trimmed = text.trim();
    let parsed = trimmed.parse::<f64>().unwrap_or(f64::NAN);
    Ok((Outcome::Normal(Value::Number(parsed)), heap, fuel))
}

/// `isNaN(value)` native implementation.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn is_nan_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Boolean(n.is_nan())), heap, fuel))
}

/// `isFinite(value)` native implementation.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn is_finite_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let n = to_number(&first_arg(&args));
    Ok((Outcome::Normal(Value::Boolean(n.is_finite())), heap, fuel))
}

/// `Number(value)` native implementation.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn number_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let value = if args.is_empty() {
        Value::Number(0.0)
    } else {
        Value::Number(to_number(&first_arg(&args)))
    };
    Ok((Outcome::Normal(value), heap, fuel))
}

/// `String(value)` native implementation.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn string_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let value = if args.is_empty() {
        Value::String(String::new())
    } else {
        Value::String(to_string(&first_arg(&args), &heap))
    };
    Ok((Outcome::Normal(value), heap, fuel))
}

/// `Boolean(value)` native implementation.
///
/// # Errors
///
/// Never returns `Err`.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn boolean_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let b = match first_arg(&args) {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => b,
        Value::Number(n) => !(n.is_nan() || n == 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Object(_) | Value::Function(_) | Value::Native(_) | Value::Promise(_) => true,
    };
    Ok((Outcome::Normal(Value::Boolean(b)), heap, fuel))
}
