//! Unit tests for runtime types: JSArray, function types, object types, PropertyTable.
//!
//! Migrated from src/runtime/array.rs, src/runtime/function.rs,
//! src/runtime/object.rs, and src/runtime/property.rs.

use mquickjs::gc::MemoryTag;
use mquickjs::runtime::JSArray;
use mquickjs::runtime::{
    CFunction, ClassId, Closure, FunctionBytecode, FunctionKind, ObjectHeader, Property,
    PropertyTable, PropertyType, VarRef,
};
use mquickjs::Value;

// ---------------------------------------------------------------------------
// JSArray tests
// ---------------------------------------------------------------------------

#[test]
fn test_array_new() {
    let arr = JSArray::new();
    assert!(arr.is_empty());
    assert_eq!(arr.len(), 0);
}

#[test]
fn test_array_with_length() {
    let arr = JSArray::with_length(5);
    assert_eq!(arr.len(), 5);
    assert!(arr.get(0).unwrap().is_undefined());
}

#[test]
fn test_array_push_pop() {
    let mut arr = JSArray::new();

    arr.push(Value::int(1));
    arr.push(Value::int(2));
    arr.push(Value::int(3));

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.pop(), Some(Value::int(3)));
    assert_eq!(arr.pop(), Some(Value::int(2)));
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_array_get_set() {
    let mut arr = JSArray::new();

    arr.set(0, Value::int(10));
    arr.set(2, Value::int(30));

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get(0), Some(Value::int(10)));
    assert!(arr.get(1).unwrap().is_undefined());
    assert_eq!(arr.get(2), Some(Value::int(30)));
    assert_eq!(arr.get(3), None);
}

#[test]
fn test_array_shift_unshift() {
    let mut arr = JSArray::from_values(vec![Value::int(1), Value::int(2), Value::int(3)]);

    assert_eq!(arr.shift(), Some(Value::int(1)));
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.get(0), Some(Value::int(2)));

    arr.unshift(&[Value::int(0), Value::int(1)]);
    assert_eq!(arr.len(), 4);
    assert_eq!(arr.get(0), Some(Value::int(0)));
    assert_eq!(arr.get(1), Some(Value::int(1)));
}

#[test]
fn test_array_slice() {
    let arr = JSArray::from_values(vec![
        Value::int(0),
        Value::int(1),
        Value::int(2),
        Value::int(3),
        Value::int(4),
    ]);

    let slice = arr.slice(1, 4);
    assert_eq!(slice.len(), 3);
    assert_eq!(slice.get(0), Some(Value::int(1)));
    assert_eq!(slice.get(2), Some(Value::int(3)));

    // Negative indices
    let slice = arr.slice(-2, -1);
    assert_eq!(slice.len(), 1);
    assert_eq!(slice.get(0), Some(Value::int(3)));
}

#[test]
fn test_array_splice() {
    let mut arr = JSArray::from_values(vec![
        Value::int(0),
        Value::int(1),
        Value::int(2),
        Value::int(3),
    ]);

    let removed = arr.splice(1, 2, &[Value::int(10), Value::int(20), Value::int(30)]);

    assert_eq!(removed.len(), 2);
    assert_eq!(removed.get(0), Some(Value::int(1)));
    assert_eq!(removed.get(1), Some(Value::int(2)));

    assert_eq!(arr.len(), 5);
    assert_eq!(arr.get(0), Some(Value::int(0)));
    assert_eq!(arr.get(1), Some(Value::int(10)));
    assert_eq!(arr.get(2), Some(Value::int(20)));
    assert_eq!(arr.get(3), Some(Value::int(30)));
    assert_eq!(arr.get(4), Some(Value::int(3)));
}

#[test]
fn test_array_reverse() {
    let mut arr = JSArray::from_values(vec![Value::int(1), Value::int(2), Value::int(3)]);

    arr.reverse();

    assert_eq!(arr.get(0), Some(Value::int(3)));
    assert_eq!(arr.get(1), Some(Value::int(2)));
    assert_eq!(arr.get(2), Some(Value::int(1)));
}

#[test]
fn test_array_concat() {
    let arr1 = JSArray::from_values(vec![Value::int(1), Value::int(2)]);
    let arr2 = JSArray::from_values(vec![Value::int(3), Value::int(4)]);

    let result = arr1.concat(&arr2).unwrap();
    assert_eq!(result.len(), 4);
    assert_eq!(result.get(2), Some(Value::int(3)));
}

