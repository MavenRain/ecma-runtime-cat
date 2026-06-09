//! `Promise` global: `resolve(v)`, `reject(v)`, `all(arr)`,
//! `race(arr)` (async track chunk 4 of 5-7).
//!
//! Each static method allocates a fresh promise on the heap via
//! `boa_cat::heap::Heap::alloc_promise(state)` and returns the
//! corresponding `Value::Promise(id)`.
//!
//! - `Promise.resolve(v)`: if `v` is already a Promise, returns it
//!   unchanged (per spec); otherwise wraps it in a `Resolved(v)`.
//! - `Promise.reject(v)`: always wraps `v` in a `Rejected(v)`, even
//!   when `v` is itself a Promise (per spec).
//! - `Promise.all(arr)`: when every element is already-Resolved or
//!   non-Promise, returns a Promise resolved with an array of
//!   element values; the first Rejected element short-circuits the
//!   result to that rejection.  Pending inputs throw a `TypeError` --
//!   we have no continuation-passing transform yet.
//! - `Promise.race(arr)`: returns a Promise mirroring the first
//!   settled input (Resolved or Rejected).  When every input is
//!   Pending, throws a `TypeError` for the same reason as `all`.

use std::collections::BTreeMap;

use boa_cat::PromiseState;
use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

/// Build the `Promise` object and allocate it on the heap.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let mut props = BTreeMap::new();
    let _ = props.insert("resolve".to_owned(), Value::Native(resolve_impl));
    let _ = props.insert("reject".to_owned(), Value::Native(reject_impl));
    let _ = props.insert("all".to_owned(), Value::Native(all_impl));
    let _ = props.insert("race".to_owned(), Value::Native(race_impl));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}

/// `Promise.resolve(v)`: if `v` is already a `Value::Promise`,
/// return it unchanged.  Otherwise allocate a fresh `Resolved(v)`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn resolve_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    match value {
        Value::Promise(_) => Ok((Outcome::Normal(value), heap, fuel)),
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Object(_)
        | Value::Function(_)
        | Value::Native(_) => {
            let (id, heap) = heap.alloc_promise(PromiseState::Resolved(value));
            Ok((Outcome::Normal(Value::Promise(id)), heap, fuel))
        }
    }
}

/// `Promise.reject(v)`: always wraps `v` in a fresh `Rejected(v)`,
/// regardless of whether `v` is already a Promise (per spec).
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn reject_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let (id, heap) = heap.alloc_promise(PromiseState::Rejected(value));
    Ok((Outcome::Normal(Value::Promise(id)), heap, fuel))
}

/// `Promise.all(arr)`: walks the array; collects each element's
/// effective value (already-Resolved -> inner; non-Promise ->
/// itself).  Returns `Resolved(values_array)` once every input is
/// settled; the first `Rejected` element short-circuits the result
/// to that rejection.  Pending inputs throw `TypeError` -- we have
/// no continuation-passing transform in this engine yet, so we
/// can't suspend the caller waiting for Pendings to settle.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn all_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let array = args.first().cloned().unwrap_or(Value::Undefined);
    let entries = array_entries(&array, &heap);
    match settle_all(&entries, &heap) {
        SettleAll::AllResolved(values) => {
            let (array_value, heap) = build_array_object(values, heap);
            let (id, heap) = heap.alloc_promise(PromiseState::Resolved(array_value));
            Ok((Outcome::Normal(Value::Promise(id)), heap, fuel))
        }
        SettleAll::FirstRejected(value) => {
            let (id, heap) = heap.alloc_promise(PromiseState::Rejected(value));
            Ok((Outcome::Normal(Value::Promise(id)), heap, fuel))
        }
        SettleAll::SomePending => Ok((
            Outcome::Throw(type_error(
                "Promise.all over Pending inputs is unsupported in this engine's synchronous model",
            )),
            heap,
            fuel,
        )),
    }
}

/// `Promise.race(arr)`: returns a Promise mirroring the first
/// settled input.  When every input is Pending, throws `TypeError`
/// for the same reason as `Promise.all`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn race_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let array = args.first().cloned().unwrap_or(Value::Undefined);
    let entries = array_entries(&array, &heap);
    let first_settled = entries
        .into_iter()
        .find_map(|value| classify_entry(&value, &heap));
    match first_settled {
        Some(Settled::Resolved(v)) => {
            let (id, heap) = heap.alloc_promise(PromiseState::Resolved(v));
            Ok((Outcome::Normal(Value::Promise(id)), heap, fuel))
        }
        Some(Settled::Rejected(v)) => {
            let (id, heap) = heap.alloc_promise(PromiseState::Rejected(v));
            Ok((Outcome::Normal(Value::Promise(id)), heap, fuel))
        }
        None => Ok((
            Outcome::Throw(type_error(
                "Promise.race over Pending-only inputs is unsupported in this engine's synchronous model",
            )),
            heap,
            fuel,
        )),
    }
}

enum SettleAll {
    AllResolved(Vec<Value>),
    FirstRejected(Value),
    SomePending,
}

#[derive(Clone)]
enum Settled {
    Resolved(Value),
    Rejected(Value),
}

fn settle_all(entries: &[Value], heap: &Heap) -> SettleAll {
    let folded = entries.iter().try_fold(Vec::<Value>::new(), |acc, value| {
        match classify_entry(value, heap) {
            Some(Settled::Resolved(v)) => Ok(acc.into_iter().chain(std::iter::once(v)).collect()),
            Some(Settled::Rejected(v)) => Err(Some(v)),
            None => Err(None),
        }
    });
    match folded {
        Ok(values) => SettleAll::AllResolved(values),
        Err(Some(rejected)) => SettleAll::FirstRejected(rejected),
        Err(None) => SettleAll::SomePending,
    }
}

fn classify_entry(value: &Value, heap: &Heap) -> Option<Settled> {
    match value {
        Value::Promise(id) => match heap.promise(*id) {
            Some(PromiseState::Resolved(v)) => Some(Settled::Resolved(v.clone())),
            Some(PromiseState::Rejected(v)) => Some(Settled::Rejected(v.clone())),
            Some(PromiseState::Pending(_)) | None => None,
        },
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Object(_)
        | Value::Function(_)
        | Value::Native(_) => Some(Settled::Resolved(value.clone())),
    }
}

fn array_entries(value: &Value, heap: &Heap) -> Vec<Value> {
    let id_opt = match value {
        Value::Object(id) => Some(*id),
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Function(_)
        | Value::Native(_)
        | Value::Promise(_) => None,
    };
    id_opt
        .and_then(|id| heap.object(id))
        .map(|object| {
            let length = match object.get("length") {
                Some(Value::Number(n)) if n.is_finite() && *n >= 0.0 => *n as u32,
                Some(_) | None => 0,
            };
            (0..length)
                .map(|i| {
                    object
                        .get(&format!("{i}"))
                        .cloned()
                        .unwrap_or(Value::Undefined)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn build_array_object(values: Vec<Value>, heap: Heap) -> (Value, Heap) {
    let length = u32::try_from(values.len()).unwrap_or(u32::MAX);
    let map: BTreeMap<String, Value> = values
        .into_iter()
        .enumerate()
        .map(|(i, v)| (format!("{i}"), v))
        .chain(std::iter::once((
            "length".to_owned(),
            Value::Number(f64::from(length)),
        )))
        .collect();
    let (id, heap) = heap.alloc_object(Object::from_properties(map));
    (Value::Object(id), heap)
}

fn type_error(message: &str) -> Value {
    Value::String(format!("TypeError: {message}"))
}
