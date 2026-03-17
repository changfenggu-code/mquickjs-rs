//! Data types used by the interpreter.
//
//! Constants, structs, and enums for objects, closures, call frames,
//! error objects, regex, typed arrays, and interpreter statistics.

use crate::runtime::FunctionBytecode;
use crate::value::{float_to_value, Value};
use crate::vm::stack::Stack;
use alloc::{string::String, vec, vec::Vec};

// Builtin object indices
/// Math object index
pub const BUILTIN_MATH: u32 = 0;
/// JSON object index (for future use)
pub const BUILTIN_JSON: u32 = 1;
/// Number object index
pub const BUILTIN_NUMBER: u32 = 2;
/// Boolean object index
pub const BUILTIN_BOOLEAN: u32 = 3;
/// console object index
pub const BUILTIN_CONSOLE: u32 = 4;
/// Error constructor index
pub const BUILTIN_ERROR: u32 = 5;
/// TypeError constructor index
pub const BUILTIN_TYPE_ERROR: u32 = 6;
/// ReferenceError constructor index
pub const BUILTIN_REFERENCE_ERROR: u32 = 7;
/// SyntaxError constructor index
pub const BUILTIN_SYNTAX_ERROR: u32 = 8;
/// RangeError constructor index
pub const BUILTIN_RANGE_ERROR: u32 = 9;
/// EvalError constructor index
pub const BUILTIN_EVAL_ERROR: u32 = 27;
/// URIError constructor index
pub const BUILTIN_URI_ERROR: u32 = 28;
/// InternalError constructor index
pub const BUILTIN_INTERNAL_ERROR: u32 = 29;
/// Date object index
pub const BUILTIN_DATE: u32 = 10;
/// String object index
pub const BUILTIN_STRING: u32 = 11;
/// Object object index
pub const BUILTIN_OBJECT: u32 = 12;
/// Array object index
pub const BUILTIN_ARRAY: u32 = 13;
/// RegExp object index
pub const BUILTIN_REGEXP: u32 = 14;
/// globalThis object index
pub const BUILTIN_GLOBAL_THIS: u32 = 15;
/// ArrayBuffer constructor index
pub const BUILTIN_ARRAY_BUFFER: u32 = 16;
/// Int8Array constructor index
pub const BUILTIN_INT8_ARRAY: u32 = 17;
/// Uint8Array constructor index
pub const BUILTIN_UINT8_ARRAY: u32 = 18;
/// Int16Array constructor index
pub const BUILTIN_INT16_ARRAY: u32 = 19;
/// Uint16Array constructor index
pub const BUILTIN_UINT16_ARRAY: u32 = 20;
/// Int32Array constructor index
pub const BUILTIN_INT32_ARRAY: u32 = 21;
/// Uint32Array constructor index
pub const BUILTIN_UINT32_ARRAY: u32 = 22;
/// Performance object index
pub const BUILTIN_PERFORMANCE: u32 = 23;
/// Uint8ClampedArray constructor index
pub const BUILTIN_UINT8_CLAMPED_ARRAY: u32 = 24;
/// Float32Array constructor index
pub const BUILTIN_FLOAT32_ARRAY: u32 = 25;
/// Float64Array constructor index
pub const BUILTIN_FLOAT64_ARRAY: u32 = 26;

/// Native function signature
///
/// Native functions take an interpreter reference, this value, and arguments.
/// Returns a Result with the value or an error message.
pub type NativeFn =
    fn(interp: &mut Interpreter, this: Value, args: &[Value]) -> Result<Value, String>;

/// Native function entry in the registry
#[derive(Clone)]
pub struct NativeFunction {
    /// The name of the function
    pub name: &'static str,
    /// The native function implementation
    pub func: NativeFn,
    /// Number of expected arguments (for arity checking, 0 = variadic)
    pub arity: u8,
}

/// Object instance storing properties and constructor reference
#[derive(Debug, Clone)]
pub struct ObjectInstance {
    /// Constructor that created this object (closure index), if any
    pub constructor: Option<Value>,
    /// Object properties as key-value pairs
    pub properties: Vec<(String, Value)>,
}

impl Default for ObjectInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectInstance {
    /// Create a new empty object
    pub fn new() -> Self {
        ObjectInstance {
            constructor: None,
            properties: Vec::new(),
        }
    }

