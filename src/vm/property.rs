//! Property access dispatch for built-in types

use super::interpreter::{
    Interpreter, BUILTIN_ARRAY, BUILTIN_ARRAY_BUFFER, BUILTIN_BOOLEAN, BUILTIN_CONSOLE,
    BUILTIN_DATE, BUILTIN_ERROR, BUILTIN_FLOAT32_ARRAY, BUILTIN_FLOAT64_ARRAY, BUILTIN_GLOBAL_THIS,
    BUILTIN_INT16_ARRAY, BUILTIN_INT32_ARRAY, BUILTIN_INT8_ARRAY, BUILTIN_JSON, BUILTIN_MATH,
    BUILTIN_NUMBER, BUILTIN_OBJECT, BUILTIN_PERFORMANCE, BUILTIN_REGEXP, BUILTIN_STRING,
    BUILTIN_UINT16_ARRAY, BUILTIN_UINT32_ARRAY, BUILTIN_UINT8_ARRAY, BUILTIN_UINT8_CLAMPED_ARRAY,
};
use crate::value::Value;
use alloc::format;

impl Interpreter {
    /// Get a property from an array (Array.prototype methods or length)
    pub(crate) fn get_array_property(&self, arr: Value, prop_name: &str) -> Value {
        match prop_name {
            "length" => {
                // Return the array length
                if let Some(arr_idx) = arr.to_array_idx() {
                    if let Some(arr_data) = self.arrays.get(arr_idx as usize) {
                        return Value::int(arr_data.len() as i32);
                    }
                }
                Value::undefined()
            }
            "push" => self
                .get_native_func("Array.prototype.push")
                .unwrap_or_default(),
            "pop" => self
                .get_native_func("Array.prototype.pop")
                .unwrap_or_default(),
            "shift" => self
                .get_native_func("Array.prototype.shift")
                .unwrap_or_default(),
            "unshift" => self
                .get_native_func("Array.prototype.unshift")
                .unwrap_or_default(),
            "indexOf" => self
                .get_native_func("Array.prototype.indexOf")
                .unwrap_or_default(),
            "lastIndexOf" => self
                .get_native_func("Array.prototype.lastIndexOf")
                .unwrap_or_default(),
            "join" => self
                .get_native_func("Array.prototype.join")
                .unwrap_or_default(),
            "reverse" => self
                .get_native_func("Array.prototype.reverse")
                .unwrap_or_default(),
            "slice" => self
                .get_native_func("Array.prototype.slice")
                .unwrap_or_default(),
            "map" => self
                .get_native_func("Array.prototype.map")
                .unwrap_or_default(),
            "filter" => self
                .get_native_func("Array.prototype.filter")
                .unwrap_or_default(),
            "forEach" => self
                .get_native_func("Array.prototype.forEach")
                .unwrap_or_default(),
            "reduce" => self
                .get_native_func("Array.prototype.reduce")
                .unwrap_or_default(),
            "find" => self
                .get_native_func("Array.prototype.find")
                .unwrap_or_default(),
            "findIndex" => self
                .get_native_func("Array.prototype.findIndex")
                .unwrap_or_default(),
            "some" => self
                .get_native_func("Array.prototype.some")
                .unwrap_or_default(),
            "every" => self
                .get_native_func("Array.prototype.every")
                .unwrap_or_default(),
            "includes" => self
                .get_native_func("Array.prototype.includes")
                .unwrap_or_default(),
            "concat" => self
                .get_native_func("Array.prototype.concat")
                .unwrap_or_default(),
            "sort" => self
                .get_native_func("Array.prototype.sort")
                .unwrap_or_default(),
            "flat" => self
                .get_native_func("Array.prototype.flat")
                .unwrap_or_default(),
            "fill" => self
                .get_native_func("Array.prototype.fill")
                .unwrap_or_default(),
            "toString" => self
                .get_native_func("Array.prototype.toString")
                .unwrap_or_default(),
            "reduceRight" => self
                .get_native_func("Array.prototype.reduceRight")
                .unwrap_or_default(),
            _ => Value::undefined(),
        }
    }

