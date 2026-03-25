use super::FfiManager;
use std::time::Instant;
use typescript_types::{TsError, TsValue};

/// NAPI performance benchmark
pub fn run_napi_benchmark() {
    println!("Running NAPI performance benchmark...");

    // Create FFI manager
    let mut manager = FfiManager::new();

    // Register test functions
    let mut test_module = super::FfiModule::new("test");

    // Register a simple add function
    test_module.register_function("add", |args| {
        if args.len() != 2 {
            return Err(TsError::TypeError("Expected 2 arguments".to_string()));
        }
        match (&args[0], &args[1]) {
            (TsValue::Number(a), TsValue::Number(b)) => Ok(TsValue::Number(a + b)),
            _ => Err(TsError::TypeError("Expected numbers".to_string())),
        }
    });

    manager.register_module(test_module);

    // Benchmark the function call
    let iterations = 100000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = manager.call_function("test", "add", &[TsValue::Number(1.0), TsValue::Number(2.0)]);
    }

    let duration = start.elapsed();
    println!("NAPI benchmark: {} iterations in {:?} ({:?} per call)", iterations, duration, duration / iterations);
}

/// FFI performance benchmark
pub fn run_ffi_benchmark() {
    println!("Running FFI performance benchmark...");

    // Create FFI manager
    let mut manager = FfiManager::new();

    // Register test functions
    let mut test_module = super::FfiModule::new("test");

    // Register a simple add function
    test_module.register_function("add", |args| {
        if args.len() != 2 {
            return Err(TsError::TypeError("Expected 2 arguments".to_string()));
        }
        match (&args[0], &args[1]) {
            (TsValue::Number(a), TsValue::Number(b)) => Ok(TsValue::Number(a + b)),
            _ => Err(TsError::TypeError("Expected numbers".to_string())),
        }
    });

    manager.register_module(test_module);

    // Benchmark the function call
    let iterations = 100000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = manager.call_function("test", "add", &[TsValue::Number(1.0), TsValue::Number(2.0)]);
    }

    let duration = start.elapsed();
    println!("FFI benchmark: {} iterations in {:?} ({:?} per call)", iterations, duration, duration / iterations);
}
