//! Native function implementations for built-in JS methods.
//!
//! All functions follow the signature: fn(interp, this, args) -> Result<Value, String>

use super::interpreter::*;
use crate::util::dtoa::u64_to_str_radix;
use crate::util::unicode::{utf16_len, utf16_to_utf8_index, utf8_to_utf16_index};
use crate::value::{float_to_value, format_float, Float, Value};
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

// =============================================================================
// Native function implementations
// =============================================================================

fn js_value_to_string(interp: &Interpreter, val: Value) -> Option<String> {
    if let Some(str_idx) = val.to_string_idx() {
        interp.get_string_by_idx(str_idx).map(|s| s.to_string())
    } else if let Some(n) = val.to_i32() {
        Some(n.to_string())
    } else if let Some(f) = val.to_f32() {
        Some(format_float(f))
    } else if val.is_undefined() {
        Some("undefined".to_string())
    } else if val.is_null() {
        Some("null".to_string())
    } else {
        val.to_bool()
            .map(|b| if b { "true" } else { "false" }.to_string())
    }
}

fn js_radix_arg(interp: &Interpreter, args: &[Value]) -> Option<u32> {
    let Some(radix_val) = args.get(1).copied() else {
        return Some(0);
    };
    let radix_num = interp.to_number(radix_val);
    if radix_num.is_nan_value() || radix_num.is_undefined() {
        return Some(0);
    }
    let Some(radix) = radix_num.to_number_f32() else {
        return Some(0);
    };
    let radix = radix as i32;
    if radix == 0 {
        Some(0)
    } else if (2..=36).contains(&radix) {
        Some(radix as u32)
    } else {
        None
    }
}

fn parse_int_string(interp: &Interpreter, s: &str, args: &[Value]) -> Value {
    let mut s = s.trim_start();
    let mut sign = 1.0_f32;

    if let Some(rest) = s.strip_prefix('+') {
        s = rest;
    } else if let Some(rest) = s.strip_prefix('-') {
        s = rest;
        sign = -1.0;
    }

    let Some(mut radix) = js_radix_arg(interp, args) else {
        return Value::nan();
    };

    if radix == 0 {
        if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            radix = 16;
            s = rest;
        } else {
            radix = 10;
        }
    } else if radix == 16 {
        if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            s = rest;
        }
    }

    let mut value = 0.0_f32;
    let mut parsed_any = false;
    for ch in s.chars() {
        let Some(digit) = ch.to_digit(radix) else {
            break;
        };
        parsed_any = true;
        value = value * radix as f32 + digit as f32;
    }

    if !parsed_any {
        return Value::nan();
    }
    if sign < 0.0 && value == 0.0 {
        return float_to_value(-0.0);
    }
    float_to_value(sign * value)
}

fn parse_float_prefix_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    if i < len && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }

    if s[i..].starts_with("Infinity") {
        return i + "Infinity".len();
    }

    let int_start = i;
    while i < len && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let mut has_digits = i > int_start;

    if i < len && bytes[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
        has_digits |= i > frac_start;
    }

    if !has_digits {
        return 0;
    }

    let before_exp = i;
    if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut exp_i = i + 1;
        if exp_i < len && (bytes[exp_i] == b'+' || bytes[exp_i] == b'-') {
            exp_i += 1;
        }
        let exp_start = exp_i;
        while exp_i < len && bytes[exp_i].is_ascii_digit() {
            exp_i += 1;
        }
        if exp_i > exp_start {
            i = exp_i;
        } else {
            i = before_exp;
        }
    }

    i
}

fn utf16_clamped_range(s: &str, start: usize, end: usize) -> (usize, usize) {
    let len = utf16_len(s);
    let start = start.min(len);
    let end = end.min(len);
    (utf16_to_utf8_index(s, start), utf16_to_utf8_index(s, end))
}

fn utf16_substring_owned(s: &str, start: usize, end: usize) -> String {
    let (start_u8, end_u8) = utf16_clamped_range(s, start, end);
    if start_u8 >= end_u8 {
        String::new()
    } else {
        s[start_u8..end_u8].to_string()
    }
}

