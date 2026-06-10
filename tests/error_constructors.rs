//! Standard `Error` constructors (v0.3.5).

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

fn fail(message: &'static str) -> Error {
    Error::Engine(EngineError::UncaughtException {
        rendered: message.to_owned(),
    })
}

#[test]
fn error_carries_message_from_constructor_arg() -> Result<(), Error> {
    let value = eval("Error('oops').message")?;
    matches!(value, Value::String(ref s) if s == "oops")
        .then_some(())
        .ok_or_else(|| fail("expected message 'oops'"))
}

#[test]
fn error_carries_name() -> Result<(), Error> {
    let value = eval("Error('oops').name")?;
    matches!(value, Value::String(ref s) if s == "Error")
        .then_some(())
        .ok_or_else(|| fail("expected name 'Error'"))
}

#[test]
fn error_with_no_arg_has_empty_message() -> Result<(), Error> {
    let value = eval("Error().message")?;
    matches!(value, Value::String(ref s) if s.is_empty())
        .then_some(())
        .ok_or_else(|| fail("expected empty message"))
}

#[test]
fn error_via_new_keyword_returns_same_shape() -> Result<(), Error> {
    let value = eval("(new Error('boom')).message")?;
    matches!(value, Value::String(ref s) if s == "boom")
        .then_some(())
        .ok_or_else(|| fail("expected message 'boom' via `new`"))
}

#[test]
fn error_message_coerces_non_string_arg() -> Result<(), Error> {
    // Per spec, `Error(42).message` is `"42"`.
    let value = eval("Error(42).message")?;
    matches!(value, Value::String(ref s) if s == "42")
        .then_some(())
        .ok_or_else(|| fail("expected message '42'"))
}

#[test]
fn type_error_carries_correct_name() -> Result<(), Error> {
    let value = eval("TypeError('bad').name")?;
    matches!(value, Value::String(ref s) if s == "TypeError")
        .then_some(())
        .ok_or_else(|| fail("expected name 'TypeError'"))
}

#[test]
fn range_error_carries_correct_name() -> Result<(), Error> {
    let value = eval("RangeError().name")?;
    matches!(value, Value::String(ref s) if s == "RangeError")
        .then_some(())
        .ok_or_else(|| fail("expected name 'RangeError'"))
}

#[test]
fn syntax_error_carries_correct_name() -> Result<(), Error> {
    let value = eval("SyntaxError().name")?;
    matches!(value, Value::String(ref s) if s == "SyntaxError")
        .then_some(())
        .ok_or_else(|| fail("expected name 'SyntaxError'"))
}

#[test]
fn reference_error_carries_correct_name() -> Result<(), Error> {
    let value = eval("ReferenceError().name")?;
    matches!(value, Value::String(ref s) if s == "ReferenceError")
        .then_some(())
        .ok_or_else(|| fail("expected name 'ReferenceError'"))
}

#[test]
fn eval_error_carries_correct_name() -> Result<(), Error> {
    let value = eval("EvalError().name")?;
    matches!(value, Value::String(ref s) if s == "EvalError")
        .then_some(())
        .ok_or_else(|| fail("expected name 'EvalError'"))
}

#[test]
fn uri_error_carries_correct_name() -> Result<(), Error> {
    let value = eval("URIError().name")?;
    matches!(value, Value::String(ref s) if s == "URIError")
        .then_some(())
        .ok_or_else(|| fail("expected name 'URIError'"))
}

#[test]
fn throw_error_caught_as_object_with_message() -> Result<(), Error> {
    // Real-world usage: throw and catch through try/catch, reading
    // the error's `.message` in the catch block.
    let value = eval(
        "let caught = '';
        try { throw new Error('something broke'); }
        catch (e) { caught = e.message; }
        caught",
    )?;
    matches!(value, Value::String(ref s) if s == "something broke")
        .then_some(())
        .ok_or_else(|| fail("expected caught message 'something broke'"))
}

#[test]
fn throw_type_error_caught_with_name() -> Result<(), Error> {
    let value = eval(
        "let name = '';
        try { throw new TypeError('bad type'); }
        catch (e) { name = e.name; }
        name",
    )?;
    matches!(value, Value::String(ref s) if s == "TypeError")
        .then_some(())
        .ok_or_else(|| fail("expected caught name 'TypeError'"))
}
