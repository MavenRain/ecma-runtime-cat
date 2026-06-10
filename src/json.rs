//! `JSON.stringify` native callable.

use std::collections::BTreeMap;

use boa_cat::Value;
use boa_cat::fuel::Fuel;
use boa_cat::heap::Heap;
use boa_cat::outcome::{EvalResult, Outcome};
use boa_cat::value::Object;

use crate::coercion::{first_arg, number_to_string};

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
fn stringify_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let value = first_arg(&args);
    let outcome = stringify_value(&value, &heap).map_or(Outcome::Normal(Value::Undefined), |s| {
        Outcome::Normal(Value::String(s))
    });
    Ok((outcome, heap, fuel))
}

fn stringify_value(value: &Value, heap: &Heap) -> Option<String> {
    match value {
        Value::Undefined | Value::Function(_) | Value::Native(_) | Value::Promise(_) => None,
        Value::Null => Some("null".to_owned()),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Number(n) => {
            if n.is_finite() {
                Some(number_to_string(*n))
            } else {
                Some("null".to_owned())
            }
        }
        Value::String(s) => Some(stringify_string(s)),
        Value::Object(id) => heap.object(*id).map(|obj| stringify_object(obj, heap)),
    }
}

fn stringify_string(s: &str) -> String {
    let escaped: String = s
        .chars()
        .map(|c| match c {
            '"' => "\\\"".to_owned(),
            '\\' => "\\\\".to_owned(),
            '\n' => "\\n".to_owned(),
            '\r' => "\\r".to_owned(),
            '\t' => "\\t".to_owned(),
            ch if u32::from(ch) < 0x20 => format!("\\u{:04x}", u32::from(ch)),
            ch => ch.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}

fn stringify_object(obj: &Object, heap: &Heap) -> String {
    if is_array_object(obj) {
        stringify_array(obj, heap)
    } else {
        stringify_plain_object(obj, heap)
    }
}

fn is_array_object(obj: &Object) -> bool {
    obj.get("length")
        .is_some_and(|v| matches!(v, Value::Number(_)))
        && obj
            .properties()
            .keys()
            .all(|k| k == "length" || k.parse::<u32>().is_ok())
}

fn stringify_array(obj: &Object, heap: &Heap) -> String {
    // ECMA-262 array length is a ToUint32-bounded value; the `as u32`
    // after the finite/non-negative guard mirrors that bounded conversion.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let length = obj
        .get("length")
        .and_then(|v| match v {
            Value::Number(n) if n.is_finite() && *n >= 0.0 => Some(*n as u32),
            _other => None,
        })
        .unwrap_or(0);
    let body: Vec<String> = (0..length)
        .map(|i| {
            obj.get(&format!("{i}")).map_or("null".to_owned(), |v| {
                stringify_value(v, heap).unwrap_or_else(|| "null".to_owned())
            })
        })
        .collect();
    format!("[{}]", body.join(","))
}

fn stringify_plain_object(obj: &Object, heap: &Heap) -> String {
    let body: Vec<String> = obj
        .properties()
        .iter()
        .filter_map(|(k, v)| {
            stringify_value(v, heap).map(|rendered| format!("{}:{rendered}", stringify_string(k)))
        })
        .collect();
    format!("{{{}}}", body.join(","))
}

/// Build the `JSON` object and allocate it on the heap.
#[must_use]
pub fn build(heap: Heap) -> (Value, Heap) {
    let mut props = BTreeMap::new();
    let _ = props.insert("stringify".to_owned(), Value::Native(stringify_impl));
    let _ = props.insert("parse".to_owned(), Value::Native(parse_impl));
    let (id, heap) = heap.alloc_object(Object::from_properties(props));
    (Value::Object(id), heap)
}

/// `JSON.parse(source)` native callable (v0.3.2).  Parses the
/// first argument as a JSON string and returns the resulting
/// `Value`.  Throws `SyntaxError` on malformed input.
///
/// # Errors
///
/// Never returns `Err`; bad inputs surface as `Outcome::Throw`.
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn parse_impl(args: Vec<Value>, _this: Value, heap: Heap, fuel: Fuel) -> EvalResult {
    let source = first_string_arg(&args);
    // Snapshot the heap so a parse error restores it.  `parse_value`
    // may have allocated partial object / array values before
    // hitting the malformed token; throwing without restoring
    // would leave the caller's heap in an inconsistent half-built
    // state.
    let original_heap = heap.clone();
    parse_value_top_level(&source, heap).map_or_else(
        |message| {
            Ok((
                Outcome::Throw(Value::String(format!("SyntaxError: {message}"))),
                original_heap,
                fuel,
            ))
        },
        |(value, heap)| Ok((Outcome::Normal(value), heap, fuel)),
    )
}

fn first_string_arg(args: &[Value]) -> String {
    args.first()
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Undefined
            | Value::Null
            | Value::Boolean(_)
            | Value::Number(_)
            | Value::Object(_)
            | Value::Function(_)
            | Value::Native(_)
            | Value::Promise(_) => None,
        })
        .unwrap_or_default()
}

