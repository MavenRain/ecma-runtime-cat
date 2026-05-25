//! Math built-ins.

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

#[test]
fn math_floor() -> Result<(), Error> {
    approx(&eval("Math.floor(3.7)")?, 3.0)
}

#[test]
fn math_ceil() -> Result<(), Error> {
    approx(&eval("Math.ceil(3.2)")?, 4.0)
}

#[test]
fn math_abs() -> Result<(), Error> {
    approx(&eval("Math.abs(-5)")?, 5.0)
}

#[test]
fn math_sqrt() -> Result<(), Error> {
    approx(&eval("Math.sqrt(16)")?, 4.0)
}

#[test]
fn math_pow() -> Result<(), Error> {
    approx(&eval("Math.pow(2, 10)")?, 1024.0)
}

#[test]
fn math_min_max() -> Result<(), Error> {
    approx(&eval("Math.min(1, 2, 3)")?, 1.0)?;
    approx(&eval("Math.max(1, 5, 3)")?, 5.0)
}

#[test]
fn math_constants() -> Result<(), Error> {
    approx(&eval("Math.PI")?, std::f64::consts::PI)?;
    approx(&eval("Math.E")?, std::f64::consts::E)
}

#[test]
fn math_round_half_to_positive_infinity() -> Result<(), Error> {
    approx(&eval("Math.round(0.5)")?, 1.0)?;
    approx(&eval("Math.round(-0.5)")?, 0.0)
}

#[test]
fn math_trig_round_trip() -> Result<(), Error> {
    approx(&eval("Math.sin(0)")?, 0.0)?;
    approx(&eval("Math.cos(0)")?, 1.0)
}
