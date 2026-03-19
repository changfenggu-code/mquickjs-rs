# Bug Audit Report

> Generated: 2026-03-18 | Audited commit: `ef342e0`
> Scope: Full codebase review (excluding `src/vm/interpreter.rs` — under active modification)
>
> **Update 2026-03-19**: 4 additional tests in `led-runtime` were found failing during
> `EffectError` refactoring (pre-existing, not caused by recent changes):
> - `test_blink_set_color` (effects.rs) — `set_config("color", ...)` 不生效
> - `test_blink_set_speed` (effects.rs) — `set_config("speed", ...)` 不生效
> - `effect_instance_set_config_changes_behavior` (effect_api.rs) — 同上
> - `effect_manager_can_set_active_config` (effect_api.rs) — 同上
>
> **Root cause confirmed (2026-03-19)**: NOT in led-runtime. The bug is in **Rust mquickjs
> interpreter** — closure capture of outer-scope variable reassignment is broken. When JS code
> does `color = toRgb(value)` inside a handler closure, the outer `color` variable is reassigned
> but the closure's captured reference does not see the new value. Verified: C mqjs works correctly,
> Rust mqjs does not. See `src/vm/interpreter.rs` — the `OpCode::Closure` + variable binding
> mechanism does not properly handle reassignment of captured outer-scope variables.

## Summary

| Severity | Count |
|----------|-------|
| High     | 11    |
| Medium   | 19    |
| Low      | 7     |
| **Total**| **37 (+ 4 pre-existing led-runtime failures)**|

---

## High Severity (Must Fix)

### H1. `**=` compound assignment compiled as `=`
- **File:** `src/parser/compiler.rs` ~line 1958-1972
- **Issue:** `StarStarEq` is recognized by `is_assignment_op()` but has no arm in the compound assignment match block. Falls through to `_ => {}`, so `x **= 2` is silently compiled as `x = 2`.
- **Fix:** Add `Token::StarStarEq => self.emit_op(OpCode::Pow)` to the match block.

### H2. `typeof` undeclared variable corrupts constant pool
- **File:** `src/parser/compiler.rs` ~line 2044-2050
- **Issue:** `add_constant(Value::string(0))` is used as a placeholder, but `add_constant` deduplicates. If `Value::string(0)` already exists, the existing constant is overwritten with the new string index, corrupting earlier references.
- **Trigger:** `var x = "hello"; typeof missing;` — the constant for "hello" gets overwritten.
- **Fix:** Push a fresh constant directly without deduplication, or compute the real `str_idx` first.

### H3. `try {} finally {}` skips finally on exception path
- **File:** `src/parser/compiler.rs` ~line 1675-1695
- **Issue:** For `try-finally` (no catch), the exception path emits `Throw` before the finally block. The finally code is only reached on the normal path, violating JS semantics.
- **Trigger:** `try { throw 1; } finally { cleanup(); }` — cleanup never runs.
- **Fix:** Execute finally before re-throw (duplicate finally code on both paths or use Gosub/Ret mechanism).

### H4. `parseInt` ignores radix parameter
- **File:** `src/vm/natives.rs` ~line 853-859
- **Issue:** `args[1]` is completely ignored. Always parses in base 10.
- **Trigger:** `parseInt("0xFF", 16)` returns NaN instead of 255.
- **Fix:** Read and validate radix from `args[1]`, implement multi-radix parsing.

### H5. `parseInt`/`parseFloat` don't parse leading valid portion
- **File:** `src/vm/natives.rs` ~line 853-910
- **Issue:** Uses `s.parse::<i32>()` / `s.parse::<Float>()` which require the entire string to be valid.
- **Trigger:** `parseInt("123abc")` returns NaN instead of 123.
- **Fix:** Implement character-by-character parsing that stops at the first invalid character.

### H6. `Array.sort` rejects compare functions
- **File:** `src/vm/natives.rs` ~line 667-670
- **Issue:** When a comparison function is provided, the implementation throws "sort compareFn is not supported" instead of using it.
- **Fix:** Implement compare function support in the sort algorithm.

### H7. `Array.sort` default sorts numbers numerically
- **File:** `src/vm/natives.rs` ~line 679-684
- **Issue:** Special `all_numbers` path sorts numerically. Per ES5, default sort converts all elements to strings and sorts lexicographically.
- **Trigger:** `[9, 80].sort()` returns `[9, 80]` instead of correct `[80, 9]`.
- **Fix:** Remove the numeric fast path; always convert to string for default sort.

### H8. `Date.now()` returns truncated value
- **File:** `src/vm/natives.rs` ~line 3097-3100
- **Issue:** `millis % (1 << 30)` wraps every ~12 days. Should return full Unix timestamp.
- **Fix:** Return as float value to preserve the full millisecond timestamp.

### H9. `Array.toString` doesn't convert null/undefined to empty string
- **File:** `src/vm/natives.rs` ~line 3673
- **Issue:** Uses `format_value` which converts null to `"null"`. Per ES5, null and undefined should become empty strings in `toString`/`join`.
- **Trigger:** `[null, 1].toString()` → `"null,1"` instead of `",1"`.
- **Fix:** Check for null/undefined before formatting and emit empty string.

