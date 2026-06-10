//! `Object` global static methods (v0.3.3): `keys`, `values`,
//! `entries`, `assign`.
//!
//! All four follow ECMA-262 semantics on enumerable own data
//! properties.  Accessor properties are walked via
//! `Object::accessors()` only for `assign` (which copies the
//! accessor itself onto the target so the receiver sees the same
//! getter/setter pair); `keys` / `values` / `entries` ignore
//! accessors per spec (in the spec they invoke the getter; in
//! this engine we have no way to invoke from a `NativeFn` without
//! a public callback API, so we surface only the data view).
//!
//! Arrays are returned as `Value::Object` with numeric string keys
//! plus a `length` property, mirroring the array shape the
//! existing array-literal evaluator and `JSON.parse` produce.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::first_arg;

/// Build the `Object` global and allocate it on the heap.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let mut props = BTreeMap::new();
    let _ = props.insert("keys".to_owned(), Value::Native(keys_impl));
    let _ = props.insert("values".to_owned(), Value::Native(values_impl));
    let _ = props.insert("entries".to_owned(), Value::Native(entries_impl));
    let _ = props.insert("assign".to_owned(), Value::Native(assign_impl));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}

/// `Object.keys(obj)` -- returns an array of own data-property
/// names, excluding `length` when the target is an array (so the
/// caller sees the same key set the spec's `OwnPropertyKeys` would
/// surface).
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn keys_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let arg = first_arg(&args);
    let entries = collect_entries(&arg, &heap);
    let keys: Vec<Value> = entries
        .into_iter()
        .map(|(k, _v)| Value::String(k))
        .collect();
    let (value, heap) = build_array_object(keys, heap);
    Ok((Outcome::Normal(value), heap, fuel))
}

/// `Object.values(obj)` -- returns an array of own data-property
/// values (in the same order as `keys`).
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn values_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let arg = first_arg(&args);
    let entries = collect_entries(&arg, &heap);
    let values: Vec<Value> = entries.into_iter().map(|(_k, v)| v).collect();
    let (value, heap) = build_array_object(values, heap);
    Ok((Outcome::Normal(value), heap, fuel))
}

/// `Object.entries(obj)` -- returns an array of `[k, v]` two-element
/// arrays.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn entries_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let arg = first_arg(&args);
    let kv = collect_entries(&arg, &heap);
    let (entry_values, heap) = kv
        .into_iter()
        .fold((Vec::new(), heap), |(acc, heap), (k, v)| {
            let (pair, heap) = build_array_object(vec![Value::String(k), v], heap);
            let extended: Vec<Value> = acc.into_iter().chain(std::iter::once(pair)).collect();
            (extended, heap)
        });
    let (value, heap) = build_array_object(entry_values, heap);
    Ok((Outcome::Normal(value), heap, fuel))
}

/// `Object.assign(target, ...sources)` -- copies each source's own
/// enumerable data properties onto `target`, in argument order.
/// Accessors are copied as accessors (preserving the pair on the
/// target).  Returns `target`.  Non-object targets / sources are
/// passed through unchanged (matching spec semantics where
/// primitive sources are coerced and primitive targets throw, but
/// our looser model just no-ops).
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn assign_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let target = first_arg(&args);
    let sources: Vec<Value> = args.iter().skip(1).cloned().collect();
    // `if let` here (not `match` or `map_or_else`) because both
    // branches need to move `target` / `heap`, which the
    // map_or_else closure pair can't express without cloning.
    if let Some(id) = object_id_of(&target) {
        let heap = sources
            .into_iter()
            .fold(heap, |heap, source| merge_source_into(id, &source, heap));
        Ok((Outcome::Normal(target), heap, fuel))
    } else {
        Ok((Outcome::Normal(target), heap, fuel))
    }
}

fn merge_source_into(target_id: boa_cat::value::ObjectId, source: &Value, heap: Heap) -> Heap {
    let source_id_opt = object_id_of(source);
    source_id_opt.map_or(heap.clone(), |source_id| {
        let source_obj = heap.object(source_id).cloned();
        let target_obj = heap.object(target_id).cloned();
        source_obj
            .zip(target_obj)
            .map_or(heap.clone(), |(src, tgt)| {
                let merged_data = src
                    .properties()
                    .iter()
                    .fold(tgt, |obj, (k, v)| obj.with(k.clone(), v.clone()));
                let merged_full = src.accessors().iter().fold(merged_data, |obj, (k, pair)| {
                    obj.with_accessor(k.clone(), pair.clone())
                });
                heap.store_object(target_id, merged_full)
                    .unwrap_or_else(|h| h)
            })
    })
}

fn collect_entries(value: &Value, heap: &Heap) -> Vec<(String, Value)> {
    let id_opt = object_id_of(value);
    id_opt
        .and_then(|id| heap.object(id))
        .map(|obj| {
            obj.properties()
                .iter()
                .filter(|(k, _)| !is_array_length_key(obj, k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn is_array_length_key(obj: &Object, key: &str) -> bool {
    key == "length" && looks_like_array(obj)
}

fn looks_like_array(obj: &Object) -> bool {
    obj.get("length")
        .is_some_and(|v| matches!(v, Value::Number(_)))
        && obj
            .properties()
            .keys()
            .all(|k| k == "length" || k.parse::<u32>().is_ok())
}

fn object_id_of(value: &Value) -> Option<boa_cat::value::ObjectId> {
    match value {
        Value::Object(id) => Some(*id),
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Function(_)
        | Value::Native(_)
        | Value::Promise(_) => None,
    }
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