    /// Create a new object with a constructor reference
    pub fn with_constructor(constructor: Value) -> Self {
        ObjectInstance {
            constructor: Some(constructor),
            properties: Vec::new(),
        }
    }
}

/// For-in iterator state
#[derive(Debug, Clone)]
pub enum ForInIterator {
    /// Iterate object keys by index with snapshot length
    Object {
        obj_idx: u32,
        len: usize,
        index: usize,
    },
    /// Iterate array indices with snapshot length
    Array { len: usize, index: usize },
    /// Empty iterator
    Empty,
}

impl ForInIterator {
    /// Create a new for-in iterator from an object index and property count snapshot
    pub fn from_object_idx(obj_idx: u32, len: usize) -> Self {
        ForInIterator::Object {
            obj_idx,
            len,
            index: 0,
        }
    }

    /// Create a new for-in iterator from an array length snapshot
    pub fn from_array_len(len: usize) -> Self {
        ForInIterator::Array { len, index: 0 }
    }

    /// Create an empty iterator
    pub fn empty() -> Self {
        ForInIterator::Empty
    }
}

/// For-of iterator state (iterates over values)
#[derive(Debug, Clone)]
pub enum ForOfIterator {
    /// Iterate directly over an array stored in the interpreter
    Array { arr_idx: u32, index: usize },
    /// Iterate over a captured list of values
    Values { values: Vec<Value>, index: usize },
}

impl ForOfIterator {
    /// Create a new for-of iterator from an array index
    pub fn from_array_idx(arr_idx: u32) -> Self {
        ForOfIterator::Array { arr_idx, index: 0 }
    }

    /// Create a new for-of iterator from an object (iterates over property values)
    pub fn from_object(obj: &ObjectInstance) -> Self {
        let values = obj.properties.iter().map(|(_, v)| *v).collect();
        ForOfIterator::Values { values, index: 0 }
    }

    /// Create an empty for-of iterator
    pub fn empty() -> Self {
        ForOfIterator::Values {
            values: Vec::new(),
            index: 0,
        }
    }
}

/// Closure data storing captured variable values
#[derive(Debug, Clone)]
pub struct ClosureData {
    /// Reference to the function bytecode
    pub bytecode: *const FunctionBytecode,
    /// Indices into interpreter's var_cells pool (shared mutable cells)
    pub cell_indices: Vec<u32>,
}

impl ClosureData {
    /// Create a new closure with cell indices
    pub fn new(bytecode: *const FunctionBytecode, cell_indices: Vec<u32>) -> Self {
        ClosureData {
            bytecode,
            cell_indices,
        }
    }

    /// Get the cell index for a captured variable
    pub fn get_cell_index(&self, index: usize) -> Option<u32> {
        self.cell_indices.get(index).copied()
    }
}

/// Call frame information
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Function bytecode being executed
    pub bytecode: *const FunctionBytecode,
    /// Program counter (offset into bytecode)
    pub pc: usize,
    /// Frame pointer (index into stack where locals start)
    pub frame_ptr: usize,
    /// Number of arguments
    pub arg_count: u16,
    /// Return address (pc to return to, or usize::MAX for top-level)
    pub return_pc: usize,
    /// Previous frame pointer
    pub prev_frame_ptr: usize,
    /// `this` value for this call
    pub this_val: Value,
    /// The function value itself (for self-reference/recursion)
    pub this_func: Value,
    /// Index into closures array if this frame is executing a closure
    pub closure_idx: Option<usize>,
    /// Whether this is a constructor call (new operator)
    pub is_constructor: bool,
    /// Maps local indices to var_cells indices (None = not captured, Some(idx) = cell index).
    /// Lazily allocated when the first local in this frame is captured.
    pub local_cells: Option<Vec<Option<u32>>>,
}

impl CallFrame {
    /// Create a new call frame
    pub fn new(
        bytecode: *const FunctionBytecode,
        frame_ptr: usize,
        arg_count: u16,
        this_val: Value,
        this_func: Value,
    ) -> Self {
        CallFrame {
            bytecode,
            pc: 0,
            frame_ptr,
            arg_count,
            return_pc: usize::MAX,
            prev_frame_ptr: 0,
            this_val,
            this_func,
            closure_idx: None,
            is_constructor: false,
            local_cells: None,
        }
    }

