//! Comprehensive tests for XDL standard library functions
//!
//! Tests for functions implemented in Phases 6-18

use xdl_core::XdlValue;

// Import the stdlib modules
use xdl_stdlib::math;
use xdl_stdlib::statistics;
use xdl_stdlib::signal;
use xdl_stdlib::complex;
use xdl_stdlib::system;
use xdl_stdlib::data_structures;
use xdl_stdlib::image_io;

// ============================================================================
// Phase 6: Mathematics Tests
// ============================================================================

#[test]
fn test_prime() {
    // Test prime number detection - just check it returns a result
    let result = math::prime(&[XdlValue::Long(7)]);
    assert!(result.is_ok());

    let result = math::prime(&[XdlValue::Long(10)]);
    assert!(result.is_ok());
}

#[test]
fn test_primes() {
    // Test prime number generation
    let result = math::primes(&[XdlValue::Long(20)]).unwrap();
    match result {
        XdlValue::Array(arr) => {
            assert!(arr.contains(&2.0));
            assert!(arr.contains(&3.0));
            assert!(arr.contains(&5.0));
            assert!(arr.contains(&7.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_gcd() {
    let result = math::gcd(&[XdlValue::Long(12), XdlValue::Long(18)]).unwrap();
    match result {
        XdlValue::Long(v) => assert_eq!(v, 6),
        _ => panic!("Expected Long"),
    }
}

#[test]
fn test_lcm() {
    let result = math::lcm(&[XdlValue::Long(4), XdlValue::Long(6)]).unwrap();
    match result {
        XdlValue::Long(v) => assert_eq!(v, 12),
        _ => panic!("Expected Long"),
    }
}

#[test]
fn test_binomial() {
    // C(5,2) = 10
    let result = math::binomial(&[XdlValue::Long(5), XdlValue::Long(2)]).unwrap();
    match result {
        XdlValue::Long(v) => assert_eq!(v, 10),
        XdlValue::Double(v) => assert!((v - 10.0).abs() < 0.001),
        XdlValue::Long64(v) => assert_eq!(v, 10),
        _ => panic!("Expected numeric"),
    }
}

#[test]
fn test_poly() {
    // Evaluate polynomial 2x^2 + 3x + 1 at x=2
    // = 2*4 + 3*2 + 1 = 8 + 6 + 1 = 15
    let coeffs = XdlValue::Array(vec![1.0, 3.0, 2.0]); // coeffs in ascending order
    let result = math::poly(&[XdlValue::Double(2.0), coeffs]).unwrap();
    match result {
        XdlValue::Double(v) => assert!((v - 15.0).abs() < 0.001),
        _ => panic!("Expected Double"),
    }
}

#[test]
fn test_finite() {
    let result = math::finite(&[XdlValue::Double(1.0)]).unwrap();
    // Just check it returns a truthy value for finite
    match result {
        XdlValue::Int(v) => assert!(v != 0),
        XdlValue::Long(v) => assert!(v != 0),
        _ => panic!("Expected Int or Long"),
    }

    let result = math::finite(&[XdlValue::Double(f64::INFINITY)]).unwrap();
    match result {
        XdlValue::Int(v) => assert_eq!(v, 0),
        XdlValue::Long(v) => assert_eq!(v, 0),
        _ => panic!("Expected Int or Long"),
    }
}

#[test]
fn test_product() {
    let arr = XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0]);
    let result = math::product(&[arr]).unwrap();
    match result {
        XdlValue::Double(v) => assert!((v - 24.0).abs() < 0.001),
        _ => panic!("Expected Double"),
    }
}

// ============================================================================
// Phase 7: Statistics Tests
// ============================================================================

#[test]
fn test_robust_mean() {
    let arr = XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0]); // outlier at end
    let result = statistics::robust_mean(&[arr]).unwrap();
    match result {
        XdlValue::Double(v) => {
            // Robust mean should be less affected by outlier
            assert!(v < 20.0);
        }
        _ => panic!("Expected Double"),
    }
}

#[test]
fn test_trimmed_mean() {
    let arr = XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    let result = statistics::trimmed_mean(&[arr, XdlValue::Double(0.1)]).unwrap();
    match result {
        XdlValue::Double(v) => assert!((v - 5.5).abs() < 1.0),
        _ => panic!("Expected Double"),
    }
}

// ============================================================================
// Phase 11: Signal Processing Tests
// ============================================================================

#[test]
fn test_hanning() {
    let result = signal::hanning(&[XdlValue::Long(5)]).unwrap();
    match result {
        XdlValue::Array(arr) => {
            assert_eq!(arr.len(), 5);
            // Hanning window starts and ends near 0
            assert!(arr[0] < 0.1);
            assert!(arr[4] < 0.1);
            // Middle should be near 1
            assert!(arr[2] > 0.9);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_hamming() {
    let result = signal::hamming(&[XdlValue::Long(5)]).unwrap();
    match result {
        XdlValue::Array(arr) => {
            assert_eq!(arr.len(), 5);
            // Hamming window doesn't go to zero at edges
            assert!(arr[0] > 0.05);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_blackman() {
    let result = signal::blackman(&[XdlValue::Long(5)]).unwrap();
    match result {
        XdlValue::Array(arr) => {
            assert_eq!(arr.len(), 5);
        }
        _ => panic!("Expected Array"),
    }
}

// ============================================================================
// Phase 14: Time & Date Tests
// ============================================================================

#[test]
fn test_weekday() {
    // Just check it returns a valid result
    let jd = system::julday(&[
        XdlValue::Long(1),
        XdlValue::Long(1),
        XdlValue::Long(2024),
    ]).unwrap();

    let result = system::weekday(&[jd]);
    assert!(result.is_ok());
}

#[test]
fn test_timestamp() {
    let result = system::timestamp(&[]);
    assert!(result.is_ok());
}

// ============================================================================
// Phase 15: Pointer Management Tests
// ============================================================================

#[test]
fn test_ptr_lifecycle() {
    // Create pointer
    let ptr = data_structures::ptr_new(&[XdlValue::Long(42)]).unwrap();
    assert!(matches!(ptr, XdlValue::Pointer(_)));

    // Check validity
    let valid = data_structures::ptr_valid(&[ptr.clone()]).unwrap();
    assert_eq!(valid, XdlValue::Int(1));

    // Dereference
    let value = data_structures::ptr_deref(&[ptr.clone()]).unwrap();
    assert_eq!(value, XdlValue::Long(42));

    // Free pointer
    data_structures::ptr_free(&[ptr.clone()]).unwrap();

    // Check no longer valid
    let valid_after = data_structures::ptr_valid(&[ptr]).unwrap();
    assert_eq!(valid_after, XdlValue::Int(0));
}

#[test]
fn test_null_pointer() {
    let ptr = data_structures::ptr_new(&[]).unwrap();
    assert!(matches!(ptr, XdlValue::Pointer(0)));

    let valid = data_structures::ptr_valid(&[ptr]).unwrap();
    assert_eq!(valid, XdlValue::Int(0));
}

// ============================================================================
// Phase 16: Data Structure Tests
// ============================================================================

#[test]
fn test_list_operations() {
    // Create list
    let list = data_structures::list(&[
        XdlValue::Long(1),
        XdlValue::Long(2),
        XdlValue::Long(3),
    ]).unwrap();

    // Check count
    let count = data_structures::list_count(&[list.clone()]).unwrap();
    assert_eq!(count, XdlValue::Long(3));

    // Add item
    let new_list = data_structures::list_add(&[list, XdlValue::Long(4)]).unwrap();
    let count2 = data_structures::list_count(&[new_list]).unwrap();
    assert_eq!(count2, XdlValue::Long(4));
}

#[test]
fn test_dictionary() {
    let dict = data_structures::dictionary(&[
        XdlValue::String("name".to_string()),
        XdlValue::String("Test".to_string()),
        XdlValue::String("value".to_string()),
        XdlValue::Long(42),
    ]).unwrap();

    match dict {
        XdlValue::Struct(map) => {
            assert!(map.contains_key("name"));
            assert!(map.contains_key("value"));
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_create_struct() {
    let s = data_structures::create_struct(&[
        XdlValue::String("x".to_string()),
        XdlValue::Double(1.5),
        XdlValue::String("y".to_string()),
        XdlValue::Double(2.5),
    ]).unwrap();

    match s {
        XdlValue::Struct(map) => {
            assert!(map.contains_key("x"));
            assert!(map.contains_key("y"));
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_object_lifecycle() {
    // Create object
    let obj = data_structures::obj_new(&[XdlValue::String("TestClass".to_string())]).unwrap();
    assert!(matches!(obj, XdlValue::Object(_)));

    // Check validity
    let valid = data_structures::obj_valid(&[obj.clone()]).unwrap();
    assert_eq!(valid, XdlValue::Int(1));

    // Get class
    let class = data_structures::obj_class(&[obj.clone()]).unwrap();
    assert_eq!(class, XdlValue::String("TestClass".to_string()));

    // Destroy
    data_structures::obj_destroy(&[obj.clone()]).unwrap();
    let valid_after = data_structures::obj_valid(&[obj]).unwrap();
    assert_eq!(valid_after, XdlValue::Int(0));
}

#[test]
fn test_heap_gc() {
    let result = data_structures::heap_gc(&[]);
    assert!(result.is_ok());
}

// ============================================================================
// Phase 17: Complex Numbers Tests
// ============================================================================

#[test]
fn test_complex_operations() {
    // Create complex number
    let c = complex::complex(&[XdlValue::Double(3.0), XdlValue::Double(4.0)]).unwrap();

    // Get real and imaginary parts
    let re = complex::real_part(&[c.clone()]).unwrap();
    let im = complex::imaginary_part(&[c.clone()]).unwrap();

    match re {
        XdlValue::Double(v) => assert!((v - 3.0).abs() < 0.001),
        _ => panic!("Expected Double"),
    }

    match im {
        XdlValue::Double(v) => assert!((v - 4.0).abs() < 0.001),
        _ => panic!("Expected Double"),
    }
}

#[test]
fn test_complex_arg() {
    // arg(1 + i) = pi/4
    let c = complex::complex(&[XdlValue::Double(1.0), XdlValue::Double(1.0)]).unwrap();
    let arg = complex::arg(&[c]).unwrap();

    match arg {
        XdlValue::Double(v) => assert!((v - std::f64::consts::FRAC_PI_4).abs() < 0.001),
        _ => panic!("Expected Double"),
    }
}

#[test]
fn test_polar() {
    // polar(1, 0) = 1 + 0i
    let c = complex::polar(&[XdlValue::Double(1.0), XdlValue::Double(0.0)]).unwrap();
    let re = complex::real_part(&[c]).unwrap();

    match re {
        XdlValue::Double(v) => assert!((v - 1.0).abs() < 0.001),
        _ => panic!("Expected Double"),
    }
}

#[test]
fn test_complexarr() {
    let result = complex::complexarr(&[XdlValue::Long(5)]).unwrap();
    match result {
        XdlValue::NestedArray(arr) => assert_eq!(arr.len(), 5),
        _ => panic!("Expected NestedArray"),
    }
}

// ============================================================================
// Phase 18: System Functions Tests
// ============================================================================

#[test]
fn test_routine_info() {
    let result = system::routine_info(&[XdlValue::String("PRINT".to_string())]);
    assert!(result.is_ok());
}

// ============================================================================
// Image I/O Tests
// ============================================================================

#[test]
fn test_tv_placeholder() {
    let img = XdlValue::NestedArray(vec![
        XdlValue::Array(vec![0.0, 128.0, 255.0]),
        XdlValue::Array(vec![64.0, 192.0, 32.0]),
    ]);
    let result = image_io::tv(&[img]).unwrap();
    assert_eq!(result, XdlValue::Int(1));
}

#[test]
fn test_tvscl_placeholder() {
    let img = XdlValue::NestedArray(vec![
        XdlValue::Array(vec![0.0, 128.0, 255.0]),
        XdlValue::Array(vec![64.0, 192.0, 32.0]),
    ]);
    let result = image_io::tvscl(&[img]).unwrap();
    assert_eq!(result, XdlValue::Int(1));
}
