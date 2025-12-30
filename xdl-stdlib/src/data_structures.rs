//! Data structure functions for XDL
//!
//! This module provides functions for creating and managing data structures:
//! - Pointers (PTR_NEW, PTR_VALID, PTR_FREE)
//! - Objects (OBJ_NEW, OBJ_VALID, OBJ_DESTROY)
//! - Lists (LIST)
//! - Hash tables (HASH, ORDEREDHASH, DICTIONARY)
//! - Structures (CREATE_STRUCT)

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use lazy_static::lazy_static;
use xdl_core::{XdlError, XdlResult, XdlValue};

// Global pointer heap - stores allocated values by ID
lazy_static! {
    static ref POINTER_HEAP: RwLock<HashMap<usize, XdlValue>> = RwLock::new(HashMap::new());
    static ref NEXT_PTR_ID: AtomicUsize = AtomicUsize::new(1);

    static ref OBJECT_HEAP: RwLock<HashMap<usize, XdlValue>> = RwLock::new(HashMap::new());
    static ref NEXT_OBJ_ID: AtomicUsize = AtomicUsize::new(1);
}

// ============================================================================
// Pointer Management Functions
// ============================================================================

/// PTR_NEW - Create a new heap pointer
/// Usage: ptr = PTR_NEW(value)
/// Returns a pointer to the heap-allocated value
pub fn ptr_new(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // Create null pointer
        return Ok(XdlValue::Pointer(0));
    }

    let value = args[0].clone();
    let id = NEXT_PTR_ID.fetch_add(1, Ordering::SeqCst);

    let mut heap = POINTER_HEAP.write()
        .map_err(|_| XdlError::RuntimeError("Failed to acquire pointer heap lock".to_string()))?;
    heap.insert(id, value);

    Ok(XdlValue::Pointer(id))
}

/// PTR_VALID - Check if a pointer is valid
/// Usage: result = PTR_VALID(ptr)
/// Returns 1 if valid, 0 if invalid or null
pub fn ptr_valid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Int(0));
    }

    match &args[0] {
        XdlValue::Pointer(id) => {
            if *id == 0 {
                return Ok(XdlValue::Int(0)); // Null pointer
            }
            let heap = POINTER_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire pointer heap lock".to_string()))?;
            Ok(XdlValue::Int(if heap.contains_key(id) { 1 } else { 0 }))
        }
        XdlValue::Array(arr) => {
            // Check multiple pointers
            let heap = POINTER_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire pointer heap lock".to_string()))?;
            let results: Vec<f64> = arr.iter().map(|id| {
                let ptr_id = *id as usize;
                if ptr_id == 0 || !heap.contains_key(&ptr_id) { 0.0 } else { 1.0 }
            }).collect();
            Ok(XdlValue::Array(results))
        }
        _ => Ok(XdlValue::Int(0)),
    }
}

/// PTR_FREE - Free a heap pointer
/// Usage: PTR_FREE, ptr
pub fn ptr_free(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Undefined);
    }

    match &args[0] {
        XdlValue::Pointer(id) => {
            if *id != 0 {
                let mut heap = POINTER_HEAP.write()
                    .map_err(|_| XdlError::RuntimeError("Failed to acquire pointer heap lock".to_string()))?;
                heap.remove(id);
            }
            Ok(XdlValue::Undefined)
        }
        XdlValue::Array(arr) => {
            // Free multiple pointers
            let mut heap = POINTER_HEAP.write()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire pointer heap lock".to_string()))?;
            for id in arr {
                let ptr_id = *id as usize;
                if ptr_id != 0 {
                    heap.remove(&ptr_id);
                }
            }
            Ok(XdlValue::Undefined)
        }
        _ => Ok(XdlValue::Undefined),
    }
}

/// Dereference a pointer - get the value it points to
pub fn ptr_deref(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("PTR_DEREF requires a pointer".to_string()));
    }

    match &args[0] {
        XdlValue::Pointer(id) => {
            if *id == 0 {
                return Err(XdlError::RuntimeError("Cannot dereference null pointer".to_string()));
            }
            let heap = POINTER_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire pointer heap lock".to_string()))?;
            heap.get(id)
                .cloned()
                .ok_or_else(|| XdlError::RuntimeError("Invalid pointer".to_string()))
        }
        _ => Err(XdlError::RuntimeError("PTR_DEREF requires a pointer".to_string())),
    }
}