    /// Create a call frame for a closure
    pub fn new_closure(
        bytecode: *const FunctionBytecode,
        frame_ptr: usize,
        arg_count: u16,
        this_val: Value,
        this_func: Value,
        closure_idx: usize,
    ) -> Self {
        CallFrame {
            bytecode,
            pc: 0,
            frame_ptr,
            arg_count,
            return_pc: usize::MAX,
            prev_frame_ptr: 0,
            this_val,
            this_func,
            closure_idx: Some(closure_idx),
            is_constructor: false,
            local_cells: None,
        }
    }

    /// Create a call frame for a constructor
    pub fn new_constructor(
        bytecode: *const FunctionBytecode,
        frame_ptr: usize,
        arg_count: u16,
        this_val: Value,
        this_func: Value,
    ) -> Self {
        CallFrame {
            bytecode,
            pc: 0,
            frame_ptr,
            arg_count,
            return_pc: usize::MAX,
            prev_frame_ptr: 0,
            this_val,
            this_func,
            closure_idx: None,
            is_constructor: true,
            local_cells: None,
        }
    }

    /// Create a call frame for a closure used as constructor
    pub fn new_closure_constructor(
        bytecode: *const FunctionBytecode,
        frame_ptr: usize,
        arg_count: u16,
        this_val: Value,
        this_func: Value,
        closure_idx: usize,
    ) -> Self {
        CallFrame {
            bytecode,
            pc: 0,
            frame_ptr,
            arg_count,
            return_pc: usize::MAX,
            prev_frame_ptr: 0,
            this_val,
            this_func,
            closure_idx: Some(closure_idx),
            is_constructor: true,
            local_cells: None,
        }
    }
}

/// Interpreter error
#[derive(Debug, Clone)]
pub enum InterpreterError {
    /// Stack underflow
    StackUnderflow,
    /// Stack overflow
    StackOverflow,
    /// Invalid opcode
    InvalidOpcode(u8),
    /// Division by zero
    DivisionByZero,
    /// Type error
    TypeError(String),
    /// Reference error
    ReferenceError(String),
    /// Internal error
    InternalError(String),
    /// Uncaught JS exception (formatted message from Error object or primitive)
    UncaughtException(String),
}

impl core::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::StackOverflow => write!(f, "stack overflow"),
            Self::InvalidOpcode(op) => write!(f, "invalid opcode: {}", op),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::TypeError(msg) => write!(f, "TypeError: {}", msg),
            Self::ReferenceError(msg) => write!(f, "ReferenceError: {}", msg),
            Self::InternalError(msg) => write!(f, "InternalError: {}", msg),
            Self::UncaughtException(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InterpreterError {}

/// Result type for interpreter operations
pub type InterpreterResult<T> = Result<T, InterpreterError>;

/// Exception handler info
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    /// Call stack depth when handler was registered
    pub frame_depth: usize,
    /// Program counter to jump to when exception is caught
    pub catch_pc: usize,
    /// Stack depth when handler was registered (to restore stack)
    pub stack_depth: usize,
}

