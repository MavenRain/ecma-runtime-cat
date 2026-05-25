//! Top-level globals: parseInt, parseFloat, isNaN, isFinite, Number, String, Boolean.

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

fn assert_number(actual: &Value, expected: f64) -> Result<(), Error> {
    matches!(actual, Value::Number(n) if (n - expected).abs() < 1e-9)
        .then_some(())
        .ok_or(Error::Engine(EngineError::UncaughtException {
            rendered: format!("expected Number({expected}), got {actual:?}"),
        }))
}

fn assert_boolean(actual: &Value, expected: bool) -> Result<(), Error> {
    matches!(actual, Value::Boolean(b) if *b == expected)
        .then_some(())
        .ok_or(Error::Engine(EngineError::UncaughtException {
            rendered: format!("expected Boolean({expected}), got {actual:?}"),
        }))
}

fn assert_string(actual: &Value, expected: &str) -> Result<(), Error> {
    matches!(actual, Value::String(s) if s == expected)
        .then_some(())
        .ok_or(Error::Engine(EngineError::UncaughtException {
            rendered: format!("expected String({expected:?}), got {actual:?}"),
        }))
}

#[test]
fn parse_int_decimal() -> Result<(), Error> {
    assert_number(&eval("parseInt(\"42\")")?, 42.0)
}

#[test]
fn parse_int_hex() -> Result<(), Error> {
    assert_number(&eval("parseInt(\"1f\", 16)")?, 31.0)
}

#[test]
fn parse_float_decimal() -> Result<(), Error> {
    assert_number(&eval("parseFloat(\"2.5\")")?, 2.5)
}

#[test]
fn is_nan_global() -> Result<(), Error> {
    assert_boolean(&eval("isNaN(NaN)")?, true)?;
    assert_boolean(&eval("isNaN(1)")?, false)
}

#[test]
fn is_finite_global() -> Result<(), Error> {
    assert_boolean(&eval("isFinite(1)")?, true)?;
    assert_boolean(&eval("isFinite(Infinity)")?, false)?;
    assert_boolean(&eval("isFinite(NaN)")?, false)
}

#[test]
fn number_constructor() -> Result<(), Error> {
    assert_number(&eval("Number(\"42\")")?, 42.0)?;
    assert_number(&eval("Number()")?, 0.0)?;
    assert_number(&eval("Number(true)")?, 1.0)
}

#[test]
fn string_constructor() -> Result<(), Error> {
    assert_string(&eval("String(42)")?, "42")?;
    assert_string(&eval("String(true)")?, "true")?;
    assert_string(&eval("String()")?, "")
}

#[test]
fn boolean_constructor() -> Result<(), Error> {
    assert_boolean(&eval("Boolean(0)")?, false)?;
    assert_boolean(&eval("Boolean(1)")?, true)?;
    assert_boolean(&eval("Boolean(\"\")")?, false)?;
    assert_boolean(&eval("Boolean(\"x\")")?, true)
}
