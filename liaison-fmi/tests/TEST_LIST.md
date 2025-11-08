# Complete Test List - Liaison FMI Client Library

## integration_tests.rs (41 tests)

### Conversion Functions (5 tests)
1. `test_proto_status_to_fmi_status_conversions` - All proto::Status → fmi3Status
2. `test_fmi_status_to_proto_status_conversions` - All fmi3Status → proto::Status
3. `test_i32_to_fmi_status_conversions` - i32 → fmi3Status (valid values)
4. `test_invalid_i32_to_fmi_status_defaults_to_error` - Invalid i32 → fmi3Error
5. `test_status_conversion_round_trip` - Bidirectional consistency

### FMI Function Exports - Core (3 tests)
6. `test_fmi3_get_version_export` - Version string is "3.0"
7. `test_fmi3_set_debug_logging_with_null_instance` - Returns fmi3Error
8. `test_fmi3_free_instance_with_null` - Safe null handling

### FMI Function Exports - Lifecycle (6 tests)
9. `test_fmi3_terminate_with_null_instance` - Returns fmi3Error
10. `test_fmi3_reset_with_null_instance` - Returns fmi3Error
11. `test_fmi3_enter_initialization_mode_with_null` - Returns fmi3Error
12. `test_fmi3_exit_initialization_mode_with_null` - Returns fmi3Error
13. `test_fmi3_enter_configuration_mode_with_null` - Returns fmi3Error
14. `test_fmi3_exit_configuration_mode_with_null` - Returns fmi3Error

### FMI Function Exports - Simulation (1 test)
15. `test_fmi3_do_step_with_null_instance` - Returns fmi3Error

### FMI Function Exports - Variable Access (4 tests)
16. `test_fmi3_get_boolean_with_null_instance` - Returns fmi3Error
17. `test_fmi3_set_boolean_with_null_instance` - Returns fmi3Error
18. `test_fmi3_get_string_with_null_instance` - Returns fmi3Error
19. `test_fmi3_set_string_with_null_instance` - Returns fmi3Error

### FMI Types and Constants (3 tests)
20. `test_fmi3_status_enum_values` - Verify numeric values 0-4
21. `test_fmi3_status_equality` - Test comparison operators
22. `test_fmi3_boolean_type` - Verify Boolean type (i32)

### Error Handling (2 tests)
23. `test_null_instance_pointer_safety` - All functions handle null safely
24. `test_null_pointer_safety_in_getters_setters` - Getters/setters null safety

### FMI Type Sizes (2 tests)
25. `test_fmi_numeric_type_sizes` - Float32/64, Int8-64, UInt8-64
26. `test_fmi_pointer_type_sizes` - All pointer types

### Proto Message Types (2 tests)
27. `test_proto_status_enum_values` - Verify values 0-4
28. `test_proto_status_from_i32` - TryFrom with error handling

### Callback Function Types (2 tests)
29. `test_callback_types_are_option` - All callbacks are Option
30. `test_log_callback_function_pointer` - Create and call callback

### Value Reference Type (2 tests)
31. `test_value_reference_type` - Type is u32
32. `test_value_reference_array` - Array handling

### FMI String Handling (2 tests)
33. `test_c_string_creation_and_conversion` - CString ↔ FMI string
34. `test_null_fmi_string` - Null string handling

### FMI Instantiation Functions (3 tests)
35. `test_fmi3_instantiate_co_simulation_with_null_name` - Returns null
36. `test_fmi3_instantiate_model_exchange_with_null_name` - Returns null
37. `test_fmi3_instantiate_scheduled_execution_with_null_name` - Returns null

### Module Organization (2 tests)
38. `test_module_exports` - Verify public exports
39. `test_api_completeness` - All FMI 3.0 categories present

### Thread Safety (1 test)
40. `test_send_trait_for_placeholder` - Compile-time Send verification

### Coverage Documentation (1 test)
41. `test_coverage_summary` - Documents what is and isn't tested

---

## conversion_tests.rs (8 tests)

### Exhaustive Conversions (2 tests)
1. `test_all_proto_to_fmi_conversions` - All proto::Status variants
2. `test_all_fmi_to_proto_conversions` - All fmi3Status variants

### i32 Conversions (2 tests)
3. `test_i32_to_fmi_status_valid_values` - Values 0-4
4. `test_i32_to_fmi_status_invalid_values` - Invalid values → Error

### Bidirectional Consistency (1 test)
5. `test_bidirectional_conversion_consistency` - Round-trip proto → fmi → proto

### Numeric Representation (2 tests)
6. `test_fmi_status_numeric_values` - Enum values 0-4
7. `test_proto_status_numeric_values` - Enum values 0-4

### Pattern Matching (1 test)
8. `test_conversion_pattern_matching` - Conversions in match expressions

---

## Test Execution

```bash
# Run all tests
cargo test -p liaison-fmi

# Run specific test file
cargo test -p liaison-fmi --test integration_tests
cargo test -p liaison-fmi --test conversion_tests

# Run specific test
cargo test -p liaison-fmi test_fmi3_get_version_export

# Run with output
cargo test -p liaison-fmi -- --nocapture
```

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total Test Files | 2 |
| Total Tests | 49 |
| Total Assertions | 99+ |
| Total Test Lines | 824 |
| Integration Tests | 41 |
| Conversion Tests | 8 |
| Test Categories | 8 |

---

## Test Coverage Map

```
liaison-fmi/
├── src/
│   ├── conversions.rs     → conversion_tests.rs ✓
│   ├── fmi3.rs           → integration_tests.rs ✓
│   ├── proto.rs          → integration_tests.rs ✓
│   ├── placeholder.rs    → (requires Zenoh server)
│   └── utils.rs          → (unit tests in module)
└── tests/
    ├── integration_tests.rs  (41 tests)
    ├── conversion_tests.rs   (8 tests)
    ├── README.md
    ├── TEST_SUMMARY.md
    └── TEST_LIST.md
```

---

## Quick Reference

### Most Important Tests

**Type Safety**:
- `test_all_proto_to_fmi_conversions`
- `test_all_fmi_to_proto_conversions`
- `test_fmi_numeric_type_sizes`

**Null Safety**:
- `test_null_instance_pointer_safety`
- `test_fmi3_terminate_with_null_instance`

**API Verification**:
- `test_fmi3_get_version_export`
- `test_api_completeness`

**Error Handling**:
- `test_invalid_i32_to_fmi_status_defaults_to_error`
- `test_null_pointer_safety_in_getters_setters`

### Adding New Tests

When adding a new FMI function:
1. Add null instance test in integration_tests.rs
2. Add type conversion test if new types introduced
3. Update test_api_completeness documentation
4. Update TEST_SUMMARY.md coverage table