/// Array.prototype.push - add elements to end of array
pub(crate) fn native_array_push(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "push called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get_mut(arr_idx as usize) {
        for arg in args {
            arr.push(*arg);
        }
        Ok(Value::int(arr.len() as i32))
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.pop - remove and return last element
pub(crate) fn native_array_pop(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "pop called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get_mut(arr_idx as usize) {
        Ok(arr.pop().unwrap_or_default())
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.length - get array length
pub(crate) fn native_array_length(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "length called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get(arr_idx as usize) {
        Ok(Value::int(arr.len() as i32))
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.shift - remove and return first element
pub(crate) fn native_array_shift(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "shift called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get_mut(arr_idx as usize) {
        if arr.is_empty() {
            Ok(Value::undefined())
        } else {
            Ok(arr.remove(0))
        }
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.unshift - add elements to beginning, return new length
pub(crate) fn native_array_unshift(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "unshift called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get_mut(arr_idx as usize) {
        // Insert arguments at beginning in order
        for (i, arg) in args.iter().enumerate() {
            arr.insert(i, *arg);
        }
        Ok(Value::int(arr.len() as i32))
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.indexOf - find index of element
pub(crate) fn native_array_index_of(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "indexOf called on non-array".to_string())?;

    let search_val = args.first().copied().unwrap_or_default();

    if let Some(arr) = interp.arrays.get(arr_idx as usize) {
        let len = arr.len() as i32;
        let from_index = args.get(1).and_then(|v| v.to_i32()).unwrap_or(0);
        let start = if from_index < 0 {
            (len + from_index).max(0) as usize
        } else {
            from_index.min(len) as usize
        };

        for (i, val) in arr.iter().enumerate().skip(start) {
            // Simple equality check (comparing raw values)
            if val.0 == search_val.0 {
                return Ok(Value::int(i as i32));
            }
        }
        Ok(Value::int(-1)) // Not found
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.lastIndexOf - find last occurrence of element
pub(crate) fn native_array_last_index_of(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "lastIndexOf called on non-array".to_string())?;

    let search_val = args.first().copied().unwrap_or_default();

    if let Some(arr) = interp.arrays.get(arr_idx as usize) {
        if arr.is_empty() {
            return Ok(Value::int(-1));
        }

        let len = arr.len() as i32;
        let from_index = args.get(1).and_then(|v| v.to_i32()).unwrap_or(len - 1);
        let start = if from_index < 0 {
            len + from_index
        } else {
            from_index.min(len - 1)
        };
        if start < 0 {
            return Ok(Value::int(-1));
        }

        // Search from end to beginning
        for i in (0..=start as usize).rev() {
            let val = &arr[i];
            // Simple equality check (comparing raw values)
            if val.0 == search_val.0 {
                return Ok(Value::int(i as i32));
            }
        }
        Ok(Value::int(-1)) // Not found
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.join - join elements with separator
pub(crate) fn native_array_join(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "join called on non-array".to_string())?;

    // JS uses "," when separator is omitted or explicitly undefined.
    let separator = if let Some(sep_val) = args.first() {
        if sep_val.is_undefined() {
            ",".to_string()
        } else if let Some(str_idx) = sep_val.to_string_idx() {
            interp.get_string_by_idx(str_idx).unwrap_or(",").to_string()
        } else {
            format_value(interp, *sep_val)
        }
    } else {
        ",".to_string()
    };

    if let Some(arr) = interp.arrays.get(arr_idx as usize) {
        let parts: Vec<String> = arr
            .iter()
            .map(|v| {
                if v.is_undefined() || v.is_null() {
                    String::new()
                } else {
                    format_value(interp, *v)
                }
            })
            .collect();

        let result = parts.join(&separator);
        Ok(interp.create_runtime_string(result))
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.reverse - reverse array in place
pub(crate) fn native_array_reverse(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "reverse called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get_mut(arr_idx as usize) {
        arr.reverse();
        Ok(this) // Return the array itself
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.slice - return shallow copy of portion of array
pub(crate) fn native_array_slice(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "slice called on non-array".to_string())?;

    if let Some(arr) = interp.arrays.get(arr_idx as usize) {
        let len = arr.len() as i32;

        // Get start index (default 0)
        let mut start = args.first().and_then(|v| v.to_i32()).unwrap_or(0);
        if start < 0 {
            start = (len + start).max(0);
        }
        let start = start.min(len) as usize;

        // Get end index (default length)
        let mut end = args.get(1).and_then(|v| v.to_i32()).unwrap_or(len);
        if end < 0 {
            end = (len + end).max(0);
        }
        let end = end.min(len) as usize;

        // Create new array with slice
        let slice: Vec<Value> = if start < end {
            arr[start..end].to_vec()
        } else {
            Vec::new()
        };

        // Store the new array
        let (new_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(slice);
        } else {
            interp.arrays[new_idx] = slice;
        }
        Ok(Value::array_idx(new_idx as u32))
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.map - create new array with callback applied to each element
pub(crate) fn native_array_map(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "map called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "map requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("map callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        let mapped = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;
        result.push(mapped);
    }

    let (new_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(result);
    } else {
        interp.arrays[new_idx] = result;
    }
    Ok(Value::array_idx(new_idx as u32))
}

/// Array.prototype.filter - create new array with elements that pass the test
pub(crate) fn native_array_filter(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "filter called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "filter requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("filter callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    let mut result = Vec::new();

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        let keep = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;

        // Convert to boolean
        if Interpreter::value_to_bool(keep) {
            result.push(element);
        }
    }

    let (new_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(result);
    } else {
        interp.arrays[new_idx] = result;
    }
    Ok(Value::array_idx(new_idx as u32))
}

/// Array.prototype.forEach - call callback for each element
pub(crate) fn native_array_foreach(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "forEach called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "forEach requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("forEach callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;
    }

    Ok(Value::undefined())
}

/// Array.prototype.reduce - reduce array to single value
pub(crate) fn native_array_reduce(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "reduce called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "reduce requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("reduce callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    if len == 0 && args.len() < 2 {
        return Err("reduce of empty array with no initial value".to_string());
    }

    // Get initial value or first element
    let (mut accumulator, start_idx) = if args.len() >= 2 {
        (args[1], 0)
    } else {
        (interp.array_element_or_undefined(arr_idx, 0), 1)
    };

    for i in start_idx..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [accumulator, element, Value::int(i as i32), this];
        accumulator = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;
    }

    Ok(accumulator)
}

/// Array.prototype.find - find first element that satisfies the test
pub(crate) fn native_array_find(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "find called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "find requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("find callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        let result = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;

        if Interpreter::value_to_bool(result) {
            return Ok(element);
        }
    }

    Ok(Value::undefined())
}

/// Array.prototype.findIndex - find index of first element that satisfies the test
pub(crate) fn native_array_find_index(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "findIndex called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "findIndex requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("findIndex callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        let result = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;

        if Interpreter::value_to_bool(result) {
            return Ok(Value::int(i as i32));
        }
    }

    Ok(Value::int(-1))
}

/// Array.prototype.some - check if any element satisfies the test
pub(crate) fn native_array_some(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "some called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "some requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("some callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        let result = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;

        if Interpreter::value_to_bool(result) {
            return Ok(Value::bool(true));
        }
    }

    Ok(Value::bool(false))
}

/// Array.prototype.every - check if all elements satisfy the test
pub(crate) fn native_array_every(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "every called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "every requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("every callback must be a function".to_string());
    }

    let len = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .len();

    for i in 0..len {
        let element = interp.array_element_or_undefined(arr_idx, i);
        let call_args = [element, Value::int(i as i32), this];
        let result = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;

        if !Interpreter::value_to_bool(result) {
            return Ok(Value::bool(false));
        }
    }

    Ok(Value::bool(true))
}

/// Array.prototype.includes - check if array includes a value
pub(crate) fn native_array_includes(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "includes called on non-array".to_string())?;

    let search_val = args.first().copied().unwrap_or_default();

    if let Some(arr) = interp.arrays.get(arr_idx as usize) {
        for element in arr.iter() {
            // Simple equality check
            if element.raw() == search_val.raw() {
                return Ok(Value::bool(true));
            }
        }
        Ok(Value::bool(false))
    } else {
        Err("invalid array".to_string())
    }
}

/// Array.prototype.concat - concatenate arrays
pub(crate) fn native_array_concat(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "concat called on non-array".to_string())?;

    // Clone the original array
    let original = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .clone();

    let mut result = original;

    // Concatenate each argument
    for arg in args {
        if let Some(other_idx) = arg.to_array_idx() {
            // Argument is an array - append all elements
            if let Some(other_arr) = interp.arrays.get(other_idx as usize) {
                result.extend(other_arr.iter().cloned());
            }
        } else {
            // Argument is a single value - append it
            result.push(*arg);
        }
    }

    let (new_arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(result);
    } else {
        interp.arrays[new_arr_idx] = result;
    }
    Ok(Value::array_idx(new_arr_idx as u32))
}

/// Array.prototype.sort - sort array in place
pub(crate) fn native_array_sort(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "sort called on non-array".to_string())?;

    // Get optional compare function
    let compare_fn = args.first().copied().filter(|v| !v.is_undefined());

    if let Some(compare_fn) = compare_fn {
        if !compare_fn.is_closure() && compare_fn.to_func_ptr().is_none() {
            return Err("sort compareFn must be a function".to_string());
        }
    }

    let mut sorted = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .clone();

    if let Some(compare_fn) = compare_fn {
        for i in 1..sorted.len() {
            let mut j = i;
            while j > 0 {
                let a = sorted[j - 1];
                let b = sorted[j];
                let cmp = interp
                    .call_value(compare_fn, Value::undefined(), &[a, b])
                    .map_err(|e| e.to_string())?
                    .to_number_f32()
                    .unwrap_or(0.0);
                if cmp <= 0.0 {
                    break;
                }
                sorted.swap(j - 1, j);
                j -= 1;
            }
        }
    } else {
        for i in 1..sorted.len() {
            let mut j = i;
            while j > 0 {
                let left = sorted[j - 1];
                let right = sorted[j];
                let should_swap = if left.is_undefined() {
                    false
                } else if right.is_undefined() {
                    true
                } else {
                    let left_key =
                        js_value_to_string(interp, left).unwrap_or("[object]".to_string());
                    let right_key =
                        js_value_to_string(interp, right).unwrap_or("[object]".to_string());
                    left_key > right_key
                };
                if !should_swap {
                    break;
                }
                sorted.swap(j - 1, j);
                j -= 1;
            }
        }
    }

    if let Some(slot) = interp.arrays.get_mut(arr_idx as usize) {
        *slot = sorted;
    }

    // Return the array itself (sort is in-place)
    Ok(this)
}

/// Array.prototype.flat - flatten nested arrays
pub(crate) fn native_array_flat(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "flat called on non-array".to_string())?;

    // Get depth (default 1)
    let depth = args.first().and_then(|v| v.to_i32()).unwrap_or(1).max(0) as usize;

    let original = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .clone();

    fn flatten_recursive(interp: &Interpreter, arr: &[Value], depth: usize) -> Vec<Value> {
        let mut result = Vec::new();
        for elem in arr {
            if depth > 0 {
                if let Some(nested_idx) = elem.to_array_idx() {
                    if let Some(nested) = interp.arrays.get(nested_idx as usize) {
                        result.extend(flatten_recursive(interp, nested, depth - 1));
                        continue;
                    }
                }
            }
            result.push(*elem);
        }
        result
    }

    let flattened = flatten_recursive(interp, &original, depth);

    let (new_arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(flattened);
    } else {
        interp.arrays[new_arr_idx] = flattened;
    }
    Ok(Value::array_idx(new_arr_idx as u32))
}

/// Array.prototype.fill - fill array with a value
pub(crate) fn native_array_fill(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "fill called on non-array".to_string())?;

    let fill_value = args.first().copied().unwrap_or_default();

    // Get start and end indices
    let arr_len = interp
        .arrays
        .get(arr_idx as usize)
        .map(|a| a.len())
        .unwrap_or(0) as i32;

    let start = args
        .get(1)
        .and_then(|v| v.to_i32())
        .map(|s| {
            if s < 0 {
                (arr_len + s).max(0)
            } else {
                s.min(arr_len)
            }
        })
        .unwrap_or(0) as usize;

    let end = args
        .get(2)
        .and_then(|v| v.to_i32())
        .map(|e| {
            if e < 0 {
                (arr_len + e).max(0)
            } else {
                e.min(arr_len)
            }
        })
        .unwrap_or(arr_len) as usize;

    if let Some(arr) = interp.arrays.get_mut(arr_idx as usize) {
        for i in start..end.min(arr.len()) {
            arr[i] = fill_value;
        }
    }

    // Return the array itself (fill is in-place)
    Ok(this)
}

/// parseInt - parse string to integer
pub(crate) fn native_parse_int(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();

    if let Some(n) = val.to_i32() {
        Ok(Value::int(n))
    } else if let Some(f) = val.to_f32() {
        if f.is_nan() || f.is_infinite() {
            Ok(Value::nan())
        } else {
            Ok(float_to_value(libm::truncf(f)))
        }
    } else if let Some(str_idx) = val.to_string_idx() {
        if let Some(s) = interp.get_string_by_idx(str_idx) {
            Ok(parse_int_string(interp, s, args))
        } else {
            Ok(Value::nan())
        }
    } else {
        Ok(Value::nan())
    }
}

/// isNaN - check if value is NaN
pub(crate) fn native_is_nan(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();

    let num = interp.to_number(val);

    if num.to_i32().is_some() {
        Ok(Value::bool(false))
    } else if let Some(f) = num.to_f32() {
        Ok(Value::bool(f.is_nan()))
    } else {
        Ok(Value::bool(true))
    }
}

/// parseFloat - parse a string to a float
pub(crate) fn native_parse_float(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();

    if val.is_int() || val.is_float() {
        return Ok(val);
    }

    if let Some(str_idx) = val.to_string_idx() {
        if let Some(s) = interp.get_string_by_idx(str_idx) {
            let s = s.trim_start();
            if s.is_empty() {
                return Ok(Value::nan());
            }
            let prefix_len = parse_float_prefix_len(s);
            if prefix_len == 0 {
                return Ok(Value::nan());
            }
            if let Ok(f) = s[..prefix_len].parse::<Float>() {
                return Ok(float_to_value(f));
            }
        }
    }

    Ok(Value::nan())
}

/// isFinite - check if value is finite
pub(crate) fn native_is_finite(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();

    let num = interp.to_number(val);

    if num.to_i32().is_some() {
        Ok(Value::bool(true))
    } else if let Some(f) = num.to_f32() {
        Ok(Value::bool(f.is_finite()))
    } else {
        Ok(Value::bool(false))
    }
}

// =============================================================================
// Number.prototype methods
// =============================================================================

/// Number.prototype.toString - convert number to string
pub(crate) fn native_number_to_string(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let radix = args.first().and_then(|v| v.to_i32()).unwrap_or(10);
    if !(2..=36).contains(&radix) {
        return Err("RangeError: toString radix must be between 2 and 36".to_string());
    }

    if let Some(n) = this.to_i32() {
        let s = if radix == 10 {
            n.to_string()
        } else {
            let magnitude = (n as i64).unsigned_abs();
            let mut buf = [0u8; 65];
            let len = u64_to_str_radix(&mut buf, magnitude, radix as u32);
            let digits = core::str::from_utf8(&buf[..len]).unwrap_or("");
            if n < 0 {
                format!("-{}", digits)
            } else {
                digits.to_string()
            }
        };
        Ok(interp.create_runtime_string(s))
    } else if let Some(f) = this.to_f32() {
        if radix == 10 {
            Ok(interp.create_runtime_string(format_float(f)))
        } else if f.is_finite() && (f - libm::truncf(f)) == 0.0 {
            let n = libm::truncf(f) as i64;
            let magnitude = n.unsigned_abs();
            let mut buf = [0u8; 65];
            let len = u64_to_str_radix(&mut buf, magnitude, radix as u32);
            let digits = core::str::from_utf8(&buf[..len]).unwrap_or("");
            let s = if n < 0 {
                format!("-{}", digits)
            } else {
                digits.to_string()
            };
            Ok(interp.create_runtime_string(s))
        } else {
            Ok(interp.create_runtime_string(format_float(f)))
        }
    } else {
        Err("toString called on non-number".to_string())
    }
}

/// Number.prototype.toFixed - format number with fixed decimal places
pub(crate) fn native_number_to_fixed(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let digits = args.first().and_then(|v| v.to_i32()).unwrap_or(0) as usize;

    if let Some(f) = this.to_number_f32() {
        let s = format!("{:.prec$}", f, prec = digits);
        Ok(interp.create_runtime_string(s))
    } else {
        Err("toFixed called on non-number".to_string())
    }
}

/// Number.prototype.toExponential - format number in exponential notation
pub(crate) fn native_number_to_exponential(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    if let Some(f) = this.to_number_f32() {
        let s = format!("{:e}", f);
        Ok(interp.create_runtime_string(s))
    } else {
        Err("toExponential called on non-number".to_string())
    }
}

/// Number.prototype.toPrecision - format number to specified precision
pub(crate) fn native_number_to_precision(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let precision = args.first().and_then(|v| v.to_i32()).unwrap_or(1) as usize;

    if let Some(f) = this.to_number_f32() {
        let s = format!("{:.prec$}", f, prec = precision.saturating_sub(1));
        Ok(interp.create_runtime_string(s))
    } else {
        Err("toPrecision called on non-number".to_string())
    }
}

// =============================================================================
// TypedArray.prototype methods
// =============================================================================

/// TypedArray.prototype.fill - fill typed array with a value
pub(crate) fn native_typed_array_fill(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let typed_idx = this
        .to_typed_array_idx()
        .ok_or_else(|| "fill called on non-TypedArray".to_string())?;

    let fill_val = args.first().copied().unwrap_or_default();

    let ta = interp
        .typed_arrays
        .get_mut(typed_idx as usize)
        .ok_or_else(|| "invalid TypedArray index".to_string())?;

    for i in 0..ta.length {
        ta.set(i, fill_val);
    }

    Ok(this)
}

/// TypedArray.prototype.subarray - create a new typed array view
pub(crate) fn native_typed_array_subarray(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let typed_idx = this
        .to_typed_array_idx()
        .ok_or_else(|| "subarray called on non-TypedArray".to_string())?;

    let start = args.first().and_then(|v| v.to_i32()).unwrap_or(0);
    let end = args.get(1).and_then(|v| v.to_i32());

    let ta = interp
        .typed_arrays
        .get(typed_idx as usize)
        .ok_or_else(|| "invalid TypedArray index".to_string())?;

    let new_ta = ta.subarray(start, end);
    let (new_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_typed_arrays);
    if is_new {
        interp.typed_arrays.push(new_ta);
    } else {
        interp.typed_arrays[new_idx] = new_ta;
    }

    Ok(Value::typed_array_object(new_idx as u32))
}

/// Math.abs - absolute value
pub(crate) fn native_math_abs(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(n) = val.to_i32() {
        if n == i32::MIN {
            Ok(float_to_value(2147483648.0))
        } else {
            Ok(Value::int(n.abs()))
        }
    } else if let Some(f) = val.to_f32() {
        Ok(float_to_value(libm::fabsf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.floor - floor value
pub(crate) fn native_math_floor(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(n) = val.to_i32() {
        Ok(Value::int(n))
    } else if let Some(f) = val.to_f32() {
        Ok(float_to_value(libm::floorf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.ceil - ceiling value
pub(crate) fn native_math_ceil(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(n) = val.to_i32() {
        Ok(Value::int(n))
    } else if let Some(f) = val.to_f32() {
        Ok(float_to_value(libm::ceilf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.max - maximum of values
pub(crate) fn native_math_max(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::neg_infinity());
    }
    let mut max = Float::NEG_INFINITY;
    for arg in args {
        if let Some(f) = arg.to_number_f32() {
            if f.is_nan() {
                return Ok(Value::nan());
            }
            if f > max {
                max = f;
            }
        } else {
            return Ok(Value::nan());
        }
    }
    Ok(float_to_value(max))
}

/// Math.min - minimum of values
pub(crate) fn native_math_min(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::infinity());
    }
    let mut min = Float::INFINITY;
    for arg in args {
        if let Some(f) = arg.to_number_f32() {
            if f.is_nan() {
                return Ok(Value::nan());
            }
            if f < min {
                min = f;
            }
        } else {
            return Ok(Value::nan());
        }
    }
    Ok(float_to_value(min))
}

/// Math.round - round to nearest integer
pub(crate) fn native_math_round(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(n) = val.to_i32() {
        Ok(Value::int(n))
    } else if let Some(f) = val.to_f32() {
        // JS uses "round half toward positive infinity": Math.round(-0.5) === 0, not -1.
        // libm::roundf uses C semantics (half away from zero), so we use floor(x + 0.5).
        Ok(float_to_value(libm::floorf(f + 0.5)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.sqrt - square root
pub(crate) fn native_math_sqrt(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::sqrtf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.pow - power function
pub(crate) fn native_math_pow(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let base = args.first().copied().unwrap_or_default();
    let exp = args.get(1).copied().unwrap_or_default();
    if let (Some(b), Some(e)) = (base.to_number_f32(), exp.to_number_f32()) {
        Ok(float_to_value(libm::powf(b, e)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.imul - 32-bit integer multiplication
pub(crate) fn native_math_imul(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let a = args.first().and_then(|v| v.to_number_f32()).unwrap_or(0.0) as i32;
    let b = args.get(1).and_then(|v| v.to_number_f32()).unwrap_or(0.0) as i32;
    let result = (a as i64 * b as i64) as i32;
    Ok(Value::int(result))
}

/// Math.clz32 - count leading zeros in 32-bit integer
pub(crate) fn native_math_clz32(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let n = args.first().and_then(|v| v.to_number_f32()).unwrap_or(0.0) as i32;
    let result = (n as u32).leading_zeros() as i32;
    Ok(Value::int(result))
}

/// Math.fround - round to nearest 32-bit float
pub(crate) fn native_math_fround(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(Value::float(f)) // Already f32, fround is identity
    } else {
        Ok(Value::nan())
    }
}

/// Math.trunc - truncate to integer (remove fractional part)
pub(crate) fn native_math_trunc(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(n) = val.to_i32() {
        Ok(Value::int(n))
    } else if let Some(f) = val.to_f32() {
        Ok(float_to_value(libm::truncf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.log2 - base-2 logarithm
pub(crate) fn native_math_log2(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::log2f(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.log10 - base-10 logarithm
pub(crate) fn native_math_log10(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::log10f(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.sign - returns the sign of a number
pub(crate) fn native_math_sign(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        if f.is_nan() {
            Ok(Value::nan())
        } else if f > 0.0 {
            Ok(Value::int(1))
        } else if f < 0.0 {
            Ok(Value::int(-1))
        } else if f.is_sign_negative() {
            Ok(float_to_value(-0.0))
        } else {
            Ok(Value::int(0))
        }
    } else {
        Ok(Value::nan())
    }
}

/// Math.sin - returns sine of a number (radians)
pub(crate) fn native_math_sin(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::sinf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.cos - returns cosine of a number (radians)
pub(crate) fn native_math_cos(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::cosf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.tan - returns tangent of a number (radians)
pub(crate) fn native_math_tan(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::tanf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.exp - returns e^x
pub(crate) fn native_math_exp(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::expf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.log - returns natural logarithm
pub(crate) fn native_math_log(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::logf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.random - returns a pseudo-random number in [0, 1)
pub(crate) fn native_math_random(
    interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    // Simple LCG PRNG that works in no_std
    interp.random_seed = interp
        .random_seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let random = ((interp.random_seed >> 33) as u32 % 1_000_000) as Float / 1_000_000.0;
    Ok(Value::float(random))
}

/// Math.atan2 - returns arctangent of y/x (radians)
pub(crate) fn native_math_atan2(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let y = args.first().and_then(|v| v.to_number_f32()).unwrap_or(0.0);
    let x = args.get(1).and_then(|v| v.to_number_f32()).unwrap_or(0.0);
    Ok(float_to_value(libm::atan2f(y, x)))
}

/// Math.asin - returns arcsine (radians)
pub(crate) fn native_math_asin(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::asinf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.acos - returns arccosine (radians)
pub(crate) fn native_math_acos(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::acosf(f)))
    } else {
        Ok(Value::nan())
    }
}

/// Math.atan - returns arctangent (radians)
pub(crate) fn native_math_atan(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if let Some(f) = val.to_number_f32() {
        Ok(float_to_value(libm::atanf(f)))
    } else {
        Ok(Value::nan())
    }
}

// =============================================================================
// String.prototype methods
// =============================================================================

/// String.prototype.charAt - get character at index
pub(crate) fn native_string_char_at(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "charAt called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let index = args.first().and_then(|v| v.to_i32()).unwrap_or(0) as usize;

    if index < s.len() {
        // Get the character at index (for ASCII strings)
        let ch = s
            .chars()
            .nth(index)
            .map(|c| c.to_string())
            .unwrap_or_default();
        let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(ch.into());
        Ok(Value::string(new_idx))
    } else {
        // Return empty string for out of bounds
        let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(String::new().into());
        Ok(Value::string(new_idx))
    }
}

/// String.prototype.charCodeAt - get character code at index
pub(crate) fn native_string_char_code_at(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "charCodeAt called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let index = args.first().and_then(|v| v.to_i32()).unwrap_or(0) as usize;

    if let Some(ch) = s.chars().nth(index) {
        Ok(Value::int(ch as i32))
    } else {
        Ok(Value::nan())
    }
}

/// String.prototype.lastIndexOf - find last occurrence of substring
pub(crate) fn native_string_last_index_of(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "lastIndexOf called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    // Get search string
    let search = if let Some(search_val) = args.first() {
        js_value_to_string(interp, *search_val).unwrap_or_default()
    } else {
        return Ok(Value::int(-1));
    };

    let position_utf16 = args
        .get(1)
        .and_then(|v| v.to_i32())
        .unwrap_or(utf16_len(s) as i32)
        .max(0) as usize;
    let max_start_utf16 = position_utf16.min(utf16_len(s));

    let mut last_match = None;
    for (byte_idx, _) in s.match_indices(&search) {
        let utf16_idx = utf8_to_utf16_index(s, byte_idx);
        if utf16_idx <= max_start_utf16 {
            last_match = Some(utf16_idx);
        }
    }
    Ok(Value::int(last_match.map(|idx| idx as i32).unwrap_or(-1)))
}

/// String.fromCharCode - create string from character codes
pub(crate) fn native_string_from_char_code(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let mut result = String::new();
    for arg in args {
        if let Some(code) = arg.to_i32() {
            if let Some(ch) = char::from_u32(code as u32) {
                result.push(ch);
            }
        }
    }
    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.fromCodePoint - create string from code points
pub(crate) fn native_string_from_code_point(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let mut result = String::new();
    for arg in args {
        if let Some(code) = arg.to_i32() {
            if code < 0 {
                return Err("Invalid code point".to_string());
            }
            if let Some(ch) = char::from_u32(code as u32) {
                result.push(ch);
            } else {
                return Err("Invalid code point".to_string());
            }
        }
    }
    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.indexOf - find substring
pub(crate) fn native_string_index_of(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "indexOf called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    // Get search string
    let search = if let Some(search_val) = args.first() {
        js_value_to_string(interp, *search_val).unwrap_or_default()
    } else {
        return Ok(Value::int(-1));
    };

    let position_utf16 = args.get(1).and_then(|v| v.to_i32()).unwrap_or(0).max(0) as usize;
    let position_u8 = utf16_to_utf8_index(s, position_utf16.min(utf16_len(s)));

    match s[position_u8..].find(&search) {
        Some(pos) => Ok(Value::int(utf8_to_utf16_index(s, position_u8 + pos) as i32)),
        None => Ok(Value::int(-1)),
    }
}

/// String.prototype.slice - extract portion of string
pub(crate) fn native_string_slice(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "slice called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let len = utf16_len(s) as i32;

    // Get start index
    let mut start = args.first().and_then(|v| v.to_i32()).unwrap_or(0);
    if start < 0 {
        start = (len + start).max(0);
    }
    let start = start.min(len) as usize;

    // Get end index
    let mut end = args.get(1).and_then(|v| v.to_i32()).unwrap_or(len);
    if end < 0 {
        end = (len + end).max(0);
    }
    let end = end.min(len) as usize;

    // Extract slice
    let result = utf16_substring_owned(s, start, end);

    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.substring - extract portion of string (similar to slice but different negative handling)
pub(crate) fn native_string_substring(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "substring called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let len = utf16_len(s) as i32;

    // Get start index (negative becomes 0)
    let start = args
        .first()
        .and_then(|v| v.to_i32())
        .unwrap_or(0)
        .max(0)
        .min(len) as usize;

    // Get end index (negative becomes 0)
    let end = args
        .get(1)
        .and_then(|v| v.to_i32())
        .unwrap_or(len)
        .max(0)
        .min(len) as usize;

    // Swap if start > end
    let (start, end) = if start > end {
        (end, start)
    } else {
        (start, end)
    };

    let result = utf16_substring_owned(s, start, end);

    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.toUpperCase - convert to uppercase
pub(crate) fn native_string_to_upper_case(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "toUpperCase called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let result = s.to_uppercase();

    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.toLowerCase - convert to lowercase
pub(crate) fn native_string_to_lower_case(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "toLowerCase called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let result = s.to_lowercase();

    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.trim - remove whitespace from both ends
pub(crate) fn native_string_trim(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "trim called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let result = s.trim().to_string();

    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.split - split string into array
pub(crate) fn native_string_split(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "split called on non-string".to_string())?;

    // Clone the string to avoid borrow issues
    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    // Get separator
    let separator = if let Some(sep_val) = args.first() {
        if sep_val.is_undefined() {
            let new_str_idx =
                interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
            interp.runtime_strings.push(s.into());

            let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
            if is_new {
                interp.arrays.push(vec![Value::string(new_str_idx)]);
            } else {
                interp.arrays[arr_idx] = vec![Value::string(new_str_idx)];
            }
            return Ok(Value::array_idx(arr_idx as u32));
        }
        js_value_to_string(interp, *sep_val).unwrap_or_default()
    } else {
        // No separator - return array with whole string
        let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(s.into());

        let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(vec![Value::string(new_str_idx)]);
        } else {
            interp.arrays[arr_idx] = vec![Value::string(new_str_idx)];
        }
        return Ok(Value::array_idx(arr_idx as u32));
    };

    // Split and create array of strings
    // Special case: empty separator splits into individual characters
    let string_parts: Vec<String> = if separator.is_empty() {
        s.chars().map(|c| c.to_string()).collect()
    } else {
        s.split(&separator).map(|p| p.to_string()).collect()
    };
    let mut parts: Vec<Value> = Vec::with_capacity(string_parts.len());
    for part in string_parts {
        let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(part.into());
        parts.push(Value::string(new_str_idx));
    }

    let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(parts);
    } else {
        interp.arrays[arr_idx] = parts;
    }
    Ok(Value::array_idx(arr_idx as u32))
}

/// String.prototype.concat - concatenate strings
pub(crate) fn native_string_concat(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "concat called on non-string".to_string())?;

    let mut result = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    // Concatenate all arguments
    for arg in args {
        if let Some(arg_str) = js_value_to_string(interp, *arg) {
            result.push_str(&arg_str);
        }
    }

    let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_str_idx))
}

/// String.prototype.repeat - repeat string n times
pub(crate) fn native_string_repeat(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "repeat called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let count = args.first().and_then(|v| v.to_i32()).unwrap_or(0);
    if count < 0 {
        return Err("RangeError: Invalid count value".to_string());
    }

    let result = s.repeat(count as usize);

    let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_str_idx))
}

/// String.prototype.startsWith - check if string starts with search string
pub(crate) fn native_string_starts_with(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "startsWith called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let search = if let Some(search_val) = args.first() {
        js_value_to_string(interp, *search_val).unwrap_or_default()
    } else {
        return Ok(Value::bool(false));
    };

    // Optional position argument
    let position = args.get(1).and_then(|v| v.to_i32()).unwrap_or(0).max(0) as usize;
    let position_u8 = utf16_to_utf8_index(s, position.min(utf16_len(s)));

    if position_u8 >= s.len() {
        return Ok(Value::bool(search.is_empty()));
    }

    Ok(Value::bool(s[position_u8..].starts_with(&search)))
}

/// String.prototype.endsWith - check if string ends with search string
pub(crate) fn native_string_ends_with(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "endsWith called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let search = if let Some(search_val) = args.first() {
        js_value_to_string(interp, *search_val).unwrap_or_default()
    } else {
        return Ok(Value::bool(false));
    };

    // Optional end position argument
    let end_position = args
        .get(1)
        .and_then(|v| v.to_i32())
        .map(|v| v.max(0) as usize)
        .unwrap_or(utf16_len(s));

    let end = end_position.min(utf16_len(s));
    let end_u8 = utf16_to_utf8_index(s, end);

    if utf16_len(&search) > end {
        return Ok(Value::bool(false));
    }

    Ok(Value::bool(s[..end_u8].ends_with(&search)))
}

/// String.prototype.padStart - pad string from start to target length
pub(crate) fn native_string_pad_start(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "padStart called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let target_length = args.first().and_then(|v| v.to_i32()).unwrap_or(0).max(0) as usize;
    let s_len = utf16_len(&s);

    if s_len >= target_length {
        let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(s.into());
        return Ok(Value::string(new_str_idx));
    }

    let pad_string = if let Some(pad_val) = args.get(1) {
        if let Some(pad_idx) = pad_val.to_string_idx() {
            interp.get_string_by_idx(pad_idx).unwrap_or(" ").to_string()
        } else {
            " ".to_string()
        }
    } else {
        " ".to_string()
    };

    if pad_string.is_empty() {
        let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(s.into());
        return Ok(Value::string(new_str_idx));
    }

    let pad_units = utf16_len(&pad_string);
    let pad_needed = target_length - s_len;
    let full_pads = pad_needed / pad_units;
    let partial_pad = pad_needed % pad_units;

    let mut result = pad_string.repeat(full_pads);
    result.push_str(&utf16_substring_owned(&pad_string, 0, partial_pad));
    result.push_str(&s);

    let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_str_idx))
}

/// String.prototype.padEnd - pad string from end to target length
pub(crate) fn native_string_pad_end(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "padEnd called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let target_length = args.first().and_then(|v| v.to_i32()).unwrap_or(0).max(0) as usize;
    let s_len = utf16_len(&s);

    if s_len >= target_length {
        let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(s.into());
        return Ok(Value::string(new_str_idx));
    }

    let pad_string = if let Some(pad_val) = args.get(1) {
        if let Some(pad_idx) = pad_val.to_string_idx() {
            interp.get_string_by_idx(pad_idx).unwrap_or(" ").to_string()
        } else {
            " ".to_string()
        }
    } else {
        " ".to_string()
    };

    if pad_string.is_empty() {
        let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(s.into());
        return Ok(Value::string(new_str_idx));
    }

    let pad_units = utf16_len(&pad_string);
    let pad_needed = target_length - s_len;
    let full_pads = pad_needed / pad_units;
    let partial_pad = pad_needed % pad_units;

    let mut result = s;
    result.push_str(&pad_string.repeat(full_pads));
    result.push_str(&utf16_substring_owned(&pad_string, 0, partial_pad));

    let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_str_idx))
}

/// String.prototype.replace - replace first occurrence of search with replacement
pub(crate) fn native_string_replace(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "replace called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let search = if let Some(search_val) = args.first() {
        if let Some(search_idx) = search_val.to_string_idx() {
            interp
                .get_string_by_idx(search_idx)
                .unwrap_or_default()
                .to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let replacement = if let Some(replace_val) = args.get(1) {
        if let Some(replace_idx) = replace_val.to_string_idx() {
            interp
                .get_string_by_idx(replace_idx)
                .unwrap_or_default()
                .to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    // Replace first occurrence only
    let result = s.replacen(&search, &replacement, 1);

    let new_str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_str_idx))
}

/// String.prototype.includes - check if string contains search string
pub(crate) fn native_string_includes(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "includes called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let search = if let Some(search_val) = args.first() {
        js_value_to_string(interp, *search_val).unwrap_or_default()
    } else {
        "undefined".to_string()
    };

    // Optional position argument
    let position = args.get(1).and_then(|v| v.to_i32()).unwrap_or(0).max(0) as usize;
    let position_u8 = utf16_to_utf8_index(s, position.min(utf16_len(s)));

    if position_u8 >= s.len() {
        return Ok(Value::bool(search.is_empty()));
    }

    Ok(Value::bool(s[position_u8..].contains(&search)))
}

/// String.prototype.match - match string against a RegExp
#[cfg(feature = "std")]
pub(crate) fn native_string_match(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "match called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let regex_arg = args.first().copied().unwrap_or_default();

    if let Some(regex_idx) = regex_arg.to_regexp_object_idx() {
        let re = interp
            .regex_objects
            .get(regex_idx as usize)
            .ok_or_else(|| "invalid RegExp object".to_string())?
            .clone();

        if re.global {
            let matches: Vec<String> = re
                .regex
                .find_iter(&s)
                .map(|m| m.as_str().to_string())
                .collect();

            if matches.is_empty() {
                return Ok(Value::null());
            }

            let mut result_arr: Vec<Value> = Vec::with_capacity(matches.len());
            for matched in matches {
                let str_idx =
                    interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
                interp.runtime_strings.push(matched.into());
                result_arr.push(Value::string(str_idx));
            }

            let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
            if is_new {
                interp.arrays.push(result_arr);
            } else {
                interp.arrays[arr_idx] = result_arr;
            }
            Ok(Value::array_idx(arr_idx as u32))
        } else {
            if let Some(m) = re.regex.find(&s) {
                let matched = m.as_str().to_string();
                let str_idx =
                    interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
                interp.runtime_strings.push(matched.into());

                let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
                if is_new {
                    interp.arrays.push(vec![Value::string(str_idx)]);
                } else {
                    interp.arrays[arr_idx] = vec![Value::string(str_idx)];
                }
                Ok(Value::array_idx(arr_idx as u32))
            } else {
                Ok(Value::null())
            }
        }
    } else if let Some(pattern_idx) = regex_arg.to_string_idx() {
        let pattern = interp
            .get_string_by_idx(pattern_idx)
            .ok_or_else(|| "invalid pattern string".to_string())?
            .to_string();

        match regex::Regex::new(&pattern) {
            Ok(re) => {
                if let Some(m) = re.find(&s) {
                    let matched = m.as_str().to_string();
                    let str_idx =
                        interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
                    interp.runtime_strings.push(matched.into());

                    let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
                    if is_new {
                        interp.arrays.push(vec![Value::string(str_idx)]);
                    } else {
                        interp.arrays[arr_idx] = vec![Value::string(str_idx)];
                    }
                    Ok(Value::array_idx(arr_idx as u32))
                } else {
                    Ok(Value::null())
                }
            }
            Err(_) => Ok(Value::null()),
        }
    } else {
        Ok(Value::null())
    }
}

#[cfg(not(feature = "std"))]
pub(crate) fn native_string_match(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Ok(Value::null())
}

/// String.prototype.search - search for a match and return index
#[cfg(feature = "std")]
pub(crate) fn native_string_search(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "search called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let regex_arg = args.first().copied().unwrap_or_default();

    if let Some(regex_idx) = regex_arg.to_regexp_object_idx() {
        let re = interp
            .regex_objects
            .get(regex_idx as usize)
            .ok_or_else(|| "invalid RegExp object".to_string())?
            .clone();

        if let Some(m) = re.regex.find(&s) {
            Ok(Value::int(m.start() as i32))
        } else {
            Ok(Value::int(-1))
        }
    } else if let Some(pattern_idx) = regex_arg.to_string_idx() {
        let pattern = interp
            .get_string_by_idx(pattern_idx)
            .ok_or_else(|| "invalid pattern string".to_string())?
            .to_string();

        match regex::Regex::new(&pattern) {
            Ok(re) => {
                if let Some(m) = re.find(&s) {
                    Ok(Value::int(m.start() as i32))
                } else {
                    Ok(Value::int(-1))
                }
            }
            Err(_) => Ok(Value::int(-1)),
        }
    } else {
        Ok(Value::int(-1))
    }
}

#[cfg(not(feature = "std"))]
pub(crate) fn native_string_search(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Ok(Value::int(-1))
}

/// String.prototype.codePointAt - get Unicode code point at position
pub(crate) fn native_string_code_point_at(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "codePointAt called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let index = args.first().and_then(|v| v.to_i32()).unwrap_or(0) as usize;

    // Get code point at index
    if let Some(ch) = s.chars().nth(index) {
        Ok(Value::int(ch as i32))
    } else {
        Ok(Value::undefined())
    }
}

/// String.prototype.trimStart - remove leading whitespace
pub(crate) fn native_string_trim_start(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "trimStart called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let trimmed = s.trim_start().to_string();
    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(trimmed.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.trimEnd - remove trailing whitespace
pub(crate) fn native_string_trim_end(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "trimEnd called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?;

    let trimmed = s.trim_end().to_string();
    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(trimmed.into());
    Ok(Value::string(new_idx))
}

/// String.prototype.replaceAll - replace all occurrences
pub(crate) fn native_string_replace_all(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let str_idx = this
        .to_string_idx()
        .ok_or_else(|| "replaceAll called on non-string".to_string())?;

    let s = interp
        .get_string_by_idx(str_idx)
        .ok_or_else(|| "invalid string".to_string())?
        .to_string();

    let search = args
        .first()
        .and_then(|v| v.to_string_idx())
        .and_then(|idx| interp.get_string_by_idx(idx).map(|s| s.to_string()))
        .unwrap_or_default();

    let replacement = args
        .get(1)
        .and_then(|v| v.to_string_idx())
        .and_then(|idx| interp.get_string_by_idx(idx).map(|s| s.to_string()))
        .unwrap_or_default();

    let result = s.replace(&search, &replacement);
    let new_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
    interp.runtime_strings.push(result.into());
    Ok(Value::string(new_idx))
}

// =============================================================================
// Number static methods
// =============================================================================

/// Number.isInteger - check if value is an integer
pub(crate) fn native_number_is_integer(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if val.to_i32().is_some() {
        Ok(Value::bool(true))
    } else if let Some(f) = val.to_f32() {
        Ok(Value::bool(f.is_finite() && (f - libm::truncf(f)) == 0.0))
    } else {
        Ok(Value::bool(false))
    }
}

/// Number.isNaN - check if value is NaN (strict: no type coercion)
pub(crate) fn native_number_is_nan(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    Ok(Value::bool(val.is_nan_value()))
}

/// Number.isFinite - check if value is a finite number (strict: no type coercion)
pub(crate) fn native_number_is_finite(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    if val.to_i32().is_some() {
        Ok(Value::bool(true))
    } else if let Some(f) = val.to_f32() {
        Ok(Value::bool(f.is_finite()))
    } else {
        Ok(Value::bool(false))
    }
}

// =============================================================================
// console methods
// =============================================================================

/// console.log - print values to stdout
#[cfg(feature = "std")]
pub(crate) fn native_console_log(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let output = format_console_args(interp, args);
    println!("{}", output);
    Ok(Value::undefined())
}

/// console.log - no-op in no_std
#[cfg(not(feature = "std"))]
pub(crate) fn native_console_log(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Ok(Value::undefined())
}

/// console.error - print values to stderr
#[cfg(feature = "std")]
pub(crate) fn native_console_error(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let output = format_console_args(interp, args);
    eprintln!("{}", output);
    Ok(Value::undefined())
}

/// console.error - no-op in no_std
#[cfg(not(feature = "std"))]
pub(crate) fn native_console_error(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Ok(Value::undefined())
}

/// console.warn - print values to stderr with warning
#[cfg(feature = "std")]
pub(crate) fn native_console_warn(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let output = format_console_args(interp, args);
    eprintln!("{}", output);
    Ok(Value::undefined())
}

/// console.warn - no-op in no_std
#[cfg(not(feature = "std"))]
pub(crate) fn native_console_warn(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Ok(Value::undefined())
}

/// Format arguments for console output
pub(crate) fn format_console_args(interp: &Interpreter, args: &[Value]) -> String {
    args.iter()
        .map(|v| format_value(interp, *v))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a single value for output (with depth limit to prevent circular reference crashes)
pub(crate) fn format_value(interp: &Interpreter, val: Value) -> String {
    format_value_depth(interp, val, 0)
}

const FORMAT_MAX_DEPTH: usize = 10;

fn format_value_depth(interp: &Interpreter, val: Value, depth: usize) -> String {
    if let Some(n) = val.to_i32() {
        n.to_string()
    } else if let Some(f) = val.to_f32() {
        format_float(f)
    } else if let Some(b) = val.to_bool() {
        b.to_string()
    } else if val.is_null() {
        "null".to_string()
    } else if val.is_undefined() {
        "undefined".to_string()
    } else if let Some(str_idx) = val.to_string_idx() {
        if let Some(s) = interp.get_string_by_idx(str_idx) {
            s.to_string()
        } else {
            "<string>".to_string()
        }
    } else if val.is_array() {
        if depth >= FORMAT_MAX_DEPTH {
            return "[Array]".to_string();
        }
        if let Some(arr_idx) = val.to_array_idx() {
            if let Some(arr) = interp.arrays.get(arr_idx as usize) {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| format_value_depth(interp, *v, depth + 1))
                    .collect();
                format!("[{}]", items.join(", "))
            } else {
                "[Array]".to_string()
            }
        } else {
            "[Array]".to_string()
        }
    } else if val.is_error_object() {
        if let Some(err_idx) = val.to_error_object_idx() {
            if let Some(err) = interp.error_objects.get(err_idx as usize) {
                if err.message.is_empty() {
                    err.name.clone()
                } else {
                    format!("{}: {}", err.name, err.message)
                }
            } else {
                "Error".to_string()
            }
        } else {
            "Error".to_string()
        }
    } else if val.is_object() {
        "[object Object]".to_string()
    } else if val.is_closure() {
        "[Function]".to_string()
    } else {
        format!("{:?}", val)
    }
}

// ===========================================
// JSON Functions
// ===========================================

/// JSON.stringify - convert a value to a JSON string
pub(crate) fn native_json_stringify(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::undefined());
    }
    let val = args[0];
    let mut visited = Vec::new();
    let json_str = json_stringify_value(interp, val, &mut visited);
    if json_str == "undefined" {
        return Ok(Value::undefined());
    }
    Ok(interp.create_runtime_string_json(json_str))
}

/// Helper function to stringify a value to JSON format.
/// `visited` tracks object/array indices to detect circular references.
fn json_stringify_value(interp: &Interpreter, val: Value, visited: &mut Vec<u64>) -> String {
    if let Some(n) = val.to_i32() {
        n.to_string()
    } else if let Some(f) = val.to_f32() {
        // JSON spec: NaN and Infinity serialize as null
        if f.is_nan() || f.is_infinite() {
            "null".to_string()
        } else {
            format_float(f)
        }
    } else if let Some(b) = val.to_bool() {
        b.to_string()
    } else if val.is_null() {
        "null".to_string()
    } else if val.is_undefined() {
        // undefined values are excluded in JSON.stringify
        "undefined".to_string()
    } else if let Some(str_idx) = val.to_string_idx() {
        if let Some(s) = interp.get_string_by_idx(str_idx) {
            // Escape the string for JSON
            format!("\"{}\"", escape_json_string(s))
        } else {
            "\"\"".to_string()
        }
    } else if val.is_array() {
        if let Some(arr_idx) = val.to_array_idx() {
            let key = val.raw().0;
            if visited.contains(&key) {
                // Circular reference — throw would be ideal, but return null for safety
                return "null".to_string();
            }
            if let Some(arr) = interp.arrays.get(arr_idx as usize) {
                visited.push(key);
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| {
                        let s = json_stringify_value(interp, *v, visited);
                        // Replace undefined with null in arrays
                        if s == "undefined" {
                            "null".to_string()
                        } else {
                            s
                        }
                    })
                    .collect();
                visited.pop();
                format!("[{}]", items.join(","))
            } else {
                "[]".to_string()
            }
        } else {
            "[]".to_string()
        }
    } else if val.is_object() {
        if let Some(obj_idx) = val.to_object_idx() {
            let key = val.raw().0;
            if visited.contains(&key) {
                // Circular reference — throw would be ideal, but return null for safety
                return "null".to_string();
            }
            if let Some(obj) = interp.objects.get(obj_idx as usize) {
                visited.push(key);
                let items: Vec<String> = obj
                    .properties
                    .iter()
                    .filter_map(|(k, v)| {
                        let val_str = json_stringify_value(interp, *v, visited);
                        // Skip undefined values in objects
                        if val_str == "undefined" {
                            None
                        } else {
                            Some(format!("\"{}\":{}", escape_json_string(k), val_str))
                        }
                    })
                    .collect();
                visited.pop();
                format!("{{{}}}", items.join(","))
            } else {
                "{}".to_string()
            }
        } else {
            "{}".to_string()
        }
    } else if val.is_closure() {
        // Functions are excluded in JSON.stringify
        "undefined".to_string()
    } else {
        "null".to_string()
    }
}

/// Escape a string for JSON output
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < ' ' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result
}

/// JSON.parse - parse a JSON string into a value
pub(crate) fn native_json_parse(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    if args.is_empty() {
        return Err("JSON.parse requires a string argument".to_string());
    }
    let val = args[0];

    // Get the string to parse
    if let Some(str_idx) = val.to_string_idx() {
        let json_str = if let Some(s) = interp.get_string_by_idx(str_idx) {
            s
        } else {
            return Err("Invalid string argument".to_string());
        };

        if str_idx < Interpreter::RUNTIME_STRING_OFFSET {
            // SAFETY: built-in and compile-time strings do not live inside
            // `interp.runtime_strings`, so parsing while mutating interpreter
            // allocation state will not invalidate this source slice.
            let json_ptr = json_str as *const str;
            let mut parser = JsonParser::new(unsafe { &*json_ptr });
            parser.parse_value(interp)
        } else {
            let json_owned = json_str.to_string();
            let mut parser = JsonParser::new(&json_owned);
            parser.parse_value(interp)
        }
    } else if let Some(n) = val.to_i32() {
        // Numbers can be parsed as JSON
        Ok(Value::int(n))
    } else {
        Err("JSON.parse requires a string argument".to_string())
    }
}

/// Simple JSON parser
pub(crate) struct JsonParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_value(&mut self, interp: &mut Interpreter) -> Result<Value, String> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Err("Unexpected end of JSON input".to_string());
        }

        let c = self.peek_char();
        match c {
            '"' => self.parse_string(interp),
            '[' => self.parse_array(interp),
            '{' => self.parse_object(interp),
            't' | 'f' => self.parse_boolean(),
            'n' => self.parse_null(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character '{}' in JSON", c)),
        }
    }

    fn peek_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn next_char(&mut self) -> char {
        let c = self.peek_char();
        if c != '\0' {
            self.pos += c.len_utf8();
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.peek_char() {
                ' ' | '\t' | '\n' | '\r' => {
                    self.next_char();
                }
                _ => break,
            }
        }
    }

    fn parse_string(&mut self, interp: &mut Interpreter) -> Result<Value, String> {
        self.next_char(); // consume opening quote
        let mut result = String::with_capacity(16);

        loop {
            if self.pos >= self.input.len() {
                return Err("Unterminated string in JSON".to_string());
            }

            let c = self.next_char();
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char();
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0C'),
                        'u' => {
                            // Parse unicode escape \uXXXX
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                let c = self.next_char();
                                if !c.is_ascii_hexdigit() {
                                    return Err("Invalid unicode escape in JSON string".to_string());
                                }
                                hex.push(c);
                            }
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(c) = char::from_u32(code) {
                                    result.push(c);
                                } else {
                                    return Err("Invalid unicode scalar in JSON string".to_string());
                                }
                            } else {
                                return Err("Invalid unicode escape in JSON string".to_string());
                            }
                        }
                        _ => result.push(escaped),
                    }
                }
                _ => result.push(c),
            }
        }
        Ok(interp.create_runtime_string(result))
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;

        // Handle negative sign
        if self.peek_char() == '-' {
            self.next_char();
        }

        // Parse digits
        while self.pos < self.input.len() && self.peek_char().is_ascii_digit() {
            self.next_char();
        }

        // Check for decimal point (we only support integers for now)
        if self.peek_char() == '.' {
            // Skip decimal part but parse as integer
            self.next_char();
            while self.pos < self.input.len() && self.peek_char().is_ascii_digit() {
                self.next_char();
            }
        }

        // Check for exponent
        if self.peek_char() == 'e' || self.peek_char() == 'E' {
            self.next_char();
            if self.peek_char() == '+' || self.peek_char() == '-' {
                self.next_char();
            }
            while self.pos < self.input.len() && self.peek_char().is_ascii_digit() {
                self.next_char();
            }
        }

        let num_str = &self.input[start..self.pos];

        if let Ok(n) = num_str.parse::<i32>() {
            Ok(Value::int(n))
        } else if let Ok(f) = num_str.parse::<Float>() {
            Ok(float_to_value(f))
        } else {
            Err(format!("Invalid number in JSON: {}", num_str))
        }
    }

    fn parse_boolean(&mut self) -> Result<Value, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(Value::bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(Value::bool(false))
        } else {
            Err("Invalid boolean in JSON".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<Value, String> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(Value::null())
        } else {
            Err("Invalid null in JSON".to_string())
        }
    }

    fn parse_array(&mut self, interp: &mut Interpreter) -> Result<Value, String> {
        self.next_char(); // consume '['
        self.skip_whitespace();

        let mut items: Vec<Value> = Vec::with_capacity(4);

        // Empty array
        if self.peek_char() == ']' {
            self.next_char();
            let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
            if is_new {
                interp.arrays.push(items);
            } else {
                interp.arrays[arr_idx] = items;
            }
            return Ok(Value::array_idx(arr_idx as u32));
        }

        loop {
            let value = self.parse_value(interp)?;
            items.push(value);

            self.skip_whitespace();
            let c = self.next_char();

            match c {
                ',' => {
                    self.skip_whitespace();
                }
                ']' => break,
                _ => return Err(format!("Expected ',' or ']' in array, found '{}'", c)),
            }
        }

        let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(items);
        } else {
            interp.arrays[arr_idx] = items;
        }
        Ok(Value::array_idx(arr_idx as u32))
    }

    fn parse_object(&mut self, interp: &mut Interpreter) -> Result<Value, String> {
        self.next_char(); // consume '{'
        self.skip_whitespace();

        let mut props: Vec<(String, Value)> = Vec::with_capacity(4);

        // Empty object
        if self.peek_char() == '}' {
            self.next_char();
            let (obj_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_objects);
            let obj = ObjectInstance {
                constructor: None,
                properties: props,
                accessors: Vec::new(),
            };
            if is_new {
                interp.objects.push(obj);
            } else {
                interp.objects[obj_idx] = obj;
            }
            return Ok(Value::object_idx(obj_idx as u32));
        }

        loop {
            self.skip_whitespace();

            // Parse key (must be a string)
            if self.peek_char() != '"' {
                return Err("Expected string key in object".to_string());
            }

            // Parse the key string directly
            self.next_char(); // consume opening quote
            let mut key = String::with_capacity(16);
            loop {
                if self.pos >= self.input.len() {
                    return Err("Unterminated string key in JSON".to_string());
                }
                let c = self.next_char();
                match c {
                    '"' => break,
                    '\\' => {
                        let escaped = self.next_char();
                        match escaped {
                            '"' => key.push('"'),
                            '\\' => key.push('\\'),
                            '/' => key.push('/'),
                            'n' => key.push('\n'),
                            'r' => key.push('\r'),
                            't' => key.push('\t'),
                            'b' => key.push('\x08'),
                            'f' => key.push('\x0C'),
                            'u' => {
                                let mut hex = String::with_capacity(4);
                                for _ in 0..4 {
                                    if self.pos < self.input.len() {
                                        hex.push(self.next_char());
                                    }
                                }
                                if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                                    if let Some(ch) = char::from_u32(cp) {
                                        key.push(ch);
                                    }
                                }
                            }
                            _ => key.push(escaped),
                        }
                    }
                    _ => key.push(c),
                }
            }

            self.skip_whitespace();

            // Expect colon
            if self.next_char() != ':' {
                return Err("Expected ':' after key in object".to_string());
            }

            self.skip_whitespace();

            // Parse value
            let value = self.parse_value(interp)?;
            props.push((key, value));

            self.skip_whitespace();
            let c = self.next_char();

            match c {
                ',' => {
                    self.skip_whitespace();
                }
                '}' => break,
                _ => return Err(format!("Expected ',' or '}}' in object, found '{}'", c)),
            }
        }

        let (obj_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_objects);
        let obj = ObjectInstance {
            constructor: None,
            properties: props,
            accessors: Vec::new(),
        };
        if is_new {
            interp.objects.push(obj);
        } else {
            interp.objects[obj_idx] = obj;
        }
        Ok(Value::object_idx(obj_idx as u32))
    }
}

// ===========================================
// Date Functions
// ===========================================

/// Date.now - returns current timestamp in milliseconds
pub(crate) fn native_date_now(
    interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let millis = interp.current_time_millis().unwrap_or(0) as Float;
    Ok(Value::float(millis))
}

/// performance.now - high-resolution time in milliseconds
pub(crate) fn native_performance_now(
    interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let now = interp
        .current_time_millis()
        .unwrap_or(interp.time_origin_millis);
    let elapsed = now.saturating_sub(interp.time_origin_millis) as Float;
    Ok(Value::float(elapsed))
}

// ===========================================
// RegExp Methods
// ===========================================

/// RegExp.prototype.test - tests if the regex matches the string
#[cfg(feature = "std")]
pub(crate) fn native_regexp_test(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let regex_idx = this
        .to_regexp_object_idx()
        .ok_or_else(|| "test called on non-RegExp".to_string())?;

    let re = interp
        .regex_objects
        .get(regex_idx as usize)
        .ok_or_else(|| "invalid RegExp object".to_string())?
        .clone();

    // Get string to test
    let test_str = if let Some(str_val) = args.first() {
        if let Some(str_idx) = str_val.to_string_idx() {
            interp
                .get_string_by_idx(str_idx)
                .ok_or_else(|| "invalid string".to_string())?
                .to_string()
        } else if let Some(n) = str_val.to_i32() {
            n.to_string()
        } else {
            "undefined".to_string()
        }
    } else {
        "undefined".to_string()
    };

    Ok(Value::bool(re.regex.is_match(&test_str)))
}

/// RegExp.prototype.test - stub for no_std
#[cfg(not(feature = "std"))]
pub(crate) fn native_regexp_test(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Err("RegExp.prototype.test not available in no_std".to_string())
}

/// RegExp.prototype.exec - executes the regex and returns match result
#[cfg(feature = "std")]
pub(crate) fn native_regexp_exec(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let regex_idx = this
        .to_regexp_object_idx()
        .ok_or_else(|| "exec called on non-RegExp".to_string())?;

    let re = interp
        .regex_objects
        .get(regex_idx as usize)
        .ok_or_else(|| "invalid RegExp object".to_string())?
        .clone();

    // Get string to match
    let match_str = if let Some(str_val) = args.first() {
        if let Some(str_idx) = str_val.to_string_idx() {
            interp
                .get_string_by_idx(str_idx)
                .ok_or_else(|| "invalid string".to_string())?
                .to_string()
        } else if let Some(n) = str_val.to_i32() {
            n.to_string()
        } else {
            "undefined".to_string()
        }
    } else {
        "undefined".to_string()
    };

    // Find the match
    if let Some(m) = re.regex.find(&match_str) {
        // Create result array with matched string
        let matched = m.as_str().to_string();
        let str_idx = interp.runtime_strings.len() as u16 + Interpreter::RUNTIME_STRING_OFFSET;
        interp.runtime_strings.push(matched.into());

        let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(vec![Value::string(str_idx)]);
        } else {
            interp.arrays[arr_idx] = vec![Value::string(str_idx)];
        }

        // Create result object with index and input properties
        let (_result_obj_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_objects);
        if is_new {
            interp.objects.push(crate::vm::interpreter::ObjectInstance {
                constructor: None,
                properties: vec![("index".to_string(), Value::int(m.start() as i32))],
                accessors: Vec::new(),
            });
        } else {
            interp.objects[_result_obj_idx] = crate::vm::interpreter::ObjectInstance {
                constructor: None,
                properties: vec![("index".to_string(), Value::int(m.start() as i32))],
                accessors: Vec::new(),
            };
        }

        // For now, just return the array (input property would require more work)
        Ok(Value::array_idx(arr_idx as u32))
    } else {
        Ok(Value::null())
    }
}

/// RegExp.prototype.exec - stub for no_std
#[cfg(not(feature = "std"))]
pub(crate) fn native_regexp_exec(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Err("RegExp.prototype.exec not available in no_std".to_string())
}

// ===========================================
// Object Static Methods
// ===========================================

/// Object.keys - returns array of object's own property names
pub(crate) fn native_object_keys(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let obj = args.first().copied().unwrap_or_default();

    if let Some(obj_idx) = obj.to_object_idx() {
        // Clone keys first to avoid borrow issues
        let key_strings: Vec<String> = interp
            .objects
            .get(obj_idx as usize)
            .map(|obj| obj.properties.iter().map(|(k, _)| k.clone()).collect())
            .unwrap_or_default();

        // Now create string values
        let keys: Vec<Value> = key_strings
            .into_iter()
            .map(|k| interp.create_runtime_string_object_key(k))
            .collect();

        let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(keys);
        } else {
            interp.arrays[arr_idx] = keys;
        }
        return Ok(Value::array_idx(arr_idx as u32));
    } else if let Some(arr_idx) = obj.to_array_idx() {
        // For arrays, get length first
        let len = interp
            .arrays
            .get(arr_idx as usize)
            .map(|a| a.len())
            .unwrap_or(0);

        // Create index strings
        let keys: Vec<Value> = (0..len)
            .map(|i| interp.create_runtime_string(i.to_string()))
            .collect();

        let (new_arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(keys);
        } else {
            interp.arrays[new_arr_idx] = keys;
        }
        return Ok(Value::array_idx(new_arr_idx as u32));
    }

    // Return empty array for non-objects
    let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(Vec::new());
    } else {
        interp.arrays[arr_idx] = Vec::new();
    }
    Ok(Value::array_idx(arr_idx as u32))
}

/// Object.values - returns array of object's own property values
pub(crate) fn native_object_values(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let obj = args.first().copied().unwrap_or_default();

    if let Some(obj_idx) = obj.to_object_idx() {
        // Clone values to avoid borrow issues
        let values: Vec<Value> = interp
            .objects
            .get(obj_idx as usize)
            .map(|obj| obj.properties.iter().map(|(_, v)| *v).collect())
            .unwrap_or_default();

        let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(values);
        } else {
            interp.arrays[arr_idx] = values;
        }
        return Ok(Value::array_idx(arr_idx as u32));
    } else if let Some(arr_idx) = obj.to_array_idx() {
        // For arrays, return a copy of values
        let arr_copy = interp
            .arrays
            .get(arr_idx as usize)
            .cloned()
            .unwrap_or_default();
        let (new_arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(arr_copy);
        } else {
            interp.arrays[new_arr_idx] = arr_copy;
        }
        return Ok(Value::array_idx(new_arr_idx as u32));
    }

    // Return empty array for non-objects
    let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(Vec::new());
    } else {
        interp.arrays[arr_idx] = Vec::new();
    }
    Ok(Value::array_idx(arr_idx as u32))
}

/// Object.entries - returns array of [key, value] pairs
pub(crate) fn native_object_entries(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let obj = args.first().copied().unwrap_or_default();

    if let Some(obj_idx) = obj.to_object_idx() {
        // Clone properties to avoid borrow issues
        let props: Vec<(String, Value)> = interp
            .objects
            .get(obj_idx as usize)
            .map(|obj| obj.properties.clone())
            .unwrap_or_default();

        // Create array of [key, value] pairs
        let mut entries: Vec<Value> = Vec::new();

        for (k, v) in props {
            let key_val = interp.create_runtime_string_object_entry_key(k);
            // Create inner array [key, value]
            let (pair_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
            if is_new {
                interp.arrays.push(vec![key_val, v]);
            } else {
                interp.arrays[pair_idx] = vec![key_val, v];
            }
            entries.push(Value::array_idx(pair_idx as u32));
        }

        let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
        if is_new {
            interp.arrays.push(entries);
        } else {
            interp.arrays[arr_idx] = entries;
        }
        return Ok(Value::array_idx(arr_idx as u32));
    }

    // Return empty array for non-objects
    let (arr_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if is_new {
        interp.arrays.push(Vec::new());
    } else {
        interp.arrays[arr_idx] = Vec::new();
    }
    Ok(Value::array_idx(arr_idx as u32))
}

/// Object.prototype.hasOwnProperty - check if object has own property
pub(crate) fn native_object_has_own_property(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    // Get the property name to check
    let prop_name = args
        .first()
        .and_then(|v| v.to_string_idx())
        .and_then(|idx| interp.get_string_by_idx(idx).map(|s| s.to_string()));

    let prop_name = match prop_name {
        Some(s) => s,
        None => return Ok(Value::bool(false)),
    };

    // Check if 'this' is an object and has the property
    if let Some(obj_idx) = this.to_object_idx() {
        if let Some(obj) = interp.get_object(obj_idx) {
            let exists = obj.properties.iter().any(|(k, _)| k == &prop_name)
                || obj.accessors.iter().any(|a| a.key == prop_name);
            return Ok(Value::bool(exists));
        }
        return Ok(Value::bool(false));
    }

    // Check if 'this' is an array
    if let Some(arr_idx) = this.to_array_idx() {
        if let Some(arr) = interp.arrays.get(arr_idx as usize) {
            // Check numeric indices
            if let Ok(idx) = prop_name.parse::<usize>() {
                return Ok(Value::bool(idx < arr.len()));
            }
            // Arrays also have 'length'
            if prop_name == "length" {
                return Ok(Value::bool(true));
            }
        }
        return Ok(Value::bool(false));
    }

    Ok(Value::bool(false))
}

/// Object.getPrototypeOf - get the prototype of an object
pub(crate) fn native_object_get_prototype_of(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let obj = args.first().copied().unwrap_or_default();

    // For our simple implementation, most objects don't have explicit prototypes
    // Arrays inherit from Array.prototype, objects from Object.prototype
    if obj.is_array() {
        // Return Array.prototype (represented as builtin)
        Ok(Value::builtin_object(BUILTIN_ARRAY))
    } else if obj.to_object_idx().is_some() {
        // Return Object.prototype (represented as builtin)
        Ok(Value::builtin_object(BUILTIN_OBJECT))
    } else if obj.is_string() {
        Ok(Value::builtin_object(BUILTIN_STRING))
    } else if obj.to_i32().is_some() {
        Ok(Value::builtin_object(BUILTIN_NUMBER))
    } else if obj.to_bool().is_some() {
        Ok(Value::builtin_object(BUILTIN_BOOLEAN))
    } else {
        Ok(Value::null())
    }
}

/// Object.setPrototypeOf - set the prototype of an object
pub(crate) fn native_object_set_prototype_of(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    // In our simple implementation, we don't support changing prototypes
    // Just return the object as-is (like a no-op)
    let obj = args.first().copied().unwrap_or_default();
    Ok(obj)
}

/// Object.create - create new object with specified prototype
pub(crate) fn native_object_create(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let _proto = args.first().copied().unwrap_or(Value::null());

    // Create a new empty object
    // In our simple implementation, we don't actually link the prototype
    let (obj_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_objects);
    if is_new {
        interp.objects.push(ObjectInstance {
            constructor: None,
            properties: Vec::new(),
            accessors: Vec::new(),
        });
    } else {
        interp.objects[obj_idx] = ObjectInstance {
            constructor: None,
            properties: Vec::new(),
            accessors: Vec::new(),
        };
    }

    Ok(Value::object_idx(obj_idx as u32))
}

/// Object.defineProperty - define a property on an object
pub(crate) fn native_object_define_property(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let obj = args.first().copied().unwrap_or_default();
    let prop = args.get(1).copied().unwrap_or_default();
    let descriptor = args.get(2).copied().unwrap_or_default();

    // Get property name
    let prop_name = if let Some(str_idx) = prop.to_string_idx() {
        interp.get_string_by_idx(str_idx).map(|s| s.to_string())
    } else {
        prop.to_i32().map(|n| n.to_string())
    };

    let prop_name = match prop_name {
        Some(s) => s,
        None => return Ok(obj),
    };

    if let Some(obj_idx) = obj.to_object_idx() {
        if let Some(desc_idx) = descriptor.to_object_idx() {
            let (value, getter, setter) =
                if let Some(desc_obj) = interp.objects.get(desc_idx as usize) {
                    let value = desc_obj
                        .properties
                        .iter()
                        .find(|(k, _)| k == "value")
                        .map(|(_, v)| *v)
                        .unwrap_or(Value::undefined());
                    let getter = desc_obj
                        .properties
                        .iter()
                        .find(|(k, _)| k == "get")
                        .map(|(_, v)| *v)
                        .unwrap_or(Value::undefined());
                    let setter = desc_obj
                        .properties
                        .iter()
                        .find(|(k, _)| k == "set")
                        .map(|(_, v)| *v)
                        .unwrap_or(Value::undefined());
                    (value, getter, setter)
                } else {
                    (Value::undefined(), Value::undefined(), Value::undefined())
                };

            if !getter.is_undefined() || !setter.is_undefined() {
                interp.object_define_accessor(obj_idx, prop_name, getter, setter);
            } else {
                interp
                    .object_set_property(obj_idx, prop_name, value)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            interp
                .object_set_property(obj_idx, prop_name, Value::undefined())
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(obj)
}

// ===========================================
// Array Static Methods
// ===========================================

/// Array.isArray - check if value is an array
pub(crate) fn native_array_is_array(
    _interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let val = args.first().copied().unwrap_or_default();
    Ok(Value::bool(val.is_array()))
}

// ===========================================
// Function.prototype Methods
// ===========================================

/// Function.prototype.call - call function with specified this value and arguments
/// Usage: func.call(thisArg, arg1, arg2, ...)
pub(crate) fn native_function_call(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    // 'this' is the function to call
    if !this.is_closure() && this.to_func_ptr().is_none() {
        return Err("call() called on non-function".to_string());
    }

    // First argument is the new 'this' value
    let new_this = args.first().copied().unwrap_or_default();

    // Remaining arguments are passed to the function
    let call_args: Vec<Value> = args.iter().skip(1).copied().collect();

    interp
        .call_value(this, new_this, &call_args)
        .map_err(|e| e.to_string())
}

/// Function.prototype.apply - call function with specified this value and arguments array
/// Usage: func.apply(thisArg, [argsArray])
pub(crate) fn native_function_apply(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    // 'this' is the function to call
    if !this.is_closure() && this.to_func_ptr().is_none() {
        return Err("apply() called on non-function".to_string());
    }

    // First argument is the new 'this' value
    let new_this = args.first().copied().unwrap_or_default();

    // Second argument should be an array of arguments
    let call_args: Vec<Value> = if let Some(arr_val) = args.get(1) {
        if let Some(arr_idx) = arr_val.to_array_idx() {
            interp.get_array(arr_idx).cloned().unwrap_or_default()
        } else if arr_val.is_undefined() || arr_val.is_null() {
            Vec::new()
        } else {
            return Err("second argument to apply() must be an array".to_string());
        }
    } else {
        Vec::new()
    };

    interp
        .call_value(this, new_this, &call_args)
        .map_err(|e| e.to_string())
}

/// Function.prototype.bind - create a new function with bound this value
/// Usage: func.bind(thisArg, arg1, arg2, ...) -> boundFunction
/// Note: Returns a value that stores the bound function, this, and args
pub(crate) fn native_function_bind(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    // 'this' is the function to bind
    if !this.is_closure() && this.to_func_ptr().is_none() {
        return Err("bind() called on non-function".to_string());
    }

    // Create a bound function object
    // We store: original function, bound this, and bound args
    let bound_this = args.first().copied().unwrap_or_default();
    let bound_args: Vec<Value> = args.iter().skip(1).copied().collect();

    // Create an object to store the bound function info
    let (obj_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_objects);
    let mut obj = ObjectInstance::new();
    obj.properties.push(("__bound_func__".to_string(), this));
    obj.properties
        .push(("__bound_this__".to_string(), bound_this));

    // Store bound args in an array
    let (arr_idx, arr_is_new) = interp.gc.alloc_slot(&mut interp.gen_arrays);
    if arr_is_new {
        interp.arrays.push(bound_args);
    } else {
        interp.arrays[arr_idx] = bound_args;
    }
    obj.properties.push((
        "__bound_args__".to_string(),
        Value::array_idx(arr_idx as u32),
    ));

    // Mark as bound function
    obj.properties
        .push(("__is_bound__".to_string(), Value::bool(true)));

    if is_new {
        interp.objects.push(obj);
    } else {
        interp.objects[obj_idx] = obj;
    }

    // Return as object (will be callable via special handling)
    Ok(Value::object_idx(obj_idx as u32))
}

/// Error.prototype.toString - returns "ErrorName: message"
pub(crate) fn native_error_to_string(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    if let Some(err_idx) = this.to_error_object_idx() {
        if let Some(err) = interp.error_objects.get(err_idx as usize).cloned() {
            let result = if err.message.is_empty() {
                err.name.clone()
            } else {
                format!("{}: {}", err.name, err.message)
            };
            return Ok(interp.create_runtime_string_error(result));
        }
    }
    // Fallback
    Ok(interp.create_runtime_string_error("Error".to_string()))
}

/// Function.prototype.toString - returns function source representation
pub(crate) fn native_function_to_string(
    interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    // In a real implementation, this would return the function source
    // For our simple implementation, return a generic representation
    Ok(interp.create_runtime_string("function () { [native code] }".to_string()))
}

/// Array.prototype.toString - same as join()
pub(crate) fn native_array_to_string(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let _ = args;
    native_array_join(interp, this, &[])
}

/// Array.prototype.reduceRight - reduce array from right to left
pub(crate) fn native_array_reduce_right(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let arr_idx = this
        .to_array_idx()
        .ok_or_else(|| "reduceRight called on non-array".to_string())?;

    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "reduceRight requires a callback function".to_string())?;

    if !callback.is_closure() && callback.to_func_ptr().is_none() {
        return Err("reduceRight callback must be a function".to_string());
    }

    // Clone the array to avoid borrow issues
    let arr_clone = interp
        .arrays
        .get(arr_idx as usize)
        .ok_or_else(|| "invalid array".to_string())?
        .clone();

    if arr_clone.is_empty() && args.len() < 2 {
        return Err("reduceRight of empty array with no initial value".to_string());
    }

    // Get initial value or last element
    let len = arr_clone.len();
    let (mut accumulator, end_idx) = if args.len() >= 2 {
        (args[1], len)
    } else {
        (arr_clone[len - 1], len - 1)
    };

    // Iterate from right to left
    for i in (0..end_idx).rev() {
        let element = arr_clone[i];
        let call_args = [accumulator, element, Value::int(i as i32), this];
        accumulator = interp
            .call_value(callback, Value::undefined(), &call_args)
            .map_err(|e| e.to_string())?;
    }

    Ok(accumulator)
}

/// Object.prototype.toString - returns "[object Type]" string representation
pub(crate) fn native_object_to_string(
    interp: &mut Interpreter,
    this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    let type_str = if this.is_null() {
        "[object Null]"
    } else if this.is_undefined() {
        "[object Undefined]"
    } else if this.is_array() {
        "[object Array]"
    } else if this.to_object_idx().is_some() {
        "[object Object]"
    } else if this.is_error_object() {
        "[object Error]"
    } else if this.is_regexp_object() {
        "[object RegExp]"
    } else if this.to_string_idx().is_some() || this.is_string() {
        "[object String]"
    } else if this.to_i32().is_some() {
        "[object Number]"
    } else if this.to_bool().is_some() {
        "[object Boolean]"
    } else if this.is_closure() || this.to_native_func_idx().is_some() {
        "[object Function]"
    } else {
        "[object Object]"
    };

    Ok(interp.create_runtime_string_type(type_str.to_string()))
}

/// gc() - trigger garbage collection immediately
pub(crate) fn native_gc(
    interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    interp.gc_collect();
    Ok(Value::undefined())
}

/// load(filename) - load and execute a JavaScript file
#[cfg(feature = "std")]
pub(crate) fn native_load(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let filename = args
        .first()
        .and_then(|v| v.to_string_idx())
        .and_then(|idx| interp.get_string_by_idx(idx).map(|s| s.to_string()))
        .ok_or_else(|| "load requires a filename string".to_string())?;

    // Read the file
    let contents = std::fs::read_to_string(&filename)
        .map_err(|e| format!("cannot load '{}': {}", filename, e))?;

    // Compile the source
    use crate::parser::compiler::Compiler;
    use crate::runtime::CaptureInfo;

    let compiled = Compiler::new(&contents)
        .compile()
        .map_err(|e| format!("compile error in '{}': {}", filename, e))?;

    // Convert to FunctionBytecode
    fn to_bytecode(
        compiled: crate::parser::compiler::CompiledFunction,
    ) -> crate::runtime::FunctionBytecode {
        let inner_functions = compiled.functions.into_iter().map(to_bytecode).collect();

        let captures = compiled
            .captures
            .into_iter()
            .map(|c| CaptureInfo {
                outer_index: c.outer_index,
                is_local: c.is_local,
            })
            .collect();

        crate::runtime::FunctionBytecode {
            name: None,
            arg_count: compiled.arg_count as u16,
            local_count: compiled.local_count as u16,
            stack_size: 64,
            has_arguments: false,
            uses_local0_string_builder: false,
            bytecode: compiled.bytecode,
            constants: compiled.constants,
            string_constants: compiled.string_constants,
            source_file: None,
            line_numbers: Vec::new(),
            inner_functions,
            captures,
        }
    }

    let bytecode = to_bytecode(compiled);

    interp
        .execute(&bytecode)
        .map_err(|e| format!("runtime error in '{}': {}", filename, e))
}

/// setTimeout(callback, delay) - schedule callback after delay (returns timer ID)
#[cfg(feature = "std")]
pub(crate) fn native_set_timeout(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let callback = args
        .first()
        .copied()
        .ok_or_else(|| "setTimeout requires a callback function".to_string())?;

    if !callback.is_closure()
        && callback.to_func_ptr().is_none()
        && callback.to_native_func_idx().is_none()
    {
        return Err("setTimeout callback must be a function".to_string());
    }

    let delay = args.get(1).and_then(|v| v.to_i32()).unwrap_or(0) as u64;

    // Get current time
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let timer_id = interp.next_timer_id;
    interp.next_timer_id += 1;

    let (timer_idx, is_new) = interp.gc.alloc_slot(&mut interp.gen_timers);
    let timer = Timer {
        id: timer_id,
        callback,
        fire_at: now + delay,
        cancelled: false,
    };
    if is_new {
        interp.timers.push(timer);
    } else {
        interp.timers[timer_idx] = timer;
    }

    Ok(Value::int(timer_id as i32))
}

/// setTimeout - stub for no_std (not supported)
#[cfg(not(feature = "std"))]
pub(crate) fn native_set_timeout(
    _interp: &mut Interpreter,
    _this: Value,
    _args: &[Value],
) -> Result<Value, String> {
    Err("setTimeout not available in no_std".to_string())
}

/// clearTimeout(id) - cancel a scheduled timeout
#[cfg(feature = "std")]
pub(crate) fn native_clear_timeout(
    interp: &mut Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let timer_id = args
        .first()
        .and_then(|v| v.to_i32())
        .ok_or_else(|| "clearTimeout requires a timer ID".to_string())? as u32;

    // Mark the timer as cancelled
    for timer in &mut interp.timers {
        if timer.id == timer_id {
            timer.cancelled = true;
            break;
        }
    }

    Ok(Value::undefined())
}

// =============================================================================
// Native function dispatch and registration
// =============================================================================

impl Interpreter {
    pub(crate) fn classify_native_error(message: String) -> InterpreterError {
        if let Some(msg) = message.strip_prefix("RangeError: ") {
            InterpreterError::RangeError(msg.to_string())
        } else if let Some(msg) = message.strip_prefix("ReferenceError: ") {
            InterpreterError::ReferenceError(msg.to_string())
        } else if let Some(msg) = message.strip_prefix("InternalError: ") {
            InterpreterError::InternalError(msg.to_string())
        } else if let Some(msg) = message.strip_prefix("TypeError: ") {
            InterpreterError::TypeError(msg.to_string())
        } else {
            InterpreterError::TypeError(message)
        }
    }

    /// Call a native function by index
    pub(crate) fn call_native_func(
        &mut self,
        idx: u32,
        this: Value,
        args: &[Value],
    ) -> InterpreterResult<Value> {
        let func = self
            .native_functions
            .get(idx as usize)
            .ok_or_else(|| {
                InterpreterError::InternalError(format!("invalid native function index: {}", idx))
            })?
            .clone();

        (func.func)(self, this, args).map_err(Self::classify_native_error)
    }

    /// Call a builtin object as a function (e.g., Boolean(value), Number(value))
    pub(crate) fn call_builtin_as_function(
        &mut self,
        builtin_idx: u32,
        args: &[Value],
    ) -> InterpreterResult<Value> {
        match builtin_idx {
            BUILTIN_BOOLEAN => {
                // Boolean(value) - coerces value to boolean
                let arg = args.first().copied().unwrap_or_default();
                Ok(Value::bool(self.to_boolean(arg)))
            }
            BUILTIN_NUMBER => {
                // Number(value) - coerces value to number
                let arg = args.first().copied().unwrap_or_default();
                Ok(self.to_number(arg))
            }
            BUILTIN_STRING => {
                // String(value) - coerces value to string
                let arg = args.first().copied().unwrap_or_default();
                Ok(self.stringify_value(arg))
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Builtin {} is not callable as a function",
                builtin_idx
            ))),
        }
    }

    /// Register built-in native functions
    pub(crate) fn register_builtins(&mut self) {
        // Array methods
        self.register_native("Array.prototype.push", native_array_push, 0);
        self.register_native("Array.prototype.map", native_array_map, 1);
        self.register_native("Array.prototype.filter", native_array_filter, 1);
        self.register_native("Array.prototype.reduce", native_array_reduce, 1);
        self.register_native("Array.prototype.pop", native_array_pop, 0);
        self.register_native("Array.prototype.length", native_array_length, 0);
        self.register_native("Array.prototype.shift", native_array_shift, 0);
        self.register_native("Array.prototype.unshift", native_array_unshift, 0);
        self.register_native("Array.prototype.indexOf", native_array_index_of, 1);
        self.register_native("Array.prototype.lastIndexOf", native_array_last_index_of, 1);
        self.register_native("Array.prototype.join", native_array_join, 0);
        self.register_native("Array.prototype.reverse", native_array_reverse, 0);
        self.register_native("Array.prototype.slice", native_array_slice, 0);
        self.register_native("Array.prototype.forEach", native_array_foreach, 1);
        self.register_native("Array.prototype.find", native_array_find, 1);
        self.register_native("Array.prototype.findIndex", native_array_find_index, 1);
        self.register_native("Array.prototype.some", native_array_some, 1);
        self.register_native("Array.prototype.every", native_array_every, 1);
        self.register_native("Array.prototype.includes", native_array_includes, 1);
        self.register_native("Array.prototype.concat", native_array_concat, 0);
        self.register_native("Array.prototype.sort", native_array_sort, 0);
        self.register_native("Array.prototype.flat", native_array_flat, 0);
        self.register_native("Array.prototype.fill", native_array_fill, 1);

        // TypedArray.prototype methods
        self.register_native("TypedArray.prototype.fill", native_typed_array_fill, 1);
        self.register_native(
            "TypedArray.prototype.subarray",
            native_typed_array_subarray,
            2,
        );

        // Global functions
        self.register_native("parseInt", native_parse_int, 1);
        self.register_native("parseFloat", native_parse_float, 1);
        self.register_native("isNaN", native_is_nan, 1);
        self.register_native("isFinite", native_is_finite, 1);

        // Math functions
        self.register_native("Math.abs", native_math_abs, 1);
        self.register_native("Math.floor", native_math_floor, 1);
        self.register_native("Math.ceil", native_math_ceil, 1);
        self.register_native("Math.round", native_math_round, 1);
        self.register_native("Math.sqrt", native_math_sqrt, 1);
        self.register_native("Math.pow", native_math_pow, 2);
        self.register_native("Math.max", native_math_max, 0);
        self.register_native("Math.min", native_math_min, 0);
        // mquickjs-specific Math functions
        self.register_native("Math.imul", native_math_imul, 2);
        self.register_native("Math.clz32", native_math_clz32, 1);
        self.register_native("Math.fround", native_math_fround, 1);
        self.register_native("Math.trunc", native_math_trunc, 1);
        self.register_native("Math.log2", native_math_log2, 1);
        self.register_native("Math.log10", native_math_log10, 1);
        self.register_native("Math.sign", native_math_sign, 1);
        self.register_native("Math.sin", native_math_sin, 1);
        self.register_native("Math.cos", native_math_cos, 1);
        self.register_native("Math.tan", native_math_tan, 1);
        self.register_native("Math.exp", native_math_exp, 1);
        self.register_native("Math.log", native_math_log, 1);
        self.register_native("Math.random", native_math_random, 0);
        self.register_native("Math.atan2", native_math_atan2, 2);
        self.register_native("Math.asin", native_math_asin, 1);
        self.register_native("Math.acos", native_math_acos, 1);
        self.register_native("Math.atan", native_math_atan, 1);

        // String methods
        self.register_native("String.prototype.charAt", native_string_char_at, 1);
        self.register_native("String.prototype.charCodeAt", native_string_char_code_at, 1);
        self.register_native("String.prototype.indexOf", native_string_index_of, 1);
        self.register_native(
            "String.prototype.lastIndexOf",
            native_string_last_index_of,
            1,
        );
        self.register_native("String.fromCharCode", native_string_from_char_code, 0);
        self.register_native("String.fromCodePoint", native_string_from_code_point, 0);
        self.register_native("String.prototype.slice", native_string_slice, 0);
        self.register_native("String.prototype.substring", native_string_substring, 0);
        self.register_native(
            "String.prototype.toUpperCase",
            native_string_to_upper_case,
            0,
        );
        self.register_native(
            "String.prototype.toLowerCase",
            native_string_to_lower_case,
            0,
        );
        self.register_native("String.prototype.trim", native_string_trim, 0);
        self.register_native("String.prototype.split", native_string_split, 0);
        self.register_native("String.prototype.concat", native_string_concat, 0);
        self.register_native("String.prototype.repeat", native_string_repeat, 1);
        self.register_native("String.prototype.startsWith", native_string_starts_with, 1);
        self.register_native("String.prototype.endsWith", native_string_ends_with, 1);
        self.register_native("String.prototype.padStart", native_string_pad_start, 1);
        self.register_native("String.prototype.padEnd", native_string_pad_end, 1);
        self.register_native("String.prototype.replace", native_string_replace, 2);
        self.register_native("String.prototype.includes", native_string_includes, 1);
        self.register_native("String.prototype.match", native_string_match, 1);
        self.register_native("String.prototype.search", native_string_search, 1);
        // mquickjs-specific String methods
        self.register_native(
            "String.prototype.codePointAt",
            native_string_code_point_at,
            1,
        );
        self.register_native("String.prototype.trimStart", native_string_trim_start, 0);
        self.register_native("String.prototype.trimEnd", native_string_trim_end, 0);
        self.register_native("String.prototype.replaceAll", native_string_replace_all, 2);

        // Number static methods
        self.register_native("Number.isInteger", native_number_is_integer, 1);
        self.register_native("Number.isNaN", native_number_is_nan, 1);
        self.register_native("Number.isFinite", native_number_is_finite, 1);

        // Number.prototype methods
        self.register_native("Number.prototype.toString", native_number_to_string, 0);
        self.register_native("Number.prototype.toFixed", native_number_to_fixed, 0);
        self.register_native(
            "Number.prototype.toExponential",
            native_number_to_exponential,
            0,
        );
        self.register_native(
            "Number.prototype.toPrecision",
            native_number_to_precision,
            0,
        );

        // console methods
        self.register_native("console.log", native_console_log, 0);
        self.register_native("console.error", native_console_error, 0);
        self.register_native("console.warn", native_console_warn, 0);

        // JSON methods
        self.register_native("JSON.stringify", native_json_stringify, 1);
        self.register_native("JSON.parse", native_json_parse, 1);

        // Date methods
        self.register_native("Date.now", native_date_now, 0);
        self.register_native("performance.now", native_performance_now, 0);

        // RegExp methods (require regex crate, std only)
        #[cfg(feature = "std")]
        {
            self.register_native("RegExp.prototype.test", native_regexp_test, 1);
            self.register_native("RegExp.prototype.exec", native_regexp_exec, 1);
        }

        // Object static methods
        self.register_native("Object.keys", native_object_keys, 1);
        self.register_native("Object.values", native_object_values, 1);
        self.register_native("Object.entries", native_object_entries, 1);
        self.register_native("Object.getPrototypeOf", native_object_get_prototype_of, 1);
        self.register_native("Object.setPrototypeOf", native_object_set_prototype_of, 2);
        self.register_native("Object.create", native_object_create, 1);
        self.register_native("Object.defineProperty", native_object_define_property, 3);
        // Object.prototype methods
        self.register_native(
            "Object.prototype.hasOwnProperty",
            native_object_has_own_property,
            1,
        );
        self.register_native("Object.prototype.toString", native_object_to_string, 0);

        // Array static methods
        self.register_native("Array.isArray", native_array_is_array, 1);

        // Function.prototype methods
        self.register_native("Function.prototype.call", native_function_call, 0);
        self.register_native("Function.prototype.apply", native_function_apply, 0);
        self.register_native("Function.prototype.bind", native_function_bind, 0);
        self.register_native("Function.prototype.toString", native_function_to_string, 0);

        // Error.prototype methods
        self.register_native("Error.prototype.toString", native_error_to_string, 0);

        // Array.prototype.toString and reduceRight
        self.register_native("Array.prototype.toString", native_array_to_string, 0);
        self.register_native("Array.prototype.reduceRight", native_array_reduce_right, 2);

        // Global utility functions
        self.register_native("gc", native_gc, 0);
        #[cfg(feature = "std")]
        self.register_native("load", native_load, 1);
        self.register_native("setTimeout", native_set_timeout, 2);
        #[cfg(feature = "std")]
        self.register_native("clearTimeout", native_clear_timeout, 1);
    }
}
