//! ECMAScript runtime: native built-ins for the [`boa_cat`] engine.
//!
//! Call [`install`] to fold the runtime's `(name, Value)` pairs into a
//! caller-supplied environment, or [`build_initial`] to get a fresh
//! environment and heap pre-populated with the runtime.
//!
//! # Example
//!
//! ```
//! # fn main() -> Result<(), ecma_runtime_cat::Error> {
//! use boa_cat::evaluate_program_with;
//! use boa_cat::fuel::Fuel;
//! use ecma_lex_cat::lex;
//! use ecma_parse_cat::parse_script;
//! use ecma_runtime_cat::build_initial;
//!
//! let source = "Math.floor(3.7) + Math.abs(-5)";
//! let tokens = lex(source).map_err(boa_cat::Error::from)?;
//! let program = parse_script(&tokens).map_err(boa_cat::Error::from)?;
//! let (env, heap) = build_initial();
//! let (value, _heap) = evaluate_program_with(&program, env, heap, Fuel::new(10_000))?;
//! assert_eq!(format!("{value}"), "8");
//! # Ok(())
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(clippy::similar_names)]
// IEEE-754 equality matches ECMA-262 semantics in our coercion helpers.
#![allow(clippy::float_cmp)]
// Numeric conversions follow ToInt32/ToUint32/length rules.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

pub mod coercion;
pub mod console;
pub mod error;
pub mod globals;
pub mod json;
pub mod math;
pub mod promise;

use boa_cat::Value;
use boa_cat::env::Env;
use boa_cat::heap::Heap;
use boa_cat::value::Cell;

pub use error::Error;

/// Build a fresh `(Env, Heap)` pre-populated with the runtime's built-in
/// globals (Math, console, JSON, parseInt, parseFloat, isNaN, isFinite,
/// Number, String, Boolean, plus undefined / NaN / Infinity).
#[must_use]
pub fn build_initial() -> (Env, Heap) {
    install(Env::empty(), Heap::new())
}

/// Fold the runtime's bindings into `env` and `heap`, returning the
/// extended pair.  Bindings already present in `env` are shadowed by the
/// runtime's entries (the runtime extends on top).
#[must_use]
pub fn install(env: Env, heap: Heap) -> (Env, Heap) {
    let (console_value, heap) = console::build(heap);
    let (math_value, heap) = math::build(heap);
    let (json_value, heap) = json::build(heap);
    let (promise_value, heap) = promise::build(heap);
    let bindings: Vec<(&str, Value)> = vec![
        ("undefined", Value::Undefined),
        ("NaN", Value::Number(f64::NAN)),
        ("Infinity", Value::Number(f64::INFINITY)),
        ("console", console_value),
        ("Math", math_value),
        ("JSON", json_value),
        ("Promise", promise_value),
        ("parseInt", Value::Native(globals::parse_int_impl)),
        ("parseFloat", Value::Native(globals::parse_float_impl)),
        ("isNaN", Value::Native(globals::is_nan_impl)),
        ("isFinite", Value::Native(globals::is_finite_impl)),
        ("Number", Value::Native(globals::number_impl)),
        ("String", Value::Native(globals::string_impl)),
        ("Boolean", Value::Native(globals::boolean_impl)),
    ];
    bindings
        .into_iter()
        .fold((env, heap), |(env, heap), (name, value)| {
            let (cell_id, heap) = heap.alloc_cell(Cell::new(value, false));
            (env.extend_cell(name, cell_id), heap)
        })
}
