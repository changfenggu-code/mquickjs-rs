//! Arithmetic, comparison, and bitwise operators for the interpreter.
//
//! All op_* methods are pure computations on Value types.

use super::interpreter::{Interpreter, InterpreterError, InterpreterResult};
use crate::value::{float_to_value, Float, Value};
use alloc::string::ToString;

/// JS ToNumber for a subset of primitive values used by the current runtime.
/// Supports number/bool/null/undefined and numeric strings.
#[inline]
fn to_numeric(interp: &Interpreter, val: Value) -> Option<Float> {
    if let Some(n) = val.to_number_f32() {
        return Some(n);
    }
    // bool → 0 or 1
    if let Some(b) = val.to_bool() {
        return Some(if b { 1.0 } else { 0.0 });
    }
    // null → 0
    if val.is_null() {
        return Some(0.0);
    }
    // undefined → NaN
    if val.is_undefined() {
        return Some(Float::NAN);
    }
    // string → parse numeric form; invalid strings become NaN
    if let Some(str_idx) = val.to_string_idx() {
        if let Some(s) = interp.get_string_by_idx(str_idx) {
            let s = s.trim();
            if s.is_empty() {
                return Some(0.0);
            }
            if let Ok(i) = s.parse::<i32>() {
                return Some(i as Float);
            }
            if let Ok(f) = s.parse::<Float>() {
                return Some(f);
            }
            return Some(Float::NAN);
        }
        return Some(Float::NAN);
    }
    None
}

/// JS ToInt32: convert any primitive to i32 (for bitwise ops).
/// bool → 0/1, null → 0, undefined → 0, NaN/Infinity → 0.
fn to_int32(interp: &Interpreter, val: Value) -> Option<i32> {
    if let Some(f) = to_numeric(interp, val) {
        if f.is_nan() || f.is_infinite() {
            Some(0)
        } else {
            Some(f as i32)
        }
    } else {
        None
    }
}

/// Extract both operands as Float via ToNumber.
#[inline]
fn to_numeric_pair(interp: &Interpreter, a: Value, b: Value) -> Option<(Float, Float)> {
    Some((to_numeric(interp, a)?, to_numeric(interp, b)?))
}

impl Interpreter {
    // Arithmetic operations

    #[inline]
    pub(crate) fn op_neg(&self, val: Value) -> InterpreterResult<Value> {
        if let Some(n) = val.to_i32() {
            if n == 0 {
                // -0 must produce -0.0 (JS spec)
                Ok(Value::float(-0.0))
            } else {
                match n.checked_neg() {
                    Some(r) => Ok(Value::int(r)),
                    None => Ok(Value::float(-(n as Float))),
                }
            }
        } else if let Some(f) = to_numeric(self, val) {
            Ok(float_to_value(-f))
        } else {
            Err(InterpreterError::TypeError(
                "cannot negate non-number".to_string(),
            ))
        }
    }