// ============================================================================
// Object Management Functions
// ============================================================================

/// OBJ_NEW - Create a new object instance
/// Usage: obj = OBJ_NEW(class_name)
/// For now, creates a simple object with the class name stored
pub fn obj_new(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // Create null object
        return Ok(XdlValue::Object(0));
    }

    let class_name = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => "Object".to_string(),
    };

    let id = NEXT_OBJ_ID.fetch_add(1, Ordering::SeqCst);

    // Create a struct to represent the object
    let mut obj_data = HashMap::new();
    obj_data.insert("__class__".to_string(), XdlValue::String(class_name));

    // Add any initialization properties from remaining args
    if args.len() > 1 {
        for (i, arg) in args[1..].iter().enumerate() {
            obj_data.insert(format!("prop{}", i), arg.clone());
        }
    }

    let mut heap = OBJECT_HEAP.write()
        .map_err(|_| XdlError::RuntimeError("Failed to acquire object heap lock".to_string()))?;
    heap.insert(id, XdlValue::Struct(obj_data));

    Ok(XdlValue::Object(id))
}

/// OBJ_VALID - Check if an object reference is valid
/// Usage: result = OBJ_VALID(obj)
/// Returns 1 if valid, 0 if invalid or null
pub fn obj_valid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Int(0));
    }

    match &args[0] {
        XdlValue::Object(id) => {
            if *id == 0 {
                return Ok(XdlValue::Int(0)); // Null object
            }
            let heap = OBJECT_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire object heap lock".to_string()))?;
            Ok(XdlValue::Int(if heap.contains_key(id) { 1 } else { 0 }))
        }
        XdlValue::ObjRef(id) => {
            if *id == 0 {
                return Ok(XdlValue::Int(0));
            }
            let heap = OBJECT_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire object heap lock".to_string()))?;
            Ok(XdlValue::Int(if heap.contains_key(id) { 1 } else { 0 }))
        }
        _ => Ok(XdlValue::Int(0)),
    }
}

/// OBJ_DESTROY - Destroy an object instance
/// Usage: OBJ_DESTROY, obj
pub fn obj_destroy(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Undefined);
    }

    match &args[0] {
        XdlValue::Object(id) | XdlValue::ObjRef(id) => {
            if *id != 0 {
                let mut heap = OBJECT_HEAP.write()
                    .map_err(|_| XdlError::RuntimeError("Failed to acquire object heap lock".to_string()))?;
                heap.remove(id);
            }
            Ok(XdlValue::Undefined)
        }
        _ => Ok(XdlValue::Undefined),
    }
}

/// OBJ_CLASS - Get the class name of an object
pub fn obj_class(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("OBJ_CLASS requires an object".to_string()));
    }

    match &args[0] {
        XdlValue::Object(id) | XdlValue::ObjRef(id) => {
            if *id == 0 {
                return Ok(XdlValue::String("".to_string()));
            }
            let heap = OBJECT_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire object heap lock".to_string()))?;
            if let Some(XdlValue::Struct(data)) = heap.get(id) {
                if let Some(XdlValue::String(class_name)) = data.get("__class__") {
                    return Ok(XdlValue::String(class_name.clone()));
                }
            }
            Ok(XdlValue::String("Object".to_string()))
        }
        _ => Ok(XdlValue::String("".to_string())),
    }
}