    /// Get a property from a string (String.prototype methods or length)
    pub(crate) fn get_string_property(&self, str_val: Value, prop_name: &str) -> Value {
        match prop_name {
            "length" => {
                // Get string length
                if let Some(str_idx) = str_val.to_string_idx() {
                    if let Some(s) = self.get_string_by_idx(str_idx) {
                        return Value::int(s.len() as i32);
                    }
                }
                Value::int(0)
            }
            "charAt" => self
                .get_native_func("String.prototype.charAt")
                .unwrap_or_default(),
            "charCodeAt" => self
                .get_native_func("String.prototype.charCodeAt")
                .unwrap_or_default(),
            "indexOf" => self
                .get_native_func("String.prototype.indexOf")
                .unwrap_or_default(),
            "lastIndexOf" => self
                .get_native_func("String.prototype.lastIndexOf")
                .unwrap_or_default(),
            "slice" => self
                .get_native_func("String.prototype.slice")
                .unwrap_or_default(),
            "substring" => self
                .get_native_func("String.prototype.substring")
                .unwrap_or_default(),
            "toUpperCase" => self
                .get_native_func("String.prototype.toUpperCase")
                .unwrap_or_default(),
            "toLowerCase" => self
                .get_native_func("String.prototype.toLowerCase")
                .unwrap_or_default(),
            "trim" => self
                .get_native_func("String.prototype.trim")
                .unwrap_or_default(),
            "split" => self
                .get_native_func("String.prototype.split")
                .unwrap_or_default(),
            "concat" => self
                .get_native_func("String.prototype.concat")
                .unwrap_or_default(),
            "repeat" => self
                .get_native_func("String.prototype.repeat")
                .unwrap_or_default(),
            "startsWith" => self
                .get_native_func("String.prototype.startsWith")
                .unwrap_or_default(),
            "endsWith" => self
                .get_native_func("String.prototype.endsWith")
                .unwrap_or_default(),
            "padStart" => self
                .get_native_func("String.prototype.padStart")
                .unwrap_or_default(),
            "padEnd" => self
                .get_native_func("String.prototype.padEnd")
                .unwrap_or_default(),
            "replace" => self
                .get_native_func("String.prototype.replace")
                .unwrap_or_default(),
            "includes" => self
                .get_native_func("String.prototype.includes")
                .unwrap_or_default(),
            "match" => self
                .get_native_func("String.prototype.match")
                .unwrap_or_default(),
            "search" => self
                .get_native_func("String.prototype.search")
                .unwrap_or_default(),
            // mquickjs-specific String methods
            "codePointAt" => self
                .get_native_func("String.prototype.codePointAt")
                .unwrap_or_default(),
            "trimStart" => self
                .get_native_func("String.prototype.trimStart")
                .unwrap_or_default(),
            "trimEnd" => self
                .get_native_func("String.prototype.trimEnd")
                .unwrap_or_default(),
            "replaceAll" => self
                .get_native_func("String.prototype.replaceAll")
                .unwrap_or_default(),
            _ => Value::undefined(),
        }
    }

    /// Get a property from a number (Number.prototype methods)
    pub(crate) fn get_number_property(&self, _num_val: Value, prop_name: &str) -> Value {
        match prop_name {
            "toString" => self
                .get_native_func("Number.prototype.toString")
                .unwrap_or_default(),
            "toFixed" => self
                .get_native_func("Number.prototype.toFixed")
                .unwrap_or_default(),
            "toExponential" => self
                .get_native_func("Number.prototype.toExponential")
                .unwrap_or_default(),
            "toPrecision" => self
                .get_native_func("Number.prototype.toPrecision")
                .unwrap_or_default(),
            _ => Value::undefined(),
        }
    }

