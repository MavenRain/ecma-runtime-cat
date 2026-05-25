# ecma-runtime-cat

ECMAScript runtime built-ins (`console.*`, `Math.*`, `JSON.*`, `parseInt`, `parseFloat`, `isNaN`, `isFinite`, `Number`, `String`, `Boolean`) for the [`boa-cat`](https://crates.io/crates/boa-cat) engine.

`ecma-runtime-cat` is the runtime layer of a `comp-cat-rs` reformulation of a JavaScript stack targeting Tauri integration.  It registers a set of Rust-implemented `NativeFn` callables into the engine's initial environment so scripts can call standard globals.  The same framework constraints apply: no `mut`, no `Rc`/`Arc`, no interior mutability, no panics, exhaustive matches, static dispatch.

## Example

```rust
use boa_cat::evaluate_program_with;
use boa_cat::fuel::Fuel;
use ecma_lex_cat::lex;
use ecma_parse_cat::parse_script;
use ecma_runtime_cat::{Error, build_initial};

fn main() -> Result<(), Error> {
    let source = "Math.floor(3.7) + Math.abs(-5)";
    let tokens = lex(source).map_err(boa_cat::Error::from)?;
    let program = parse_script(&tokens).map_err(boa_cat::Error::from)?;
    let (env, heap) = build_initial();
    let (value, _heap) = evaluate_program_with(&program, env, heap, Fuel::new(10_000))?;
    assert_eq!(format!("{value}"), "8");
    Ok(())
}
```

## v0 scope

- `console.log` / `console.error` / `console.warn`.
- `Math.{abs, floor, ceil, round, sqrt, pow, min, max, log, exp, sin, cos, tan, atan, atan2, random}` and constants `Math.{PI, E, LN2, LN10, SQRT2}`.
- `JSON.stringify` (basic; no `replacer` or `space` argument).
- Globals: `parseInt`, `parseFloat`, `isNaN`, `isFinite`, `Number`, `String`, `Boolean`, plus `undefined`, `NaN`, `Infinity`.

## Deferred to v0.2+

- `JSON.parse`.
- `Object.{keys, values, entries, assign}`.
- `Array.{isArray, from, of}` and `Array.prototype.*`.
- `String.prototype.*` (requires prototype-chain support in `boa-cat`).
- `Date`, `Promise`, `Symbol`, `Error` constructor.

## Design

Each built-in is a `NativeFn` (`fn(Vec<Value>, Value, Heap, Fuel) -> EvalResult`).  [`build_initial()`](https://docs.rs/ecma-runtime-cat/latest/ecma_runtime_cat/fn.build_initial.html) returns a pre-populated `(Env, Heap)`; [`install(env, heap)`](https://docs.rs/ecma-runtime-cat/latest/ecma_runtime_cat/fn.install.html) folds the runtime onto a caller-supplied env+heap.  Downstream callers thread these into `boa_cat::evaluate_program_with`.

The runtime never panics.  Bad inputs surface as `Outcome::Throw(Value::String("TypeError: ..."))` so script-level `try`/`catch` can recover, or as `Value::Number(NaN)` where ECMA-262 specifies that fallback (`parseInt`, `Number`, arithmetic on objects).

## License

MIT OR Apache-2.0