### H10. `String.length` returns UTF-8 byte count
- **File:** `src/vm/property.rs` ~line 112
- **Issue:** `s.len()` returns byte count, not character count. JS `String.length` should return UTF-16 code unit count.
- **Trigger:** `"café".length` returns 5 instead of 4.
- **Fix:** Use `utf16_len(s)` from `src/util/unicode.rs` (already exists).

### H11. All string methods confuse UTF-8 bytes with characters
- **File:** `src/vm/natives.rs` (multiple locations)
- **Issue:** `charAt`, `slice`, `substring`, `indexOf`, `lastIndexOf`, `includes`, `startsWith`, `padStart`, `padEnd` all use `s.len()` (byte length) for bounds and `s[start..end]` (byte slicing).
- **Impact:** Any non-ASCII string produces wrong results or panics.
- **Affected lines:** ~1474, 1627, 1547, 1646, 1688, 1930, 1997, 2049, 2163
- **Fix:** Systematic refactor to use character-based (or UTF-16-based) indexing throughout.

---

## Medium Severity (Should Fix)

### M1. `continue` in switch inside loop leaks stack value
- **File:** `src/parser/compiler.rs` ~line 1353-1377
- **Issue:** `continue` inside `switch` inside a loop jumps to loop target without dropping the switch discriminant value from the stack.
- **Trigger:** `for (...) { switch(x) { case 1: continue; } }`
- **Fix:** Emit `Drop` opcodes to clean up switch value before continue jump.

### M2. `skip_to_colon` doesn't handle ternary operator
- **File:** `src/parser/compiler.rs` ~line 1569-1588
- **Issue:** Stops at the first `:` at depth 0, not distinguishing case colon from ternary colon.
- **Trigger:** `switch(x) { case a ? 1 : 2: foo(); }` — parse error.
- **Fix:** Track ternary depth (increment on `?`, decrement on `:` when depth > 0).

### M3. `Array.indexOf`/`lastIndexOf` ignore fromIndex
- **File:** `src/vm/natives.rs` ~line 110-158
- **Issue:** `args[1]` (fromIndex) is ignored, always searches from 0.
- **Fix:** Read fromIndex, clamp to valid range, and start search from that position.

### M4. `Math.sign(-0)` returns 0
- **File:** `src/vm/natives.rs` ~line 1300-1314
- **Issue:** `Value::int(0)` is returned for both +0 and -0, losing the sign.
- **Fix:** Check `f.is_sign_negative()` and return `Value::float(-0.0)` for -0.

### M5. `Math.abs(i32::MIN)` returns negative
- **File:** `src/vm/natives.rs` ~line 1076
- **Issue:** `n.wrapping_abs()` for `i32::MIN` returns `i32::MIN` (cannot represent 2^31 in i32).
- **Fix:** Promote to float for the overflow case.

### M6. `String.includes()` with no args returns true
- **File:** `src/vm/natives.rs` ~line 2157
- **Issue:** Should coerce missing arg to `"undefined"` and search for it.
- **Fix:** Default search string to `"undefined"` when no args provided.

### M7. `String.split` non-string separator defaults to ","
- **File:** `src/vm/natives.rs` ~line 1800-1804
- **Issue:** Non-string separator should be coerced to string via `ToString`.
- **Fix:** Apply ToString coercion to separator argument.

### M8. `JSON.stringify(undefined)` returns string `"undefined"`
- **File:** `src/vm/natives.rs` ~line 2650
- **Issue:** Should return JS `undefined`, not the string `"undefined"`.
- **Fix:** Return `Value::undefined()` directly for undefined input.

### M9. `String.concat` drops float arguments
- **File:** `src/vm/natives.rs` ~line 1851-1868
- **Issue:** No case for float values. `"x".concat(3.14)` produces `"x"`.
- **Fix:** Add float-to-string conversion case.

### M10. `Number.toString` only supports radix 2/8/10/16
- **File:** `src/vm/natives.rs` ~line 952
- **Issue:** No RangeError for invalid radix, and radixes 3-7, 9, 11-15, 17-36 are unsupported.
- **Fix:** Implement full radix 2-36 support and RangeError validation.

### M11. `padStart`/`padEnd` panic on multi-byte pad strings
- **File:** `src/vm/natives.rs` ~line 2024, 2077
- **Issue:** `pad_string[..partial_pad]` byte-slices and may land mid-character.
- **Fix:** Use char-boundary-aware slicing.

### M12. Large index tag collision in Value encoding
- **File:** `src/value.rs` ~line 489-531
- **Issue:** Marker bits share the same u32 space as indices. When idx ≥ 131,072 (2^17), the index bits overlap with other type markers, causing `is_array()` / `is_object()` etc. to return true for wrong types.
- **Fix:** Add bounds checks on index creation, or restructure tag encoding.