    #[inline]
    pub(crate) fn op_add(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        // Fast path: both ints
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return match va.checked_add(vb) {
                Some(r) => Ok(Value::int(r)),
                None => Ok(Value::float(va as Float + vb as Float)),
            };
        }
        // ToNumber for bool/null/undefined/float
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            return Ok(float_to_value(fa + fb));
        }
        Err(InterpreterError::TypeError(
            "cannot add non-numbers".to_string(),
        ))
    }

    #[inline]
    pub(crate) fn op_sub(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return match va.checked_sub(vb) {
                Some(r) => Ok(Value::int(r)),
                None => Ok(Value::float(va as Float - vb as Float)),
            };
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            return Ok(float_to_value(fa - fb));
        }
        Err(InterpreterError::TypeError(
            "cannot subtract non-numbers".to_string(),
        ))
    }

    #[inline]
    pub(crate) fn op_mul(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return match va.checked_mul(vb) {
                Some(r) => Ok(Value::int(r)),
                None => Ok(Value::float(va as Float * vb as Float)),
            };
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            return Ok(float_to_value(fa * fb));
        }
        Err(InterpreterError::TypeError(
            "cannot multiply non-numbers".to_string(),
        ))
    }

    pub(crate) fn op_div(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            if fb == 0.0 {
                if fa == 0.0 || fa.is_nan() {
                    Ok(Value::nan())
                } else if fa.is_sign_positive() != fb.is_sign_negative() {
                    Ok(Value::infinity())
                } else {
                    Ok(Value::neg_infinity())
                }
            } else {
                Ok(float_to_value(fa / fb))
            }
        } else {
            Err(InterpreterError::TypeError(
                "cannot divide non-numbers".to_string(),
            ))
        }
    }

    #[inline]
    pub(crate) fn op_pow(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            Ok(float_to_value(libm::powf(fa, fb)))
        } else {
            Err(InterpreterError::TypeError(
                "cannot exponentiate non-numbers".to_string(),
            ))
        }
    }

    #[inline]
    pub(crate) fn op_mod(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            if vb == 0 {
                return Ok(Value::nan());
            } else if let Some(result) = va.checked_rem(vb) {
                return Ok(Value::int(result));
            } else {
                // i32::MIN % -1 overflows → result is -0.0 per JS spec
                return Ok(Value::float(-0.0));
            }
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            if fb == 0.0 {
                Ok(Value::nan())
            } else {
                Ok(float_to_value(fa % fb))
            }
        } else {
            Err(InterpreterError::TypeError(
                "cannot modulo non-numbers".to_string(),
            ))
        }
    }

    // Comparison operations

    #[inline]
    pub(crate) fn op_lt(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return Ok(Value::bool(va < vb));
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            // NaN comparisons always false
            return Ok(Value::bool(!fa.is_nan() && !fb.is_nan() && fa < fb));
        }
        Err(InterpreterError::TypeError(
            "cannot compare non-numbers".to_string(),
        ))
    }

    #[inline]
    pub(crate) fn op_lte(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return Ok(Value::bool(va <= vb));
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            return Ok(Value::bool(!fa.is_nan() && !fb.is_nan() && fa <= fb));
        }
        Err(InterpreterError::TypeError(
            "cannot compare non-numbers".to_string(),
        ))
    }

    #[inline]
    pub(crate) fn op_gt(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return Ok(Value::bool(va > vb));
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            return Ok(Value::bool(!fa.is_nan() && !fb.is_nan() && fa > fb));
        }
        Err(InterpreterError::TypeError(
            "cannot compare non-numbers".to_string(),
        ))
    }

    #[inline]
    pub(crate) fn op_gte(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
            return Ok(Value::bool(va >= vb));
        }
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            return Ok(Value::bool(!fa.is_nan() && !fb.is_nan() && fa >= fb));
        }
        Err(InterpreterError::TypeError(
            "cannot compare non-numbers".to_string(),
        ))
    }

    #[inline]
    pub(crate) fn op_eq(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        // 1. Same type (and same bits) — NaN !== NaN
        if a == b {
            if a.is_nan_value() {
                return Ok(Value::bool(false));
            }
            return Ok(Value::bool(true));
        }

        // 2. null == undefined (and only those two are equal to each other)
        //    null/undefined are NOT equal to anything else (not 0, not false, not "")
        if a.is_null() || a.is_undefined() || b.is_null() || b.is_undefined() {
            let both_nullish =
                (a.is_null() || a.is_undefined()) && (b.is_null() || b.is_undefined());
            return Ok(Value::bool(both_nullish));
        }

        // 3. Cross-type numeric: int(3) == float(3.0), bool==number (bool→0/1 via ToNumber)
        if let Some((fa, fb)) = to_numeric_pair(self, a, b) {
            if fa.is_nan() || fb.is_nan() {
                return Ok(Value::bool(false));
            }
            return Ok(Value::bool(fa == fb));
        }

        // 4. Everything else (object vs primitive, string vs non-string, etc.)
        Ok(Value::bool(false))
    }

    #[inline]
    pub(crate) fn op_strict_eq(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        // NaN !== NaN
        if a.is_nan_value() || b.is_nan_value() {
            return Ok(Value::bool(false));
        }
        // Internal representations may differ (inline int vs short-float),
        // but at the language level both are JavaScript Number values.
        // Therefore 3 === 3.0 should evaluate to true, matching standard JS.
        if let Some((fa, fb)) = a.to_number_f32().zip(b.to_number_f32()) {
            return Ok(Value::bool(fa == fb));
        }
        Ok(Value::bool(a == b))
    }

    pub(crate) fn op_neq(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        let eq = self.op_eq(a, b)?;
        Ok(Value::bool(!eq.to_bool().unwrap_or(false)))
    }

    pub(crate) fn op_strict_neq(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        let eq = self.op_strict_eq(a, b)?;
        Ok(Value::bool(!eq.to_bool().unwrap_or(false)))
    }

    // Bitwise operations — JS ToInt32 coercion (bool/null/undefined included)

    pub(crate) fn op_bitwise_not(&self, val: Value) -> InterpreterResult<Value> {
        if let Some(n) = to_int32(self, val) {
            Ok(Value::int(!n))
        } else {
            Err(InterpreterError::TypeError(
                "cannot apply bitwise NOT to non-number".to_string(),
            ))
        }
    }

    pub(crate) fn op_bitwise_and(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        match (to_int32(self, a), to_int32(self, b)) {
            (Some(va), Some(vb)) => Ok(Value::int(va & vb)),
            _ => Err(InterpreterError::TypeError(
                "cannot apply bitwise AND to non-numbers".to_string(),
            )),
        }
    }

    pub(crate) fn op_bitwise_or(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        match (to_int32(self, a), to_int32(self, b)) {
            (Some(va), Some(vb)) => Ok(Value::int(va | vb)),
            _ => Err(InterpreterError::TypeError(
                "cannot apply bitwise OR to non-numbers".to_string(),
            )),
        }
    }

    pub(crate) fn op_bitwise_xor(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        match (to_int32(self, a), to_int32(self, b)) {
            (Some(va), Some(vb)) => Ok(Value::int(va ^ vb)),
            _ => Err(InterpreterError::TypeError(
                "cannot apply bitwise XOR to non-numbers".to_string(),
            )),
        }
    }

    pub(crate) fn op_shl(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        match (to_int32(self, a), to_int32(self, b)) {
            (Some(va), Some(vb)) => {
                let shift = (vb & 0x1f) as u32;
                Ok(Value::int(va << shift))
            }
            _ => Err(InterpreterError::TypeError(
                "cannot apply left shift to non-numbers".to_string(),
            )),
        }
    }

    pub(crate) fn op_sar(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        match (to_int32(self, a), to_int32(self, b)) {
            (Some(va), Some(vb)) => {
                let shift = (vb & 0x1f) as u32;
                Ok(Value::int(va >> shift))
            }
            _ => Err(InterpreterError::TypeError(
                "cannot apply arithmetic right shift to non-numbers".to_string(),
            )),
        }
    }

    pub(crate) fn op_shr(&self, a: Value, b: Value) -> InterpreterResult<Value> {
        match (to_int32(self, a), to_int32(self, b)) {
            (Some(va), Some(vb)) => {
                let shift = (vb & 0x1f) as u32;
                let result = (va as u32) >> shift;
                // >>> produces unsigned result; values > i32::MAX must be float
                if result <= i32::MAX as u32 {
                    Ok(Value::int(result as i32))
                } else {
                    Ok(Value::float(result as Float))
                }
            }
            _ => Err(InterpreterError::TypeError(
                "cannot apply logical right shift to non-numbers".to_string(),
            )),
        }
    }
}
