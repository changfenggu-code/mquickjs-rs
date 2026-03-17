//! Unit tests for Value encoding, numeric formatting, and string utilities.
//!
//! Migrated from src/value.rs and src/runtime/string.rs.

use mquickjs::runtime::string::{
    is_array_index, is_ascii_string, is_ident_continue, is_ident_start, JSString, StringTable,
};
use mquickjs::value::{
    float_to_value, format_float, Float, RawValue, Value, SHORT_INT_MAX, SHORT_INT_MIN,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_float() -> Float {
    314.0_f32 / 100.0
}

// ---------------------------------------------------------------------------
// Value encoding — special types
// ---------------------------------------------------------------------------

#[test]
fn test_null() {
    let v = Value::null();
    assert!(v.is_null());
    assert!(!v.is_undefined());
    assert!(!v.is_bool());
    assert!(!v.is_int());
    assert!(v.is_nullish());
}

#[test]
fn test_undefined() {
    let v = Value::undefined();
    assert!(!v.is_null());
    assert!(v.is_undefined());
    assert!(v.is_nullish());
}

#[test]
fn test_bool() {
    let t = Value::bool(true);
    let f = Value::bool(false);

    assert!(t.is_bool());
    assert!(f.is_bool());
    assert_eq!(t.to_bool(), Some(true));
    assert_eq!(f.to_bool(), Some(false));
}

#[test]
fn test_int() {
    let zero = Value::int(0);
    let pos = Value::int(42);
    let neg = Value::int(-100);
    let max = Value::int(SHORT_INT_MAX);
    let min = Value::int(SHORT_INT_MIN);

    assert!(zero.is_int());
    assert_eq!(zero.to_i32(), Some(0));
    assert_eq!(pos.to_i32(), Some(42));
    assert_eq!(neg.to_i32(), Some(-100));
    assert_eq!(max.to_i32(), Some(SHORT_INT_MAX));
    assert_eq!(min.to_i32(), Some(SHORT_INT_MIN));
}

#[test]
fn test_exception() {
    let v = Value::exception();
    assert!(v.is_exception());
    assert!(!v.is_null());
    assert!(!v.is_int());
}

#[test]
fn test_raw_value_debug() {
    assert_eq!(format!("{:?}", RawValue::NULL), "Null");
    assert_eq!(format!("{:?}", RawValue::UNDEFINED), "Undefined");
    assert_eq!(format!("{:?}", RawValue::TRUE), "Bool(true)");
    assert_eq!(format!("{:?}", RawValue::from_i32(42)), "Int(42)");
}

// ---------------------------------------------------------------------------
// Value encoding — floats
// ---------------------------------------------------------------------------

#[test]
fn test_float_basic() {
    let v = Value::float(sample_float());
    assert!(v.is_float());
    assert!(!v.is_int());
    assert!(v.is_number());
    let f = v.to_f32().unwrap();
    assert!((f - sample_float()).abs() < 0.001);
}

#[test]
fn test_float_nan() {
    let v = Value::nan();
    assert!(v.is_float());
    assert!(v.is_nan_value());
    assert!(!v.is_infinite_value());
    assert!(v.to_f32().unwrap().is_nan());
}

#[test]
fn test_float_infinity() {
    let v = Value::infinity();
    assert!(v.is_float());
    assert!(v.is_infinite_value());
    assert!(!v.is_nan_value());

    let v2 = Value::neg_infinity();
    assert!(v2.is_infinite_value());
    assert!(v2.to_f32().unwrap() < 0.0);
}

#[test]
fn test_float_no_collision() {
    // Float values must not be confused with other types
    let f = Value::float(42.0);
    assert!(!f.is_null());
    assert!(!f.is_undefined());
    assert!(!f.is_bool());
    assert!(!f.is_exception());
    assert!(!f.is_ptr());
    assert!(!f.is_string());
    assert!(!f.is_func());
}

#[test]
fn test_is_number() {
    assert!(Value::int(42).is_number());
    assert!(Value::float(sample_float()).is_number());
    assert!(!Value::null().is_number());
    assert!(!Value::bool(true).is_number());
}

#[test]
fn test_to_number_f32() {
    assert_eq!(Value::int(42).to_number_f32(), Some(42.0));
    let f = Value::float(sample_float()).to_number_f32().unwrap();
    assert!((f - sample_float()).abs() < 0.001);
    assert_eq!(Value::null().to_number_f32(), None);
}

// ---------------------------------------------------------------------------
// float_to_value / format_float
// ---------------------------------------------------------------------------

#[test]
fn test_float_to_value_normalization() {
    // Whole-number float normalizes to int
    let v = float_to_value(3.0);
    assert!(v.is_int());
    assert_eq!(v.to_i32(), Some(3));

    // Non-whole float stays as float
    let v = float_to_value(sample_float());
    assert!(v.is_float());

    // NaN stays as float
    let v = float_to_value(Float::NAN);
    assert!(v.is_float());
    assert!(v.is_nan_value());

    // Infinity stays as float
    let v = float_to_value(Float::INFINITY);
    assert!(v.is_float());
    assert!(v.is_infinite_value());
}

#[test]
fn test_format_float() {
    assert_eq!(format_float(sample_float()), "3.14");
    assert_eq!(format_float(3.0), "3");
    assert_eq!(format_float(-0.5), "-0.5");
    assert_eq!(format_float(Float::NAN), "NaN");
    assert_eq!(format_float(Float::INFINITY), "Infinity");
    assert_eq!(format_float(Float::NEG_INFINITY), "-Infinity");
    // JS spec: String(-0) === "0", sign is dropped
    assert_eq!(format_float(-0.0), "0");
    assert_eq!(format_float(0.0), "0");
    // Negative integers should still carry their sign
    assert_eq!(format_float(-5.0), "-5");
}

#[test]
fn test_negative_zero_preserved_in_float_to_value() {
    // -0.0 must stay as float, not collapse to int(0)
    let v = float_to_value(-0.0);
    assert!(v.is_float(), "-0.0 should remain a float");
    let f = v.to_f32().unwrap();
    assert!(f.is_sign_negative(), "-0.0 should preserve negative sign");
}

// ---------------------------------------------------------------------------
// Value::is_closure — no false positives for typed values
// ---------------------------------------------------------------------------

#[test]
fn test_is_closure_no_false_positive_for_regexp() {
    let v = Value::regexp_object(0);
    assert!(v.is_regexp_object());
    assert!(!v.is_closure(), "regexp must not be identified as closure");
}

#[test]
fn test_is_closure_no_false_positive_for_typed_array() {
    let v = Value::typed_array_object(0);
    assert!(v.is_typed_array());
    assert!(
        !v.is_closure(),
        "typed array must not be identified as closure"
    );
}

#[test]
fn test_is_closure_no_false_positive_for_array_buffer() {
    let v = Value::array_buffer_object(0);
    assert!(v.is_array_buffer());
    assert!(
        !v.is_closure(),
        "array buffer must not be identified as closure"
    );
}

// ---------------------------------------------------------------------------
// String utilities (from src/runtime/string.rs)
// ---------------------------------------------------------------------------

#[test]
fn test_is_ascii_string() {
    assert!(is_ascii_string("hello"));
    assert!(is_ascii_string(""));
    assert!(is_ascii_string("123"));
    assert!(!is_ascii_string("héllo"));
    assert!(!is_ascii_string("中文"));
}

#[test]
fn test_is_array_index() {
    assert_eq!(is_array_index("0"), Some(0));
    assert_eq!(is_array_index("1"), Some(1));
    assert_eq!(is_array_index("42"), Some(42));
    assert_eq!(is_array_index("12345"), Some(12345));
    assert_eq!(is_array_index(""), None);
    assert_eq!(is_array_index("01"), None); // Leading zero
    assert_eq!(is_array_index("-1"), None); // Negative
    assert_eq!(is_array_index("abc"), None);
    assert_eq!(is_array_index("1.5"), None);
}

#[test]
fn test_is_ident() {
    assert!(is_ident_start(b'a'));
    assert!(is_ident_start(b'Z'));
    assert!(is_ident_start(b'_'));
    assert!(is_ident_start(b'$'));
    assert!(!is_ident_start(b'0'));
    assert!(!is_ident_start(b'-'));

    assert!(is_ident_continue(b'a'));
    assert!(is_ident_continue(b'0'));
    assert!(is_ident_continue(b'_'));
    assert!(!is_ident_continue(b'-'));
}

#[test]
fn test_string_hash() {
    let h1 = StringTable::hash_string("hello");
    let h2 = StringTable::hash_string("hello");
    let h3 = StringTable::hash_string("world");

    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn test_string_table_creation() {
    let table = StringTable::new();
    assert_eq!(table.count(), 0);
}

#[test]
fn test_jsstring_max_len() {
    // Just verify the constant is reasonable
    let max_len = core::hint::black_box(JSString::MAX_LEN);
    assert!(max_len > 1_000_000);
}