    /// Get a property from an error object
    pub(crate) fn get_error_property(&mut self, err_idx: u32, prop_name: &str) -> Value {
        if let Some(err) = self.error_objects.get(err_idx as usize).cloned() {
            match prop_name {
                "name" => {
                    // Return the error name as a runtime string
                    self.create_runtime_string(err.name)
                }
                "message" => {
                    // Return the error message as a runtime string
                    self.create_runtime_string(err.message)
                }
                "stack" => {
                    // Return a simple stack trace (just error type and message for now)
                    let stack = format!("{}:{}", err.name, err.message);
                    self.create_runtime_string(stack)
                }
                "toString" => self
                    .get_native_func("Error.prototype.toString")
                    .unwrap_or_default(),
                _ => Value::undefined(),
            }
        } else {
            Value::undefined()
        }
    }

    /// Get a property from a function (Function.prototype methods)
    pub(crate) fn get_function_property(&self, prop_name: &str) -> Value {
        match prop_name {
            "call" => self
                .get_native_func("Function.prototype.call")
                .unwrap_or_default(),
            "apply" => self
                .get_native_func("Function.prototype.apply")
                .unwrap_or_default(),
            "bind" => self
                .get_native_func("Function.prototype.bind")
                .unwrap_or_default(),
            "toString" => self
                .get_native_func("Function.prototype.toString")
                .unwrap_or_default(),
            _ => Value::undefined(),
        }
    }

    /// Get a property from a RegExp object
    pub(crate) fn get_regexp_property(&self, regex_idx: u32, prop_name: &str) -> Value {
        if let Some(re) = self.regex_objects.get(regex_idx as usize) {
            match prop_name {
                "test" => self
                    .get_native_func("RegExp.prototype.test")
                    .unwrap_or_default(),
                "exec" => self
                    .get_native_func("RegExp.prototype.exec")
                    .unwrap_or_default(),
                "global" => Value::bool(re.global),
                "ignoreCase" => Value::bool(re.ignore_case),
                "multiline" => Value::bool(re.multiline),
                "source" => {
                    // Return pattern as a string - but we need mutable access for runtime strings
                    // For now, just return undefined
                    Value::undefined()
                }
                _ => Value::undefined(),
            }
        } else {
            Value::undefined()
        }
    }

    /// Get a property from a typed array
    pub(crate) fn get_typed_array_property(&self, typed_idx: u32, prop_name: &str) -> Value {
        if let Some(ta) = self.typed_arrays.get(typed_idx as usize) {
            match prop_name {
                "length" => Value::int(ta.length as i32),
                "byteLength" => Value::int(ta.data.len() as i32),
                "BYTES_PER_ELEMENT" => Value::int(ta.kind.byte_size() as i32),
                "fill" => self
                    .get_native_func("TypedArray.prototype.fill")
                    .unwrap_or_default(),
                "subarray" => self
                    .get_native_func("TypedArray.prototype.subarray")
                    .unwrap_or_default(),
                _ => Value::undefined(),
            }
        } else {
            Value::undefined()
        }
    }

    /// Get a property from an ArrayBuffer
    pub(crate) fn get_array_buffer_property(&self, ab_idx: u32, prop_name: &str) -> Value {
        if let Some(ab) = self.array_buffers.get(ab_idx as usize) {
            match prop_name {
                "byteLength" => Value::int(ab.byte_length() as i32),
                _ => Value::undefined(),
            }
        } else {
            Value::undefined()
        }
    }

