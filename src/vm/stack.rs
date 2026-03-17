//! Value stack for the VM
//!
//! The stack grows downward in memory (toward lower addresses).

use crate::value::Value;
use alloc::vec::Vec;

/// Value stack for bytecode execution
pub struct Stack {
    /// Stack storage
    values: Vec<Value>,
    /// Current frame pointer (index into values)
    frame_ptr: usize,
}

impl Stack {
    /// Create a new stack with the given capacity
    pub fn new(capacity: usize) -> Self {
        Stack {
            values: Vec::with_capacity(capacity),
            frame_ptr: 0,
        }
    }

    /// Push a value onto the stack
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    /// Pop a value from the stack
    #[inline]
    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    /// Clear the stack completely
    pub fn clear(&mut self) {
        self.values.clear();
        self.frame_ptr = 0;
    }

    /// Pop a value from the stack without checking
    ///
    /// # Safety
    /// Caller must ensure stack is not empty
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> Value {
        let len = self.values.len();
        debug_assert!(len > 0);
        unsafe {
            self.values.set_len(len - 1);
            *self.values.as_ptr().add(len - 1)
        }
    }

    /// Pop two values from the stack without checking
    ///
    /// # Safety
    /// Caller must ensure stack has at least 2 elements
    #[inline]
    pub unsafe fn pop2_unchecked(&mut self) -> (Value, Value) {
        let len = self.values.len();
        debug_assert!(len >= 2);
        unsafe {
            self.values.set_len(len - 2);
            let ptr = self.values.as_ptr();
            (*ptr.add(len - 1), *ptr.add(len - 2))
        }
    }

    /// Pop three values from the stack without checking
    ///
    /// # Safety
    /// Caller must ensure stack has at least 3 elements
    #[inline]
    pub unsafe fn pop3_unchecked(&mut self) -> (Value, Value, Value) {
        let len = self.values.len();
        debug_assert!(len >= 3);
        unsafe {
            self.values.set_len(len - 3);
            let ptr = self.values.as_ptr();
            (*ptr.add(len - 1), *ptr.add(len - 2), *ptr.add(len - 3))
        }
    }

    /// Peek at the top value without removing it
    #[inline]
    pub fn peek(&self) -> Option<Value> {
        self.values.last().copied()
    }

    /// Peek at a value at offset from top (0 = top)
    #[inline]
    pub fn peek_at(&self, offset: usize) -> Option<Value> {
        let len = self.values.len();
        if offset < len {
            Some(self.values[len - 1 - offset])
        } else {
            None
        }
    }

    /// Get the current stack depth
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the stack is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Drop n values from the stack
    #[inline]
    pub fn drop_n(&mut self, n: usize) {
        let new_len = self.values.len().saturating_sub(n);
        self.values.truncate(new_len);
    }

    /// Remove element at `offset` positions below the top of the stack.
    /// offset=0 removes the top, offset=1 removes one below top, etc.
    /// Returns the removed value, or None if out of range.
    #[inline]
    pub fn remove_at_offset(&mut self, offset: usize) -> Option<Value> {
        let len = self.values.len();
        if offset >= len {
            return None;
        }
        let idx = len - 1 - offset;
        Some(self.values.remove(idx))
    }

    /// Remove the function value immediately below the top `argc` arguments and
    /// compact the argument tail down by one slot.
    #[inline]
    pub fn compact_call_args(&mut self, argc: usize) -> Option<Value> {
        let len = self.values.len();
        if argc + 1 > len {
            return None;
        }

        let func_idx = len - argc - 1;
        let func = self.values[func_idx];
        self.values.copy_within(func_idx + 1..len, func_idx);
        self.values.truncate(len - 1);
        Some(func)
    }

    /// Remove the `[this, method]` pair immediately below the top `argc`
    /// arguments and compact the argument tail down by two slots.
    #[inline]
    pub fn compact_method_call_args(&mut self, argc: usize) -> Option<(Value, Value)> {
        let len = self.values.len();
        if argc + 2 > len {
            return None;
        }

        let this_idx = len - argc - 2;
        let method_idx = this_idx + 1;
        let this_val = self.values[this_idx];
        let method_val = self.values[method_idx];
        self.values.copy_within(method_idx + 1..len, this_idx);
        self.values.truncate(len - 2);
        Some((this_val, method_val))
    }

    /// Get raw value at absolute stack index
    #[inline]
    pub fn get_raw(&self, idx: usize) -> Value {
        self.values[idx]
    }

    /// Set raw value at absolute stack index
    #[inline]
    pub fn set_raw(&mut self, idx: usize, val: Value) {
        self.values[idx] = val;
    }

    /// Duplicate the top value
    #[inline]
    pub fn dup(&mut self) -> Option<()> {
        let val = self.peek()?;
        self.push(val);
        Some(())
    }

    /// Swap the top two values
    #[inline]
    pub fn swap(&mut self) -> Option<()> {
        let len = self.values.len();
        if len < 2 {
            return None;
        }
        self.values.swap(len - 1, len - 2);
        Some(())
    }

    /// Get value at index relative to frame pointer
    #[inline]
    pub fn get_local(&self, index: usize) -> Option<Value> {
        let abs_index = self.frame_ptr + index;
        self.values.get(abs_index).copied()
    }

