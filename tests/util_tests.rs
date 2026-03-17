//! Unit tests for utility functions: dtoa, unicode helpers.
//!
//! Migrated from src/util/dtoa.rs, src/util/mod.rs, and src/util/unicode.rs.

use mquickjs::util::dtoa::{i32_to_str, u64_to_str_radix};
use mquickjs::util::unicode::{
    code_unit_at_utf16, utf16_len, utf16_to_utf8_index, utf8_to_utf16_index,
};
use mquickjs::util::{unicode_from_utf8, unicode_to_utf8};

// ---------------------------------------------------------------------------
// dtoa tests
// ---------------------------------------------------------------------------

#[test]
fn test_i32_to_str() {
    let mut buf = [0u8; 32];

    let n = i32_to_str(&mut buf, 0);
    assert_eq!(&buf[..n], b"0");

    let n = i32_to_str(&mut buf, 42);
    assert_eq!(&buf[..n], b"42");

    let n = i32_to_str(&mut buf, -123);
    assert_eq!(&buf[..n], b"-123");

    let n = i32_to_str(&mut buf, i32::MAX);
    assert_eq!(&buf[..n], b"2147483647");
}

#[test]
fn test_u64_to_str_radix() {
    let mut buf = [0u8; 64];

    let n = u64_to_str_radix(&mut buf, 255, 16);
    assert_eq!(&buf[..n], b"ff");

    let n = u64_to_str_radix(&mut buf, 255, 2);
    assert_eq!(&buf[..n], b"11111111");

    let n = u64_to_str_radix(&mut buf, 35, 36);
    assert_eq!(&buf[..n], b"z");
}

// ---------------------------------------------------------------------------
// util::mod tests
// ---------------------------------------------------------------------------

#[test]
fn test_unicode_to_utf8() {
    let mut buf = [0u8; 4];

    assert_eq!(unicode_to_utf8(&mut buf, 0x41), 1);
    assert_eq!(buf[0], b'A');

    assert_eq!(unicode_to_utf8(&mut buf, 0x00E9), 2); // é
    assert_eq!(&buf[..2], &[0xC3, 0xA9]);

    assert_eq!(unicode_to_utf8(&mut buf, 0x4E2D), 3); // 中
    assert_eq!(&buf[..3], &[0xE4, 0xB8, 0xAD]);

    assert_eq!(unicode_to_utf8(&mut buf, 0x1F600), 4); // 😀
    assert_eq!(&buf[..4], &[0xF0, 0x9F, 0x98, 0x80]);
}

#[test]
fn test_unicode_from_utf8() {
    assert_eq!(unicode_from_utf8(b"A"), Some((0x41, 1)));
    assert_eq!(unicode_from_utf8(&[0xC3, 0xA9]), Some((0x00E9, 2)));
    assert_eq!(unicode_from_utf8(&[0xE4, 0xB8, 0xAD]), Some((0x4E2D, 3)));
    assert_eq!(
        unicode_from_utf8(&[0xF0, 0x9F, 0x98, 0x80]),
        Some((0x1F600, 4))
    );

    // Invalid sequences
    assert_eq!(unicode_from_utf8(&[0x80]), None); // Invalid start
    assert_eq!(unicode_from_utf8(&[0xC3]), None); // Truncated
}

// ---------------------------------------------------------------------------
// unicode tests
// ---------------------------------------------------------------------------

#[test]
fn test_utf16_len() {
    assert_eq!(utf16_len("hello"), 5);
    assert_eq!(utf16_len("中文"), 2);
    assert_eq!(utf16_len("😀"), 2); // Emoji is 2 UTF-16 code units
}

#[test]
fn test_utf16_index_conversion() {
    let s = "a中😀b";

    // "a" is at UTF-8 0, UTF-16 0
    // "中" is at UTF-8 1, UTF-16 1
    // "😀" is at UTF-8 4, UTF-16 2
    // "b" is at UTF-8 8, UTF-16 4

    assert_eq!(utf8_to_utf16_index(s, 0), 0);
    assert_eq!(utf8_to_utf16_index(s, 1), 1);
    assert_eq!(utf8_to_utf16_index(s, 4), 2);
    assert_eq!(utf8_to_utf16_index(s, 8), 4);

    assert_eq!(utf16_to_utf8_index(s, 0), 0);
    assert_eq!(utf16_to_utf8_index(s, 1), 1);
    assert_eq!(utf16_to_utf8_index(s, 2), 4);
    assert_eq!(utf16_to_utf8_index(s, 4), 8);
}

#[test]
fn test_code_unit_at() {
    let s = "a😀b";

    assert_eq!(code_unit_at_utf16(s, 0), Some(b'a' as u16));
    assert_eq!(code_unit_at_utf16(s, 1), Some(0xD83D)); // High surrogate for 😀
    assert_eq!(code_unit_at_utf16(s, 2), Some(0xDE00)); // Low surrogate for 😀
    assert_eq!(code_unit_at_utf16(s, 3), Some(b'b' as u16));
}