    /// Get a property from a builtin object (Math, JSON, etc.)
    pub(crate) fn get_builtin_property(&self, builtin_idx: u32, prop_name: &str) -> Value {
        match builtin_idx {
            BUILTIN_MATH => {
                // Math object properties
                match prop_name {
                    "abs" => self.get_native_func("Math.abs").unwrap_or_default(),
                    "floor" => self.get_native_func("Math.floor").unwrap_or_default(),
                    "ceil" => self.get_native_func("Math.ceil").unwrap_or_default(),
                    "max" => self.get_native_func("Math.max").unwrap_or_default(),
                    "min" => self.get_native_func("Math.min").unwrap_or_default(),
                    "round" => self.get_native_func("Math.round").unwrap_or_default(),
                    "sqrt" => self.get_native_func("Math.sqrt").unwrap_or_default(),
                    "pow" => self.get_native_func("Math.pow").unwrap_or_default(),
                    // mquickjs-specific Math functions
                    "imul" => self.get_native_func("Math.imul").unwrap_or_default(),
                    "clz32" => self.get_native_func("Math.clz32").unwrap_or_default(),
                    "fround" => self.get_native_func("Math.fround").unwrap_or_default(),
                    "trunc" => self.get_native_func("Math.trunc").unwrap_or_default(),
                    "log2" => self.get_native_func("Math.log2").unwrap_or_default(),
                    "log10" => self.get_native_func("Math.log10").unwrap_or_default(),
                    "sign" => self.get_native_func("Math.sign").unwrap_or_default(),
                    "sin" => self.get_native_func("Math.sin").unwrap_or_default(),
                    "cos" => self.get_native_func("Math.cos").unwrap_or_default(),
                    "tan" => self.get_native_func("Math.tan").unwrap_or_default(),
                    "exp" => self.get_native_func("Math.exp").unwrap_or_default(),
                    "log" => self.get_native_func("Math.log").unwrap_or_default(),
                    "random" => self.get_native_func("Math.random").unwrap_or_default(),
                    "atan2" => self.get_native_func("Math.atan2").unwrap_or_default(),
                    "asin" => self.get_native_func("Math.asin").unwrap_or_default(),
                    "acos" => self.get_native_func("Math.acos").unwrap_or_default(),
                    "atan" => self.get_native_func("Math.atan").unwrap_or_default(),
                    // Math constants
                    "PI" => Value::float(core::f32::consts::PI),
                    "E" => Value::float(core::f32::consts::E),
                    "LN2" => Value::float(core::f32::consts::LN_2),
                    "LN10" => Value::float(core::f32::consts::LN_10),
                    "LOG2E" => Value::float(core::f32::consts::LOG2_E),
                    "LOG10E" => Value::float(core::f32::consts::LOG10_E),
                    "SQRT2" => Value::float(core::f32::consts::SQRT_2),
                    "SQRT1_2" => Value::float(core::f32::consts::FRAC_1_SQRT_2),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_JSON => {
                // JSON object properties
                match prop_name {
                    "stringify" => self.get_native_func("JSON.stringify").unwrap_or_default(),
                    "parse" => self.get_native_func("JSON.parse").unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_NUMBER => {
                // Number object properties
                match prop_name {
                    "isInteger" => self.get_native_func("Number.isInteger").unwrap_or_default(),
                    "isNaN" => self.get_native_func("Number.isNaN").unwrap_or_default(),
                    "isFinite" => self.get_native_func("Number.isFinite").unwrap_or_default(),
                    "parseInt" => self.get_native_func("parseInt").unwrap_or_default(),
                    "MAX_VALUE" => Value::float(f32::MAX),
                    "MIN_VALUE" => Value::float(f32::MIN_POSITIVE),
                    "EPSILON" => Value::float(f32::EPSILON),
                    "MAX_SAFE_INTEGER" => Value::float(16777215.0), // 2^24 - 1 (f32 precision)
                    "MIN_SAFE_INTEGER" => Value::float(-16777215.0),
                    "NaN" => Value::nan(),
                    "POSITIVE_INFINITY" => Value::infinity(),
                    "NEGATIVE_INFINITY" => Value::neg_infinity(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_BOOLEAN => {
                // Boolean object - currently no static methods
                Value::undefined()
            }
            BUILTIN_CONSOLE => {
                // console object properties
                match prop_name {
                    "log" => self.get_native_func("console.log").unwrap_or_default(),
                    "error" => self.get_native_func("console.error").unwrap_or_default(),
                    "warn" => self.get_native_func("console.warn").unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_PERFORMANCE => {
                // performance object properties
                match prop_name {
                    "now" => self.get_native_func("performance.now").unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_DATE => {
                // Date object properties
                match prop_name {
                    "now" => self.get_native_func("Date.now").unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_OBJECT => {
                // Object static methods
                match prop_name {
                    "keys" => self.get_native_func("Object.keys").unwrap_or_default(),
                    "values" => self.get_native_func("Object.values").unwrap_or_default(),
                    "entries" => self.get_native_func("Object.entries").unwrap_or_default(),
                    "getPrototypeOf" => self
                        .get_native_func("Object.getPrototypeOf")
                        .unwrap_or_default(),
                    "setPrototypeOf" => self
                        .get_native_func("Object.setPrototypeOf")
                        .unwrap_or_default(),
                    "create" => self.get_native_func("Object.create").unwrap_or_default(),
                    "defineProperty" => self
                        .get_native_func("Object.defineProperty")
                        .unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_ARRAY => {
                // Array static methods
                match prop_name {
                    "isArray" => self.get_native_func("Array.isArray").unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_STRING => {
                // String static methods
                match prop_name {
                    "fromCharCode" => self
                        .get_native_func("String.fromCharCode")
                        .unwrap_or_default(),
                    "fromCodePoint" => self
                        .get_native_func("String.fromCodePoint")
                        .unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            BUILTIN_GLOBAL_THIS => {
                // globalThis provides access to global builtins
                match prop_name {
                    "undefined" => Value::undefined(),
                    "NaN" => Value::nan(),
                    "Infinity" => Value::infinity(),
                    "Math" => Value::builtin_object(BUILTIN_MATH),
                    "JSON" => Value::builtin_object(BUILTIN_JSON),
                    "Number" => Value::builtin_object(BUILTIN_NUMBER),
                    "Boolean" => Value::builtin_object(BUILTIN_BOOLEAN),
                    "String" => Value::builtin_object(BUILTIN_STRING),
                    "Object" => Value::builtin_object(BUILTIN_OBJECT),
                    "Array" => Value::builtin_object(BUILTIN_ARRAY),
                    "console" => Value::builtin_object(BUILTIN_CONSOLE),
                    "performance" => Value::builtin_object(BUILTIN_PERFORMANCE),
                    "Date" => Value::builtin_object(BUILTIN_DATE),
                    "Error" => Value::builtin_object(BUILTIN_ERROR),
                    "RegExp" => Value::builtin_object(BUILTIN_REGEXP),
                    "globalThis" => Value::builtin_object(BUILTIN_GLOBAL_THIS),
                    "ArrayBuffer" => Value::builtin_object(BUILTIN_ARRAY_BUFFER),
                    "Int8Array" => Value::builtin_object(BUILTIN_INT8_ARRAY),
                    "Uint8Array" => Value::builtin_object(BUILTIN_UINT8_ARRAY),
                    "Uint8ClampedArray" => Value::builtin_object(BUILTIN_UINT8_CLAMPED_ARRAY),
                    "Int16Array" => Value::builtin_object(BUILTIN_INT16_ARRAY),
                    "Uint16Array" => Value::builtin_object(BUILTIN_UINT16_ARRAY),
                    "Int32Array" => Value::builtin_object(BUILTIN_INT32_ARRAY),
                    "Uint32Array" => Value::builtin_object(BUILTIN_UINT32_ARRAY),
                    "Float32Array" => Value::builtin_object(BUILTIN_FLOAT32_ARRAY),
                    "Float64Array" => Value::builtin_object(BUILTIN_FLOAT64_ARRAY),
                    "parseInt" => self.get_native_func("parseInt").unwrap_or_default(),
                    "parseFloat" => self.get_native_func("parseFloat").unwrap_or_default(),
                    "isNaN" => self.get_native_func("isNaN").unwrap_or_default(),
                    "isFinite" => self.get_native_func("isFinite").unwrap_or_default(),
                    "gc" => self.get_native_func("gc").unwrap_or_default(),
                    "load" => self.get_native_func("load").unwrap_or_default(),
                    "setTimeout" => self.get_native_func("setTimeout").unwrap_or_default(),
                    "clearTimeout" => self.get_native_func("clearTimeout").unwrap_or_default(),
                    _ => Value::undefined(),
                }
            }
            _ => Value::undefined(),
        }
    }
}
