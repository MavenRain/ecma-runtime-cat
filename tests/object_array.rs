//! `Object` static methods + `Array.isArray` (v0.3.3).

use boa_cat::evaluate_program_with;
use boa_cat::fuel::Fuel;
use boa_cat::{Error as EngineError, Value};
use ecma_lex_cat::lex;
use ecma_parse_cat::parse_script;
use ecma_runtime_cat::{Error, build_initial};

fn eval(source: &str) -> Result<Value, Error> {
    let tokens = lex(source).map_err(EngineError::from)?;
    let program = parse_script(&tokens).map_err(EngineError::from)?;
    let (env, heap) = build_initial();
    let (value, _heap) = evaluate_program_with(&program, env, heap, Fuel::new(10_000))?;
    Ok(value)
}

fn approx(actual: &Value, expected: f64) -> Result<(), Error> {
    matches!(actual, Value::Number(n) if (n - expected).abs() < 1e-9)
        .then_some(())
        .ok_or(Error::Engine(EngineError::UncaughtException {
            rendered: format!("expected Number({expected}), got {actual:?}"),
        }))
}

fn fail(message: &'static str) -> Error {
    Error::Engine(EngineError::UncaughtException {
        rendered: message.to_owned(),
    })
}

#[test]
fn object_keys_length_matches_property_count() -> Result<(), Error> {
    approx(&eval("Object.keys({a: 1, b: 2, c: 3}).length")?, 3.0)
}

#[test]
fn object_keys_returns_strings() -> Result<(), Error> {
    matches!(eval("Object.keys({hello: 1})[0]")?, Value::String(ref s) if s == "hello")
        .then_some(())
        .ok_or_else(|| fail("expected \"hello\" as the first key"))
}

#[test]
fn object_keys_empty_object_returns_empty_array() -> Result<(), Error> {
    approx(&eval("Object.keys({}).length")?, 0.0)
}

#[test]
fn object_keys_on_array_excludes_length() -> Result<(), Error> {
    // The runtime represents arrays as objects with numeric keys
    // + `length`.  `Object.keys` of an array should surface only
    // the numeric keys.
    approx(&eval("Object.keys([10, 20, 30]).length")?, 3.0)
}

#[test]
fn object_values_returns_data_values_in_order() -> Result<(), Error> {
    approx(&eval("Object.values({a: 10, b: 20, c: 30})[1]")?, 20.0)
}

#[test]
fn object_values_length_matches_property_count() -> Result<(), Error> {
    approx(&eval("Object.values({a: 1, b: 2, c: 3}).length")?, 3.0)
}

#[test]
fn object_entries_returns_pairs() -> Result<(), Error> {
    matches!(eval("Object.entries({hi: 42})[0][0]")?, Value::String(ref s) if s == "hi")
        .then_some(())
        .ok_or_else(|| fail("expected \"hi\" as the key of the first entry"))
}

#[test]
fn object_entries_value_at_second_slot() -> Result<(), Error> {
    approx(&eval("Object.entries({hi: 42})[0][1]")?, 42.0)
}

#[test]
fn object_entries_length_matches_property_count() -> Result<(), Error> {
    approx(&eval("Object.entries({a: 1, b: 2, c: 3}).length")?, 3.0)
}

#[test]
fn object_assign_copies_sources_into_target() -> Result<(), Error> {
    approx(
        &eval("const t = { a: 1 }; Object.assign(t, { b: 2 }, { c: 3 }); t.a + t.b + t.c")?,
        6.0,
    )
}

#[test]
fn object_assign_later_source_wins() -> Result<(), Error> {
    approx(
        &eval("const t = { x: 1 }; Object.assign(t, { x: 2 }, { x: 3 }); t.x")?,
        3.0,
    )
}

#[test]
fn object_assign_returns_target() -> Result<(), Error> {
    // The return value is the target itself; reading a property
    // off the result confirms identity.
    approx(
        &eval("const t = {}; const r = Object.assign(t, { x: 5 }); r.x")?,
        5.0,
    )
}

#[test]
fn object_assign_with_no_sources_returns_target_unchanged() -> Result<(), Error> {
    approx(&eval("const t = { a: 7 }; Object.assign(t); t.a")?, 7.0)
}

#[test]
fn array_is_array_true_for_array_literal() -> Result<(), Error> {
    matches!(eval("Array.isArray([])")?, Value::Boolean(true))
        .then_some(())
        .ok_or_else(|| fail("expected true for []"))
}

#[test]
fn array_is_array_true_for_populated_array() -> Result<(), Error> {
    matches!(eval("Array.isArray([1, 2, 3])")?, Value::Boolean(true))
        .then_some(())
        .ok_or_else(|| fail("expected true for [1, 2, 3]"))
}

#[test]
fn array_is_array_false_for_plain_object() -> Result<(), Error> {
    matches!(eval("Array.isArray({})")?, Value::Boolean(false))
        .then_some(())
        .ok_or_else(|| fail("expected false for {}"))
}

#[test]
fn array_is_array_false_for_object_with_length() -> Result<(), Error> {
    // An object with length but with non-numeric keys isn't an
    // array.
    matches!(
        eval("Array.isArray({ length: 3, foo: 'bar' })")?,
        Value::Boolean(false)
    )
    .then_some(())
    .ok_or_else(|| fail("expected false for { length, foo }"))
}

#[test]
fn array_is_array_false_for_primitives() -> Result<(), Error> {
    matches!(eval("Array.isArray(42)")?, Value::Boolean(false))
        .then_some(())
        .ok_or_else(|| fail("expected false for number"))
}

#[test]
fn array_is_array_false_for_string() -> Result<(), Error> {
    matches!(eval("Array.isArray('hello')")?, Value::Boolean(false))
        .then_some(())
        .ok_or_else(|| fail("expected false for string"))
}

#[test]
fn array_is_array_true_for_json_parsed_array() -> Result<(), Error> {
    matches!(
        eval("Array.isArray(JSON.parse('[1, 2]'))")?,
        Value::Boolean(true)
    )
    .then_some(())
    .ok_or_else(|| fail("expected true for JSON.parse('[1, 2]')"))
}
