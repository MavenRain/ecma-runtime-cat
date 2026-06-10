//! `JSON.parse` (v0.3.2).

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
fn parses_number_literal() -> Result<(), Error> {
    approx(&eval("JSON.parse('42')")?, 42.0)
}

#[test]
fn parses_negative_number() -> Result<(), Error> {
    approx(&eval("JSON.parse('-3.5')")?, -3.5)
}

#[test]
fn parses_scientific_notation() -> Result<(), Error> {
    approx(&eval("JSON.parse('1.5e2')")?, 150.0)
}

#[test]
fn parses_boolean_true() -> Result<(), Error> {
    matches!(eval("JSON.parse('true')")?, Value::Boolean(true))
        .then_some(())
        .ok_or_else(|| fail("expected true"))
}

#[test]
fn parses_boolean_false() -> Result<(), Error> {
    matches!(eval("JSON.parse('false')")?, Value::Boolean(false))
        .then_some(())
        .ok_or_else(|| fail("expected false"))
}

#[test]
fn parses_null() -> Result<(), Error> {
    matches!(eval("JSON.parse('null')")?, Value::Null)
        .then_some(())
        .ok_or_else(|| fail("expected null"))
}

#[test]
fn parses_string() -> Result<(), Error> {
    matches!(eval("JSON.parse('\"hello\"')")?, Value::String(ref s) if s == "hello")
        .then_some(())
        .ok_or_else(|| fail("expected \"hello\""))
}

#[test]
fn parses_string_with_escapes() -> Result<(), Error> {
    let value = eval(r#"JSON.parse('"a\\nb"')"#)?;
    matches!(value, Value::String(ref s) if s == "a\nb")
        .then_some(())
        .ok_or_else(|| fail("expected \"a\\nb\""))
}

#[test]
fn parses_string_with_unicode_escape() -> Result<(), Error> {
    let value = eval(r#"JSON.parse('"\\u0041"')"#)?;
    matches!(value, Value::String(ref s) if s == "A")
        .then_some(())
        .ok_or_else(|| fail("expected \"A\""))
}

#[test]
fn parses_empty_array_length_zero() -> Result<(), Error> {
    approx(&eval("JSON.parse('[]').length")?, 0.0)
}

#[test]
fn parses_array_round_trip() -> Result<(), Error> {
    approx(&eval("JSON.parse('[1, 2, 3]')[1]")?, 2.0)
}

#[test]
fn parses_array_length() -> Result<(), Error> {
    approx(&eval("JSON.parse('[1, 2, 3]').length")?, 3.0)
}

#[test]
fn parses_object_string_value() -> Result<(), Error> {
    let value = eval(r#"JSON.parse('{"name": "Ada"}').name"#)?;
    matches!(value, Value::String(ref s) if s == "Ada")
        .then_some(())
        .ok_or_else(|| fail("expected \"Ada\""))
}

#[test]
fn parses_object_number_value() -> Result<(), Error> {
    approx(&eval(r#"JSON.parse('{"age": 36}').age"#)?, 36.0)
}

#[test]
fn parses_nested_object() -> Result<(), Error> {
    approx(&eval(r#"JSON.parse('{"a": {"b": {"c": 7}}}').a.b.c"#)?, 7.0)
}

#[test]
fn parses_array_of_objects() -> Result<(), Error> {
    approx(
        &eval(r#"JSON.parse('[{"v": 1}, {"v": 2}, {"v": 3}]')[2].v"#)?,
        3.0,
    )
}

#[test]
fn round_trips_through_stringify() -> Result<(), Error> {
    approx(&eval(r#"JSON.parse(JSON.stringify({"x": 10})).x"#)?, 10.0)
}

#[test]
fn syntax_error_on_unterminated_string() -> Result<(), Error> {
    // The parse throws a SyntaxError; the surrounding try/catch
    // converts it to a recovered string value.
    let value = eval(
        "let caught = '';
        try { JSON.parse('\"oops'); } catch (e) { caught = e; }
        caught",
    )?;
    matches!(value, Value::String(ref s) if s.starts_with("SyntaxError:"))
        .then_some(())
        .ok_or_else(|| fail("expected SyntaxError prefix"))
}

#[test]
fn syntax_error_on_trailing_content() -> Result<(), Error> {
    let value = eval(
        "let caught = '';
        try { JSON.parse('1 garbage'); } catch (e) { caught = e; }
        caught",
    )?;
    matches!(value, Value::String(ref s) if s.starts_with("SyntaxError:"))
        .then_some(())
        .ok_or_else(|| fail("expected SyntaxError prefix for trailing content"))
}

#[test]
fn syntax_error_on_empty_input() -> Result<(), Error> {
    let value = eval(
        "let caught = '';
        try { JSON.parse(''); } catch (e) { caught = e; }
        caught",
    )?;
    matches!(value, Value::String(ref s) if s.starts_with("SyntaxError:"))
        .then_some(())
        .ok_or_else(|| fail("expected SyntaxError prefix for empty input"))
}