fn parse_value_top_level(source: &str, heap: Heap) -> Result<(Value, Heap), String> {
    let (value, rest, heap) = parse_value(source, heap)?;
    let trailing = rest.trim_start();
    if trailing.is_empty() {
        Ok((value, heap))
    } else {
        Err(format!("unexpected trailing content: {trailing:?}"))
    }
}

fn parse_value(source: &str, heap: Heap) -> Result<(Value, &str, Heap), String> {
    let trimmed = source.trim_start();
    let first_byte = trimmed.as_bytes().first().copied();
    first_byte.map_or_else(
        || Err("unexpected end of input".to_owned()),
        |byte| dispatch_value(byte, trimmed, heap),
    )
}

fn dispatch_value(first: u8, source: &str, heap: Heap) -> Result<(Value, &str, Heap), String> {
    match first {
        b'{' => parse_object(source, heap),
        b'[' => parse_array(source, heap),
        b'"' => parse_string(source).map(|(v, rest)| (v, rest, heap)),
        b't' | b'f' => parse_bool(source).map(|(v, rest)| (v, rest, heap)),
        b'n' => parse_null(source).map(|(v, rest)| (v, rest, heap)),
        c if c == b'-' || c.is_ascii_digit() => {
            parse_number(source).map(|(v, rest)| (v, rest, heap))
        }
        c => Err(format!("unexpected byte {c:?} at start of value")),
    }
}

fn parse_null(source: &str) -> Result<(Value, &str), String> {
    source
        .strip_prefix("null")
        .map(|rest| (Value::Null, rest))
        .ok_or_else(|| "expected 'null'".to_owned())
}

fn parse_bool(source: &str) -> Result<(Value, &str), String> {
    source
        .strip_prefix("true")
        .map(|rest| (Value::Boolean(true), rest))
        .or_else(|| {
            source
                .strip_prefix("false")
                .map(|rest| (Value::Boolean(false), rest))
        })
        .ok_or_else(|| "expected 'true' or 'false'".to_owned())
}

fn parse_number(source: &str) -> Result<(Value, &str), String> {
    let end = source
        .bytes()
        .enumerate()
        .find(|(_, b)| !is_number_byte(*b))
        .map_or(source.len(), |(idx, _)| idx);
    let (number_text, rest) = source.split_at(end);
    let parsed = number_text
        .parse::<f64>()
        .map_err(|_| format!("invalid number {number_text:?}"))?;
    Ok((Value::Number(parsed), rest))
}

fn is_number_byte(b: u8) -> bool {
    b.is_ascii_digit() || b == b'-' || b == b'+' || b == b'.' || b == b'e' || b == b'E'
}

fn parse_string(source: &str) -> Result<(Value, &str), String> {
    let after_quote = source
        .strip_prefix('"')
        .ok_or_else(|| "expected opening '\"'".to_owned())?;
    decode_string(after_quote, String::new())
}

fn decode_string(source: &str, acc: String) -> Result<(Value, &str), String> {
    let first_byte = source.as_bytes().first().copied();
    first_byte.map_or_else(
        || Err("unterminated string literal".to_owned()),
        |byte| decode_string_byte(byte, source, acc),
    )
}

fn decode_string_byte(first: u8, source: &str, acc: String) -> Result<(Value, &str), String> {
    match first {
        b'"' => {
            let rest = source.get(1..).unwrap_or("");
            Ok((Value::String(acc), rest))
        }
        b'\\' => {
            let after_escape = source.get(1..).unwrap_or("");
            let (decoded, rest) = decode_escape(after_escape)?;
            decode_string(rest, format!("{acc}{decoded}"))
        }
        _other => {
            let first_char = source
                .chars()
                .next()
                .ok_or_else(|| "unexpected partial utf-8 sequence in string body".to_owned())?;
            let rest = source.get(first_char.len_utf8()..).unwrap_or("");
            decode_string(rest, format!("{acc}{first_char}"))
        }
    }
}

fn decode_escape(source: &str) -> Result<(String, &str), String> {
    let escape_byte = source
        .as_bytes()
        .first()
        .copied()
        .ok_or_else(|| "lone backslash at end of input".to_owned())?;
    let rest = source.get(1..).unwrap_or("");
    dispatch_escape(escape_byte, rest)
}