    /// Set value at index relative to frame pointer
    #[inline]
    pub fn set_local(&mut self, index: usize, value: Value) -> Option<()> {
        let abs_index = self.frame_ptr + index;
        if abs_index < self.values.len() {
            self.values[abs_index] = value;
            Some(())
        } else {
            None
        }
    }

    /// Push a new frame
    pub fn push_frame(&mut self, locals: usize) {
        let new_frame_ptr = self.values.len();

        // Initialize locals to undefined
        for _ in 0..locals {
            self.values.push(Value::undefined());
        }

        self.frame_ptr = new_frame_ptr;
    }

    /// Pop a frame, returning to the previous frame pointer
    pub fn pop_frame(&mut self, prev_frame_ptr: usize, locals: usize) {
        self.drop_n(locals);
        self.frame_ptr = prev_frame_ptr;
    }

    /// Get value at absolute index relative to a given frame pointer
    #[inline]
    pub fn get_local_at(&self, frame_ptr: usize, index: usize) -> Option<Value> {
        let abs_index = frame_ptr + index;
        self.values.get(abs_index).copied()
    }

    /// Set value at absolute index relative to a given frame pointer
    #[inline]
    pub fn set_local_at(&mut self, frame_ptr: usize, index: usize, value: Value) {
        let abs_index = frame_ptr + index;
        if abs_index < self.values.len() {
            self.values[abs_index] = value;
        }
    }

    /// Get value at absolute local slot without bounds checks.
    ///
    /// # Safety
    /// Caller must guarantee that `frame_ptr + index < self.values.len()`.
    #[inline]
    pub unsafe fn get_local_at_unchecked(&self, frame_ptr: usize, index: usize) -> Value {
        let abs_index = frame_ptr + index;
        unsafe { *self.values.get_unchecked(abs_index) }
    }

    /// Set value at absolute local slot without bounds checks.
    ///
    /// # Safety
    /// Caller must guarantee that `frame_ptr + index < self.values.len()`.
    #[inline]
    pub unsafe fn set_local_at_unchecked(&mut self, frame_ptr: usize, index: usize, value: Value) {
        let abs_index = frame_ptr + index;
        unsafe {
            *self.values.get_unchecked_mut(abs_index) = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::new(16);

        stack.push(Value::int(1));
        stack.push(Value::int(2));
        stack.push(Value::int(3));

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop().unwrap().to_i32(), Some(3));
        assert_eq!(stack.pop().unwrap().to_i32(), Some(2));
        assert_eq!(stack.pop().unwrap().to_i32(), Some(1));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::new(16);

        stack.push(Value::int(1));
        stack.push(Value::int(2));

        assert_eq!(stack.peek().unwrap().to_i32(), Some(2));
        assert_eq!(stack.peek_at(0).unwrap().to_i32(), Some(2));
        assert_eq!(stack.peek_at(1).unwrap().to_i32(), Some(1));
        assert!(stack.peek_at(2).is_none());
    }

    #[test]
    fn test_dup() {
        let mut stack = Stack::new(16);

        stack.push(Value::int(42));
        stack.dup();

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop().unwrap().to_i32(), Some(42));
        assert_eq!(stack.pop().unwrap().to_i32(), Some(42));
    }

    #[test]
    fn test_swap() {
        let mut stack = Stack::new(16);

        stack.push(Value::int(1));
        stack.push(Value::int(2));
        stack.swap();

        assert_eq!(stack.pop().unwrap().to_i32(), Some(1));
        assert_eq!(stack.pop().unwrap().to_i32(), Some(2));
    }

    #[test]
    fn test_locals() {
        let mut stack = Stack::new(16);

        stack.push_frame(3);
        assert_eq!(stack.len(), 3);

        stack.set_local(0, Value::int(10));
        stack.set_local(1, Value::int(20));
        stack.set_local(2, Value::int(30));

        assert_eq!(stack.get_local(0).unwrap().to_i32(), Some(10));
        assert_eq!(stack.get_local(1).unwrap().to_i32(), Some(20));
        assert_eq!(stack.get_local(2).unwrap().to_i32(), Some(30));
    }

    #[test]
    fn test_compact_call_args() {
        let mut stack = Stack::new(16);
        stack.push(Value::int(99));
        stack.push(Value::int(1));
        stack.push(Value::int(2));

        let func = stack.compact_call_args(2).unwrap();
        assert_eq!(func.to_i32(), Some(99));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get_raw(0).to_i32(), Some(1));
        assert_eq!(stack.get_raw(1).to_i32(), Some(2));
    }

    #[test]
    fn test_compact_method_call_args() {
        let mut stack = Stack::new(16);
        stack.push(Value::int(7));
        stack.push(Value::int(8));
        stack.push(Value::int(1));
        stack.push(Value::int(2));

        let (this_val, method_val) = stack.compact_method_call_args(2).unwrap();
        assert_eq!(this_val.to_i32(), Some(7));
        assert_eq!(method_val.to_i32(), Some(8));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get_raw(0).to_i32(), Some(1));
        assert_eq!(stack.get_raw(1).to_i32(), Some(2));
    }
}