/// OBJ_ISA - Check if object is instance of a class
pub fn obj_isa(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("OBJ_ISA requires object and class name".to_string()));
    }

    let class_name = match &args[1] {
        XdlValue::String(s) => s.to_uppercase(),
        _ => return Err(XdlError::RuntimeError("Class name must be a string".to_string())),
    };

    match &args[0] {
        XdlValue::Object(id) | XdlValue::ObjRef(id) => {
            if *id == 0 {
                return Ok(XdlValue::Int(0));
            }
            let heap = OBJECT_HEAP.read()
                .map_err(|_| XdlError::RuntimeError("Failed to acquire object heap lock".to_string()))?;
            if let Some(XdlValue::Struct(data)) = heap.get(id) {
                if let Some(XdlValue::String(obj_class)) = data.get("__class__") {
                    return Ok(XdlValue::Int(if obj_class.to_uppercase() == class_name { 1 } else { 0 }));
                }
            }
            Ok(XdlValue::Int(0))
        }
        _ => Ok(XdlValue::Int(0)),
    }
}

// ============================================================================
// Data Structure Functions
// ============================================================================

/// LIST - Create a new list
/// Usage: list = LIST(item1, item2, ...)
pub fn list(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Create a nested array to represent the list
    let items: Vec<XdlValue> = args.to_vec();
    Ok(XdlValue::NestedArray(items))
}

/// LIST_ADD - Add item to list
pub fn list_add(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("LIST_ADD requires list and item".to_string()));
    }

    match &args[0] {
        XdlValue::NestedArray(items) => {
            let mut new_items = items.clone();
            for arg in &args[1..] {
                new_items.push(arg.clone());
            }
            Ok(XdlValue::NestedArray(new_items))
        }
        _ => Err(XdlError::RuntimeError("First argument must be a list".to_string())),
    }
}

/// LIST_COUNT - Get number of items in list
pub fn list_count(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Long(0));
    }

    match &args[0] {
        XdlValue::NestedArray(items) => Ok(XdlValue::Long(items.len() as i32)),
        XdlValue::Array(items) => Ok(XdlValue::Long(items.len() as i32)),
        _ => Ok(XdlValue::Long(1)),
    }
}

/// HASH - Create a hash table
/// Usage: h = HASH(key1, value1, key2, value2, ...)
pub fn hash(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let mut map = HashMap::new();

    // Process pairs of key-value arguments
    let mut i = 0;
    while i + 1 < args.len() {
        let key = match &args[i] {
            XdlValue::String(s) => s.clone(),
            v => v.to_string_repr(),
        };
        let value = args[i + 1].clone();
        map.insert(key, value);
        i += 2;
    }

    Ok(XdlValue::Struct(map))
}

/// ORDEREDHASH - Create an ordered hash table
/// Same as HASH for now (Rust's HashMap doesn't preserve order, but this matches API)
pub fn orderedhash(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Use the same implementation as HASH
    // In a full implementation, would use IndexMap or similar
    hash(args)
}

/// DICTIONARY - Create a dictionary (alias for HASH)
pub fn dictionary(args: &[XdlValue]) -> XdlResult<XdlValue> {
    hash(args)
}

/// CREATE_STRUCT - Create a structure
/// Usage: s = CREATE_STRUCT(name, tag1, value1, tag2, value2, ...)
/// Or:    s = CREATE_STRUCT(tag1, value1, tag2, value2, ...)
pub fn create_struct(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // Empty anonymous structure
        return Ok(XdlValue::Struct(HashMap::new()));
    }

    let mut map = HashMap::new();
    let start_idx;

    // Check if first arg is structure name or first tag
    // If it's a string that looks like a name (no value following), treat as name
    if args.len() == 1 {
        // Single argument - structure name only
        return Ok(XdlValue::Struct(map));
    }

    // Check if first argument is a structure name
    let first_is_name = match &args[0] {
        XdlValue::String(s) => {
            // If second arg is also a string, first might be a name
            // Heuristic: if first string is all caps, it's a structure name
            s.chars().all(|c| c.is_uppercase() || c == '_')
        }
        _ => false,
    };

    if first_is_name && args.len() > 1 {
        // First arg is structure name, skip it
        if let XdlValue::String(name) = &args[0] {
            map.insert("__name__".to_string(), XdlValue::String(name.clone()));
        }
        start_idx = 1;
    } else {
        start_idx = 0;
    }

    // Process pairs of tag-value arguments
    let mut i = start_idx;
    while i + 1 < args.len() {
        let tag = match &args[i] {
            XdlValue::String(s) => s.clone(),
            v => v.to_string_repr(),
        };
        let value = args[i + 1].clone();
        map.insert(tag, value);
        i += 2;
    }

    Ok(XdlValue::Struct(map))
}