#[test]
fn test_array_index_of() {
    let arr = JSArray::from_values(vec![
        Value::int(1),
        Value::int(2),
        Value::int(3),
        Value::int(2),
    ]);

    assert_eq!(arr.index_of(Value::int(2), 0), Some(1));
    assert_eq!(arr.index_of(Value::int(2), 2), Some(3));
    assert_eq!(arr.index_of(Value::int(5), 0), None);

    assert_eq!(arr.last_index_of(Value::int(2), 3), Some(3));
    assert_eq!(arr.last_index_of(Value::int(2), 2), Some(1));
}

#[test]
fn test_array_set_length() {
    let mut arr = JSArray::from_values(vec![Value::int(1), Value::int(2), Value::int(3)]);

    arr.set_length(5);
    assert_eq!(arr.len(), 5);
    assert!(arr.get(3).unwrap().is_undefined());

    arr.set_length(1);
    assert_eq!(arr.len(), 1);
    assert_eq!(arr.get(0), Some(Value::int(1)));
}

// ---------------------------------------------------------------------------
// Function type tests
// ---------------------------------------------------------------------------

#[test]
fn test_c_function() {
    let cfunc = CFunction::new(42);
    assert_eq!(cfunc.idx, 42);
    assert!(cfunc.params.is_undefined());

    let cfunc = CFunction::with_params(10, Value::int(5));
    assert_eq!(cfunc.idx, 10);
    assert_eq!(cfunc.params, Value::int(5));
}

#[test]
fn test_var_ref() {
    let mut var_ref = VarRef::attached(5);
    assert!(!var_ref.is_detached);

    var_ref.detach(Value::int(100));
    assert!(var_ref.is_detached);
    assert_eq!(var_ref.value, Value::int(100));
}

#[test]
fn test_closure() {
    let bytecode = Value::null(); // Placeholder
    let var_refs = vec![
        VarRef::detached(Value::int(1)),
        VarRef::detached(Value::int(2)),
    ];

    let closure = Closure::with_var_refs(bytecode, var_refs);
    assert_eq!(closure.var_refs.len(), 2);
}

#[test]
fn test_function_bytecode() {
    let mut fb = FunctionBytecode::new(2, 3);
    fb.set_name("myFunction");

    assert_eq!(fb.arg_count, 2);
    assert_eq!(fb.local_count, 3);
    assert_eq!(fb.name, Some("myFunction".to_string()));

    let idx = fb.add_constant(Value::int(42));
    assert_eq!(fb.get_constant(idx), Some(Value::int(42)));
}

#[test]
fn test_bytecode_emit() {
    let mut fb = FunctionBytecode::new(0, 0);

    fb.emit_u8(0x01);
    fb.emit_u16(0x1234);
    fb.emit_u32(0x12345678);

    assert_eq!(fb.bytecode.len(), 7);
    assert_eq!(fb.bytecode[0], 0x01);
    assert_eq!(fb.bytecode[1], 0x34);
    assert_eq!(fb.bytecode[2], 0x12);
}

#[test]
fn test_line_numbers() {
    let mut fb = FunctionBytecode::new(0, 0);

    fb.add_line_number(0, 1);
    fb.add_line_number(10, 5);
    fb.add_line_number(20, 10);

    assert_eq!(fb.get_line_number(0), Some(1));
    assert_eq!(fb.get_line_number(5), Some(1));
    assert_eq!(fb.get_line_number(10), Some(5));
    assert_eq!(fb.get_line_number(15), Some(5));
    assert_eq!(fb.get_line_number(25), Some(10));
}

#[test]
fn test_function_kind() {
    assert!(FunctionKind::Normal.has_this_binding());
    assert!(!FunctionKind::Arrow.has_this_binding());

    assert!(FunctionKind::Normal.is_constructor());
    assert!(FunctionKind::Constructor.is_constructor());
    assert!(!FunctionKind::Arrow.is_constructor());
    assert!(!FunctionKind::Method.is_constructor());
}

// ---------------------------------------------------------------------------
// Object type tests
// ---------------------------------------------------------------------------