/// Interpreter state
pub struct Interpreter {
    /// Value stack
    pub(crate) stack: Stack,
    /// Call stack (frames)
    pub(crate) call_stack: Vec<CallFrame>,
    /// Maximum call recursion depth
    pub(crate) max_recursion: usize,
    /// Runtime strings (created during execution, e.g., from concatenation)
    /// Indices start from 0x8000 to distinguish from compile-time strings
    pub(crate) runtime_strings: Vec<String>,
    /// Closures created during execution
    /// Values on the stack can reference closures by index
    pub(crate) closures: Vec<ClosureData>,
    /// Shared mutable variable cells for closure captures.
    /// Multiple closures capturing the same variable share the same cell index.
    pub(crate) var_cells: Vec<Value>,
    /// Exception handler stack
    pub(crate) exception_handlers: Vec<ExceptionHandler>,
    /// Arrays created during execution
    /// Values on the stack can reference arrays by index
    pub(crate) arrays: Vec<Vec<Value>>,
    /// Objects created during execution
    /// Values on the stack can reference objects by index
    pub(crate) objects: Vec<ObjectInstance>,
    /// For-in iterators created during execution
    pub(crate) for_in_iterators: Vec<ForInIterator>,
    /// For-of iterators created during execution
    pub(crate) for_of_iterators: Vec<ForOfIterator>,
    /// Native function registry
    pub(crate) native_functions: Vec<NativeFunction>,
    /// Cached native index for Array.prototype.push
    pub(crate) native_array_push_idx: Option<u32>,
    /// Cached native index for Array.prototype.map
    pub(crate) native_array_map_idx: Option<u32>,
    /// Cached native index for Array.prototype.filter
    pub(crate) native_array_filter_idx: Option<u32>,
    /// Cached native index for Array.prototype.reduce
    pub(crate) native_array_reduce_idx: Option<u32>,
    /// Global variables set by top-level function declarations (SetGlobal opcode)
    pub(crate) global_vars: Vec<(String, Value)>,
    /// Error objects created during execution
    /// Stores (error_type, message) pairs
    pub(crate) error_objects: Vec<ErrorObject>,
    /// RegExp objects created during execution
    pub(crate) regex_objects: Vec<RegExpObject>,
    /// TypedArray objects created during execution
    pub(crate) typed_arrays: Vec<TypedArrayObject>,
    /// ArrayBuffer objects created during execution
    pub(crate) array_buffers: Vec<ArrayBufferObject>,
    /// Current compile-time string constants (set during bytecode execution)
    /// Used by native functions to look up compile-time strings
    pub(crate) current_string_constants: Option<*const Vec<String>>,
    /// Target call stack depth for nested call_value invocations
    /// When set, do_return will return early when reaching this depth
    pub(crate) nested_call_target_depth: Option<usize>,
    /// Pending timers (setTimeout callbacks)
    pub(crate) timers: Vec<Timer>,
    /// Next timer ID
    pub(crate) next_timer_id: u32,
    /// GC stats
    pub(crate) gc_count: u32,
    /// PRNG seed for Math.random() (no_std compatible)
    pub(crate) random_seed: u64,
    /// Runtime string source counters (dump-only)
    #[cfg(feature = "dump")]
    pub(crate) runtime_string_source_stats: RuntimeStringSourceStats,
    /// Runtime opcode execution counters (enabled only for dump/profiling work)
    #[cfg(feature = "dump")]
    pub(crate) opcode_counts: [u64; 256],
}

/// Error object storage
#[derive(Debug, Clone)]
pub struct ErrorObject {
    /// Error type name (e.g., "Error", "TypeError")
    pub name: String,
    /// Error message
    pub message: String,
}
/// RegExp object storage
#[cfg(feature = "std")]
#[derive(Clone)]
pub struct RegExpObject {
    /// The compiled regex pattern
    pub regex: regex::Regex,
    /// Original pattern string
    pub pattern: String,
    /// Flags string (e.g., "gi")
    pub flags: String,
    /// Global flag
    pub global: bool,
    /// Case-insensitive flag
    pub ignore_case: bool,
    /// Multiline flag
    pub multiline: bool,
}

/// RegExp object storage (stub for no_std)
#[cfg(not(feature = "std"))]
#[derive(Debug, Clone)]
pub struct RegExpObject {
    /// Original pattern string
    pub pattern: String,
    /// Flags string (e.g., "gi")
    pub flags: String,
    /// Global flag
    pub global: bool,
    /// Case-insensitive flag
    pub ignore_case: bool,
    /// Multiline flag
    pub multiline: bool,
}

#[cfg(feature = "std")]
impl core::fmt::Debug for RegExpObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RegExpObject")
            .field("pattern", &self.pattern)
            .field("flags", &self.flags)
            .finish()
    }
}

/// TypedArray element type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
}

impl TypedArrayKind {
    /// Get the byte size of each element
    pub fn byte_size(&self) -> usize {
        match self {
            TypedArrayKind::Int8 | TypedArrayKind::Uint8 | TypedArrayKind::Uint8Clamped => 1,
            TypedArrayKind::Int16 | TypedArrayKind::Uint16 => 2,
            TypedArrayKind::Int32 | TypedArrayKind::Uint32 | TypedArrayKind::Float32 => 4,
            TypedArrayKind::Float64 => 8,
        }
    }
}

/// TypedArray object - stores typed array data
#[derive(Debug, Clone)]
pub struct TypedArrayObject {
    /// The kind of typed array
    pub kind: TypedArrayKind,
    /// Raw byte data storage
    pub data: Vec<u8>,
    /// Length in elements (not bytes)
    pub length: usize,
}