/// N_TAGS - Get number of tags in a structure
pub fn n_tags(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Long(0));
    }

    match &args[0] {
        XdlValue::Struct(map) => {
            // Don't count internal tags like __name__, __class__
            let count = map.keys()
                .filter(|k| !k.starts_with("__"))
                .count();
            Ok(XdlValue::Long(count as i32))
        }
        _ => Ok(XdlValue::Long(0)),
    }
}

/// TAG_NAMES - Get tag names from a structure
pub fn tag_names(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::NestedArray(vec![]));
    }

    match &args[0] {
        XdlValue::Struct(map) => {
            let names: Vec<XdlValue> = map.keys()
                .filter(|k| !k.starts_with("__"))
                .map(|k| XdlValue::String(k.to_uppercase()))
                .collect();
            Ok(XdlValue::NestedArray(names))
        }
        _ => Ok(XdlValue::NestedArray(vec![])),
    }
}

/// STRUCT_ASSIGN - Assign values to structure fields
pub fn struct_assign(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("STRUCT_ASSIGN requires structure and values".to_string()));
    }

    match &args[0] {
        XdlValue::Struct(map) => {
            let mut new_map = map.clone();

            // If second arg is also a struct, merge
            if let XdlValue::Struct(other) = &args[1] {
                for (k, v) in other {
                    new_map.insert(k.clone(), v.clone());
                }
            }

            Ok(XdlValue::Struct(new_map))
        }
        _ => Err(XdlError::RuntimeError("First argument must be a structure".to_string())),
    }
}

/// HEAP_GC - Perform garbage collection on heap
pub fn heap_gc(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // For now, just return count of items in heap
    let ptr_count = POINTER_HEAP.read()
        .map(|h| h.len())
        .unwrap_or(0);
    let obj_count = OBJECT_HEAP.read()
        .map(|h| h.len())
        .unwrap_or(0);

    Ok(XdlValue::Long((ptr_count + obj_count) as i32))
}

/// HEAP_FREE - Free all heap memory
pub fn heap_free(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if let Ok(mut heap) = POINTER_HEAP.write() {
        heap.clear();
    }
    if let Ok(mut heap) = OBJECT_HEAP.write() {
        heap.clear();
    }
    Ok(XdlValue::Undefined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ptr_new_and_valid() {
        let ptr = ptr_new(&[XdlValue::Long(42)]).unwrap();
        assert!(matches!(ptr, XdlValue::Pointer(_)));

        let valid = ptr_valid(&[ptr.clone()]).unwrap();
        assert_eq!(valid, XdlValue::Int(1));

        ptr_free(&[ptr.clone()]).unwrap();
        let valid_after = ptr_valid(&[ptr]).unwrap();
        assert_eq!(valid_after, XdlValue::Int(0));
    }

    #[test]
    fn test_list() {
        let lst = list(&[XdlValue::Long(1), XdlValue::Long(2), XdlValue::Long(3)]).unwrap();
        match lst {
            XdlValue::NestedArray(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected NestedArray"),
        }
    }

    #[test]
    fn test_hash() {
        let h = hash(&[
            XdlValue::String("key1".to_string()),
            XdlValue::Long(100),
            XdlValue::String("key2".to_string()),
            XdlValue::String("value2".to_string()),
        ]).unwrap();

        match h {
            XdlValue::Struct(map) => {
                assert_eq!(map.len(), 2);
                assert!(map.contains_key("key1"));
            }
            _ => panic!("Expected Struct"),
        }
    }

    #[test]
    fn test_create_struct() {
        let s = create_struct(&[
            XdlValue::String("x".to_string()),
            XdlValue::Double(1.0),
            XdlValue::String("y".to_string()),
            XdlValue::Double(2.0),
        ]).unwrap();

        match s {
            XdlValue::Struct(map) => {
                assert!(map.contains_key("x"));
                assert!(map.contains_key("y"));
            }
            _ => panic!("Expected Struct"),
        }
    }
}