fn dispatch_escape(escape_byte: u8, rest: &str) -> Result<(String, &str), String> {
    match escape_byte {
        b'"' => Ok(("\"".to_owned(), rest)),
        b'\\' => Ok(("\\".to_owned(), rest)),
        b'/' => Ok(("/".to_owned(), rest)),
        b'b' => Ok(("\u{0008}".to_owned(), rest)),
        b'f' => Ok(("\u{000c}".to_owned(), rest)),
        b'n' => Ok(("\n".to_owned(), rest)),
        b'r' => Ok(("\r".to_owned(), rest)),
        b't' => Ok(("\t".to_owned(), rest)),
        b'u' => decode_unicode_escape(rest),
        other => Err(format!("unknown escape \\{}", other as char)),
    }
}

fn decode_unicode_escape(source: &str) -> Result<(String, &str), String> {
    let hex = source
        .get(..4)
        .ok_or_else(|| "unicode escape needs four hex digits".to_owned())?;
    let rest = source.get(4..).unwrap_or("");
    let code_point =
        u32::from_str_radix(hex, 16).map_err(|_| format!("invalid hex in \\u{hex}"))?;
    let character = char::from_u32(code_point)
        .ok_or_else(|| format!("invalid unicode code point {code_point:#x}"))?;
    Ok((character.to_string(), rest))
}

fn parse_array(source: &str, heap: Heap) -> Result<(Value, &str, Heap), String> {
    let after_bracket = source
        .strip_prefix('[')
        .ok_or_else(|| "expected '['".to_owned())?;
    collect_array_elements(after_bracket, Vec::new(), heap)
}

fn collect_array_elements(
    source: &str,
    acc: Vec<Value>,
    heap: Heap,
) -> Result<(Value, &str, Heap), String> {
    let trimmed = source.trim_start();
    let first_byte = trimmed.as_bytes().first().copied();
    first_byte.map_or_else(
        || Err("unterminated array literal".to_owned()),
        |byte| dispatch_array_element(byte, trimmed, acc, heap),
    )
}

fn dispatch_array_element(
    first: u8,
    source: &str,
    acc: Vec<Value>,
    heap: Heap,
) -> Result<(Value, &str, Heap), String> {
    match first {
        b']' => {
            let rest = source.get(1..).unwrap_or("");
            let (value, heap) = build_array_object(acc, heap);
            Ok((value, rest, heap))
        }
        _other => {
            let (value, after_value, heap) = parse_value(source, heap)?;
            let next_acc: Vec<Value> = acc.into_iter().chain(std::iter::once(value)).collect();
            let after_sep = after_value.trim_start();
            let after_comma = after_sep.strip_prefix(',').unwrap_or(after_sep);
            collect_array_elements(after_comma, next_acc, heap)
        }
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

fn parse_object(source: &str, heap: Heap) -> Result<(Value, &str, Heap), String> {
    let after_brace = source
        .strip_prefix('{')
        .ok_or_else(|| "expected '{'".to_owned())?;
    collect_object_members(after_brace, BTreeMap::new(), heap)
}

fn collect_object_members(
    source: &str,
    acc: BTreeMap<String, Value>,
    heap: Heap,
) -> Result<(Value, &str, Heap), String> {
    let trimmed = source.trim_start();
    let first_byte = trimmed.as_bytes().first().copied();
    first_byte.map_or_else(
        || Err("unterminated object literal".to_owned()),
        |byte| dispatch_object_member(byte, trimmed, acc, heap),
    )
}

fn dispatch_object_member(
    first: u8,
    source: &str,
    acc: BTreeMap<String, Value>,
    heap: Heap,
) -> Result<(Value, &str, Heap), String> {
    match first {
        b'}' => {
            let rest = source.get(1..).unwrap_or("");
            let (id, heap) = heap.alloc_object(Object::from_properties(acc));
            Ok((Value::Object(id), rest, heap))
        }
        b'"' => parse_one_object_member(source, acc, heap),
        c => Err(format!("expected string key, got {c:?}")),
    }
}

fn parse_one_object_member(
    source: &str,
    acc: BTreeMap<String, Value>,
    heap: Heap,
) -> Result<(Value, &str, Heap), String> {
    let (key_value, after_key) = parse_string(source)?;
    let key = match key_value {
        Value::String(s) => Ok(s),
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Number(_)
        | Value::Object(_)
        | Value::Function(_)
        | Value::Native(_)
        | Value::Promise(_) => Err("object key must be a JSON string".to_owned()),
    }?;
    let after_colon = after_key
        .trim_start()
        .strip_prefix(':')
        .ok_or_else(|| "expected ':' after object key".to_owned())?;
    let (value, after_value, heap) = parse_value(after_colon, heap)?;
    let mut next_acc = acc;
    let _ = next_acc.insert(key, value);
    let after_sep = after_value.trim_start();
    let after_comma = after_sep.strip_prefix(',').unwrap_or(after_sep);
    collect_object_members(after_comma, next_acc, heap)
}