### M13. `float_to_value` off-by-one at i32::MAX boundary
- **File:** `src/value.rs` ~line 910-915
- **Issue:** `i32::MAX as f32` rounds up to `2147483648.0`. The check `f <= (i32::MAX as Float)` admits this value, then `2147483648.0 as i32` saturates to `2147483647`.
- **Fix:** Use `f <= ((i32::MAX - 1) as Float)` or verify round-trip `(f as i32) as Float == f`.

### M14. `PropertyTable::define_accessor` not inserted into hash table
- **File:** `src/runtime/property.rs` ~line 251-265
- **Issue:** New accessor property is pushed to Vec but never added to the hash table. Subsequent `find(key)` never finds it.
- **Fix:** Compute hash, set `hash_next`, update `hash_table[bucket]` (mirror `set()` logic).

### M15. `define_accessor` discards getter/setter values
- **File:** `src/runtime/property.rs` ~line 254
- **Issue:** `let _ = (getter, setter);` — both values explicitly discarded. Property always stores `Value::null()`.
- **Fix:** Store getter/setter in the property (may need a new `Property` layout for accessor pairs).

### M16. `FunctionBytecode::add_constant`/`add_string` silent u16 truncation
- **File:** `src/runtime/function.rs` ~line 175-190
- **Issue:** `len() as u16` silently wraps to 0 when exceeding 65,535 entries.
- **Fix:** Return `Option<u16>` or panic/error on overflow.

### M17. `JSArray::set_length` shrink-then-grow doesn't clear stale values
- **File:** `src/runtime/array.rs` ~line 203-213
- **Issue:** `arr.length = 5; arr.length = 8;` leaves stale values at indices 5-7 instead of `undefined`.
- **Fix:** When shrinking, fill `elements[new_len..old_len]` with `Value::undefined()`.

### M18. GC compact doesn't update `heap_ptr`
- **File:** `src/gc/collector.rs` ~line 184
- **Issue:** `heap_ptr = write_offset` is commented out. Compacted memory is never reclaimed.
- **Status:** GC is currently a stub (marks everything), so no runtime impact yet.

### M19. GC compact doesn't update internal pointers
- **File:** `src/gc/collector.rs` ~line 167-168
- **Issue:** `TODO: Implement pointer updating using forwarding table`. After compaction, all object references become dangling.
- **Status:** GC is currently a stub, so no runtime impact yet.

---

## Low Severity (Fix Later)

### L1. JSON `\uXXXX` escape with non-hex chars silently consumed
- **File:** `src/vm/natives.rs` ~line 2848-2866
- **Issue:** `filter_map` skips non-hex chars. `"\u00GH"` silently fails instead of erroring.
- **Fix:** Return parse error on non-hex characters.

### L2. `String.repeat(-1)` doesn't throw RangeError
- **File:** `src/vm/natives.rs` ~line 1891
- **Issue:** Negative count is clamped to 0 via `.max(0)`. ES6 requires RangeError.
- **Fix:** Check for negative count and throw RangeError.

### L3. `performance.now` is identical to `Date.now`
- **File:** `src/vm/natives.rs` ~line 3113-3130
- **Issue:** Copy-paste of Date.now. Should be high-resolution monotonic time relative to start.
- **Fix:** Use monotonic clock and return float with sub-ms precision.

### L4. PropertyTable load factor includes tombstones
- **File:** `src/runtime/property.rs` ~line 138
- **Issue:** `properties.len()` includes deleted slots, causing premature resizes.
- **Fix:** Use `prop_count` instead.

### L5. `JSArray::from_values` doesn't truncate Vec on overflow
- **File:** `src/runtime/array.rs` ~line 50-56
- **Issue:** When `values.len() > MAX_ARRAY_LENGTH`, `len` is clamped but Vec keeps all elements. Excess elements are invisible to GC's `iter()`.
- **Fix:** `values.truncate(len as usize)`.

### L6. GC `mark_object` wrong size calculation on 32-bit targets
- **File:** `src/gc/collector.rs` ~line 113-116
- **Issue:** `size_words` is in machine words but loop iterates as if they're Value-sized. On 32-bit targets, reads 2x past the allocation.
- **Status:** Currently dead code (never called).
- **Fix:** Use `(size_words * WORD_SIZE) / VALUE_WORD_SIZE`.

### L7. `Error.stack` format inconsistent with `Error.toString`
- **File:** `src/vm/property.rs` ~line 227
- **Issue:** Uses `"{}:{}"` (no space) while `Error.toString` uses `"{}: {}"` (with space).
- **Fix:** Change to `"{}: {}"`.

---

## Priority Fix Order (Suggested)

1. **H1-H3** (compiler bugs) — Silent wrong code generation
2. **H4-H5** (parseInt/parseFloat) — Very commonly used functions
3. **H10-H11** (UTF-8/char confusion) — Systemic issue affecting all string ops
4. **H6-H7** (Array.sort) — Common usage pattern broken
5. **H8-H9** (Date.now, Array.toString) — Incorrect output
6. **M1-M19** — Medium priority fixes
7. **L1-L7** — Low priority / future work