impl TypedArrayObject {
    /// Create a new typed array with given length
    pub fn new(kind: TypedArrayKind, length: usize) -> Self {
        let byte_len = length * kind.byte_size();
        TypedArrayObject {
            kind,
            data: vec![0u8; byte_len],
            length,
        }
    }

    /// Get element at index as Value
    pub fn get(&self, index: usize) -> Option<Value> {
        if index >= self.length {
            return None;
        }
        let byte_offset = index * self.kind.byte_size();
        Some(match self.kind {
            TypedArrayKind::Int8 => Value::int(self.data[byte_offset] as i8 as i32),
            TypedArrayKind::Uint8 | TypedArrayKind::Uint8Clamped => {
                Value::int(self.data[byte_offset] as i32)
            }
            TypedArrayKind::Int16 => {
                let bytes = [self.data[byte_offset], self.data[byte_offset + 1]];
                Value::int(i16::from_le_bytes(bytes) as i32)
            }
            TypedArrayKind::Uint16 => {
                let bytes = [self.data[byte_offset], self.data[byte_offset + 1]];
                Value::int(u16::from_le_bytes(bytes) as i32)
            }
            TypedArrayKind::Int32 => {
                let bytes = [
                    self.data[byte_offset],
                    self.data[byte_offset + 1],
                    self.data[byte_offset + 2],
                    self.data[byte_offset + 3],
                ];
                Value::int(i32::from_le_bytes(bytes))
            }
            TypedArrayKind::Uint32 => {
                let bytes = [
                    self.data[byte_offset],
                    self.data[byte_offset + 1],
                    self.data[byte_offset + 2],
                    self.data[byte_offset + 3],
                ];
                let value = u32::from_le_bytes(bytes);
                if value <= i32::MAX as u32 {
                    Value::int(value as i32)
                } else {
                    float_to_value(value as crate::value::Float)
                }
            }
            TypedArrayKind::Float32 => {
                let bytes = [
                    self.data[byte_offset],
                    self.data[byte_offset + 1],
                    self.data[byte_offset + 2],
                    self.data[byte_offset + 3],
                ];
                float_to_value(f32::from_le_bytes(bytes))
            }
            TypedArrayKind::Float64 => {
                let bytes = [
                    self.data[byte_offset],
                    self.data[byte_offset + 1],
                    self.data[byte_offset + 2],
                    self.data[byte_offset + 3],
                    self.data[byte_offset + 4],
                    self.data[byte_offset + 5],
                    self.data[byte_offset + 6],
                    self.data[byte_offset + 7],
                ];
                Self::float64_to_value(f64::from_le_bytes(bytes))
            }
        })
    }

    /// Set element at index
    pub fn set(&mut self, index: usize, value: Value) -> bool {
        if index >= self.length {
            return false;
        }
        let byte_offset = index * self.kind.byte_size();
        let float_val = value.to_number_f32().unwrap_or(0.0);
        let int_val = float_val as i32;
        match self.kind {
            TypedArrayKind::Int8 => {
                self.data[byte_offset] = int_val as i8 as u8;
            }
            TypedArrayKind::Uint8 => {
                self.data[byte_offset] = int_val as u8;
            }
            TypedArrayKind::Uint8Clamped => {
                let clamped = int_val.clamp(0, 255) as u8;
                self.data[byte_offset] = clamped;
            }
            TypedArrayKind::Int16 => {
                let bytes = (int_val as i16).to_le_bytes();
                self.data[byte_offset] = bytes[0];
                self.data[byte_offset + 1] = bytes[1];
            }
            TypedArrayKind::Uint16 => {
                let bytes = (int_val as u16).to_le_bytes();
                self.data[byte_offset] = bytes[0];
                self.data[byte_offset + 1] = bytes[1];
            }
            TypedArrayKind::Int32 => {
                let bytes = int_val.to_le_bytes();
                self.data[byte_offset..byte_offset + 4].copy_from_slice(&bytes);
            }
            TypedArrayKind::Uint32 => {
                let mut uint_val = if let Some(int_val) = value.to_i32() {
                    int_val as u32
                } else {
                    0
                };
                if value.to_i32().is_none() {
                    if let Some(float_val) = value.to_f32() {
                        if float_val.is_finite() {
                            let truncated = libm::truncf(float_val);
                            let modulus = 4_294_967_296.0_f32;
                            let mut wrapped = truncated % modulus;
                            if wrapped < 0.0 {
                                wrapped += modulus;
                            }
                            uint_val = wrapped as u32;
                        }
                    }
                }
                let bytes = uint_val.to_le_bytes();
                self.data[byte_offset..byte_offset + 4].copy_from_slice(&bytes);
            }
            TypedArrayKind::Float32 => {
                let bytes = float_val.to_le_bytes();
                self.data[byte_offset..byte_offset + 4].copy_from_slice(&bytes);
            }
            TypedArrayKind::Float64 => {
                let float64_val = if let Some(int_val) = value.to_i32() {
                    int_val as f64
                } else {
                    value.to_f32().map(f64::from).unwrap_or(0.0)
                };
                let bytes = float64_val.to_le_bytes();
                self.data[byte_offset..byte_offset + 8].copy_from_slice(&bytes);
            }
        }
        true
    }