#[test]
fn test_class_id_typed_array() {
    assert!(ClassId::Uint8Array.is_typed_array());
    assert!(ClassId::Float64Array.is_typed_array());
    assert!(!ClassId::Array.is_typed_array());
    assert!(!ClassId::Object.is_typed_array());
}

#[test]
fn test_class_id_error() {
    assert!(ClassId::Error.is_error());
    assert!(ClassId::TypeError.is_error());
    assert!(!ClassId::Object.is_error());
    assert!(!ClassId::Array.is_error());
}

#[test]
fn test_class_id_function() {
    assert!(ClassId::CFunction.is_function());
    assert!(ClassId::Closure.is_function());
    assert!(!ClassId::Object.is_function());
}

#[test]
fn test_property_type() {
    let mut prop = Property::new(Value::null(), Value::int(42));
    assert_eq!(prop.prop_type(), PropertyType::Normal);

    prop.set_prop_type(PropertyType::GetSet);
    assert_eq!(prop.prop_type(), PropertyType::GetSet);

    prop.set_hash_next(100);
    assert_eq!(prop.hash_next(), 100);
    assert_eq!(prop.prop_type(), PropertyType::GetSet);
}

#[test]
fn test_object_header() {
    let header = ObjectHeader::new(MemoryTag::Object);
    assert!(!header.is_marked());
    assert_eq!(header.mtag(), MemoryTag::Object);
}

// ---------------------------------------------------------------------------
// PropertyTable tests
// ---------------------------------------------------------------------------

#[test]
fn test_property_table_empty() {
    let table = PropertyTable::new();
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert!(table.get(Value::int(1)).is_none());
}

#[test]
fn test_property_table_set_get() {
    let mut table = PropertyTable::new();

    let key = Value::int(42);
    let value = Value::int(100);

    assert!(table.set(key, value));
    assert!(!table.is_empty());
    assert_eq!(table.len(), 1);

    let prop = table.get(key).unwrap();
    assert_eq!(prop.value, value);
}

#[test]
fn test_property_table_update() {
    let mut table = PropertyTable::new();

    let key = Value::int(1);
    table.set(key, Value::int(10));
    table.set(key, Value::int(20));

    assert_eq!(table.len(), 1);
    assert_eq!(table.get(key).unwrap().value, Value::int(20));
}

#[test]
fn test_property_table_delete() {
    let mut table = PropertyTable::new();

    let key = Value::int(1);
    table.set(key, Value::int(10));
    assert!(table.has(key));

    assert!(table.delete(key));
    assert!(!table.has(key));
    assert!(table.is_empty());

    assert!(!table.delete(key)); // Already deleted
}

#[test]
fn test_property_table_multiple() {
    let mut table = PropertyTable::new();

    for i in 0..100 {
        table.set(Value::int(i), Value::int(i * 2));
    }

    assert_eq!(table.len(), 100);

    for i in 0..100 {
        let prop = table.get(Value::int(i)).unwrap();
        assert_eq!(prop.value, Value::int(i * 2));
    }
}

#[test]
fn test_property_table_resize() {
    let mut table = PropertyTable::with_capacity(4);

    // Insert enough to trigger resize
    for i in 0..20 {
        table.set(Value::int(i), Value::int(i));
    }

    // All should still be findable
    for i in 0..20 {
        assert!(table.has(Value::int(i)));
    }
}

#[test]
fn test_property_table_delete_and_reuse() {
    let mut table = PropertyTable::new();

    table.set(Value::int(1), Value::int(10));
    table.set(Value::int(2), Value::int(20));
    table.set(Value::int(3), Value::int(30));

    table.delete(Value::int(2));
    assert_eq!(table.len(), 2);

    // New property should reuse deleted slot
    table.set(Value::int(4), Value::int(40));
    assert_eq!(table.len(), 3);

    assert!(table.has(Value::int(1)));
    assert!(!table.has(Value::int(2)));
    assert!(table.has(Value::int(3)));
    assert!(table.has(Value::int(4)));
}

#[test]
fn test_property_table_keys_iterator() {
    let mut table = PropertyTable::new();

    table.set(Value::int(1), Value::int(10));
    table.set(Value::int(2), Value::int(20));
    table.set(Value::int(3), Value::int(30));

    let keys: Vec<_> = table.keys().collect();
    assert_eq!(keys.len(), 3);
}
