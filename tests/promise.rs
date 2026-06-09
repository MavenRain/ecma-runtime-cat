//! `Promise` built-in.

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
    let (value, _heap) =
        evaluate_program_with(&program, env, heap, Fuel::new(100_000)).map_err(Error::from)?;
    Ok(value)
}

fn fail(_msg: &'static str) -> Error {
    EngineError::Unsupported { feature: "test" }.into()
}

#[test]
fn promise_resolve_returns_resolved_promise() -> Result<(), Error> {
    // `Promise.resolve(7).then(cb)` synchronously fires cb with 7.
    let value = eval(
        "let captured = -1;
        Promise.resolve(7).then(v => { captured = v; });
        captured",
    )?;
    matches!(value, Value::Number(n) if (n - 7.0).abs() < 1e-9)
        .then_some(())
        .ok_or_else(|| fail("expected 7 captured from Promise.resolve"))
}

#[test]
fn promise_resolve_short_circuits_on_existing_promise() -> Result<(), Error> {
    // `Promise.resolve(p)` returns p directly when p is already a
    // Promise.  We can't easily compare identity through JS, so use
    // typeof + reaching through .then.
    let value = eval(
        "let captured = -1;
        const inner = Promise.resolve(42);
        Promise.resolve(inner).then(v => { captured = v; });
        captured",
    )?;
    matches!(value, Value::Number(n) if (n - 42.0).abs() < 1e-9)
        .then_some(())
        .ok_or_else(|| fail("expected 42 captured from short-circuit"))
}

#[test]
fn promise_reject_returns_rejected_promise() -> Result<(), Error> {
    let value = eval(
        "let caught = '';
        Promise.reject('boom').then(null, e => { caught = e; });
        caught",
    )?;
    matches!(value, Value::String(ref s) if s == "boom")
        .then_some(())
        .ok_or_else(|| fail("expected 'boom' caught from Promise.reject"))
}

#[test]
fn promise_all_with_settled_inputs_resolves_with_array() -> Result<(), Error> {
    // All inputs are non-Promise or already-Resolved; the resulting
    // array reflects each input's effective value, in order.
    let value = eval(
        "let captured = -1;
        Promise.all([1, Promise.resolve(2), 3]).then(arr => { captured = arr[1] * 10; });
        captured",
    )?;
    matches!(value, Value::Number(n) if (n - 20.0).abs() < 1e-9)
        .then_some(())
        .ok_or_else(|| fail("expected 20 from Promise.all([1, P(2), 3])[1] * 10"))
}

#[test]
fn promise_all_short_circuits_on_first_rejection() -> Result<(), Error> {
    let value = eval(
        "let caught = '';
        Promise.all([1, Promise.reject('bad'), 3]).then(null, e => { caught = e; });
        caught",
    )?;
    matches!(value, Value::String(ref s) if s == "bad")
        .then_some(())
        .ok_or_else(|| fail("expected 'bad' caught from short-circuit Promise.all"))
}

#[test]
fn promise_race_picks_first_settled() -> Result<(), Error> {
    let value = eval(
        "let captured = -1;
        Promise.race([Promise.resolve(99), Promise.resolve(2)]).then(v => { captured = v; });
        captured",
    )?;
    matches!(value, Value::Number(n) if (n - 99.0).abs() < 1e-9)
        .then_some(())
        .ok_or_else(|| fail("expected 99 (first input wins)"))
}

#[test]
fn promise_race_picks_first_rejected() -> Result<(), Error> {
    let value = eval(
        "let caught = '';
        Promise.race([Promise.reject('first'), Promise.resolve(2)]).then(null, e => { caught = e; });
        caught",
    )?;
    matches!(value, Value::String(ref s) if s == "first")
        .then_some(())
        .ok_or_else(|| fail("expected 'first' (rejected first input wins)"))
}

#[test]
fn promise_all_empty_array_resolves_to_empty_array() -> Result<(), Error> {
    let value = eval(
        "let captured = -1;
        Promise.all([]).then(arr => { captured = arr.length; });
        captured",
    )?;
    matches!(value, Value::Number(n) if (n - 0.0).abs() < 1e-9)
        .then_some(())
        .ok_or_else(|| fail("expected 0 from Promise.all([]).then(arr => arr.length)"))
}

#[test]
fn typeof_promise_resolve_is_object() -> Result<(), Error> {
    let value = eval("typeof Promise.resolve(1)")?;
    matches!(value, Value::String(ref s) if s == "object")
        .then_some(())
        .ok_or_else(|| fail("expected 'object'"))
}