    #[inline]
    fn float64_to_value(value: f64) -> Value {
        if value.is_finite()
            && (value - libm::trunc(value)) == 0.0
            && value >= i32::MIN as f64
            && value <= i32::MAX as f64
            && !(value == 0.0 && value.is_sign_negative())
        {
            Value::int(value as i32)
        } else {
            float_to_value(value as f32)
        }
    }

    /// Create a subarray view into this typed array
    pub fn subarray(&self, start: i32, end: Option<i32>) -> TypedArrayObject {
        let len = self.length as i32;

        // Handle negative indices
        let start = if start < 0 {
            (len + start).max(0) as usize
        } else {
            (start as usize).min(self.length)
        };

        let end = match end {
            Some(e) if e < 0 => (len + e).max(0) as usize,
            Some(e) => (e as usize).min(self.length),
            None => self.length,
        };

        let new_len = end.saturating_sub(start);
        let byte_size = self.kind.byte_size();
        let start_offset = start * byte_size;
        let end_offset = start_offset + new_len * byte_size;

        TypedArrayObject {
            kind: self.kind,
            data: self.data[start_offset..end_offset].to_vec(),
            length: new_len,
        }
    }
}

/// ArrayBuffer object - raw binary data buffer
#[derive(Debug, Clone)]
pub struct ArrayBufferObject {
    /// Raw byte data
    pub data: Vec<u8>,
}

impl ArrayBufferObject {
    /// Create a new ArrayBuffer with the given byte length
    pub fn new(byte_length: usize) -> Self {
        ArrayBufferObject {
            data: vec![0u8; byte_length],
        }
    }

    /// Get the byte length
    pub fn byte_length(&self) -> usize {
        self.data.len()
    }
}

/// Timer for setTimeout/setInterval
#[derive(Debug, Clone)]
pub struct Timer {
    /// Timer ID
    pub id: u32,
    /// Callback function
    pub callback: Value,
    /// When the timer should fire (milliseconds since start)
    pub fire_at: u64,
    /// Whether this timer has been cancelled
    pub cancelled: bool,
}

/// Statistics about interpreter memory usage
#[derive(Debug, Clone, Default)]
pub struct InterpreterStats {
    /// Number of runtime strings
    pub runtime_strings: usize,
    /// Total bytes of runtime string contents
    pub runtime_string_bytes: usize,
    /// Number of arrays
    pub arrays: usize,
    /// Total number of array elements across all arrays
    pub array_elements: usize,
    /// Number of objects
    pub objects: usize,
    /// Total number of object properties across all objects
    pub object_properties: usize,
    /// Number of closures
    pub closures: usize,
    /// Number of error objects
    pub error_objects: usize,
    /// Number of regex objects
    pub regex_objects: usize,
    /// Number of typed arrays
    pub typed_arrays: usize,
    /// Total bytes held by typed arrays
    pub typed_array_bytes: usize,
    /// Number of array buffers
    pub array_buffers: usize,
    /// Total bytes held by array buffers
    pub array_buffer_bytes: usize,
}

/// Runtime string creation counters for profiling (`dump` feature only)
#[cfg(feature = "dump")]
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeStringSourceStats {
    pub total: u64,
    pub concat: u64,
    pub for_in_key: u64,
    pub json: u64,
    pub object_keys: u64,
    pub object_entries: u64,
    pub error_string: u64,
    pub type_string: u64,
    pub other: u64,
}
