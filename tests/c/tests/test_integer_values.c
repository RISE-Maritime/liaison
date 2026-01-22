/*
 * Test suite for FMI3 integer value functions
 *
 * Tests all integer types: Int8, UInt8, Int16, UInt16, Int32, UInt32, Int64, UInt64
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <limits.h>
#include <stdint.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3SetInt32 with boundary values
 * ============================================================================ */
static void test_set_int32_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3Int32 values[] = {INT32_MIN, 0, INT32_MAX};

    fmi3Status status = harness->fmi3SetInt32(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_INT32);
    assert_int_equal(rec->n_values, 3);
    assert_int_equal(rec->values.int32_values[0], INT32_MIN);
    assert_int_equal(rec->values.int32_values[1], 0);
    assert_int_equal(rec->values.int32_values[2], INT32_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetInt32
 * ============================================================================ */
static void test_get_int32(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {3, 4};
    fmi3Int32 values[2];

    fmi3Status status = harness->fmi3GetInt32(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_INT32);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 3);
    assert_int_equal(rec->value_references[1], 4);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetUInt32 with boundary values
 * ============================================================================ */
static void test_set_uint32_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3UInt32 values[] = {0, UINT32_MAX};

    fmi3Status status = harness->fmi3SetUInt32(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_UINT32);
    assert_int_equal(rec->values.uint32_values[0], 0);
    assert_int_equal(rec->values.uint32_values[1], UINT32_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetInt8 with boundary values
 * ============================================================================ */
static void test_set_int8_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3Int8 values[] = {INT8_MIN, 0, INT8_MAX};

    fmi3Status status = harness->fmi3SetInt8(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_INT8);
    assert_int_equal(rec->values.int8_values[0], INT8_MIN);
    assert_int_equal(rec->values.int8_values[1], 0);
    assert_int_equal(rec->values.int8_values[2], INT8_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetUInt8 with boundary values
 * ============================================================================ */
static void test_set_uint8_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3UInt8 values[] = {0, UINT8_MAX};

    fmi3Status status = harness->fmi3SetUInt8(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_UINT8);
    assert_int_equal(rec->values.uint8_values[0], 0);
    assert_int_equal(rec->values.uint8_values[1], UINT8_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetInt16 with boundary values
 * ============================================================================ */
static void test_set_int16_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3Int16 values[] = {INT16_MIN, 0, INT16_MAX};

    fmi3Status status = harness->fmi3SetInt16(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_INT16);
    assert_int_equal(rec->values.int16_values[0], INT16_MIN);
    assert_int_equal(rec->values.int16_values[1], 0);
    assert_int_equal(rec->values.int16_values[2], INT16_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetUInt16 with boundary values
 * ============================================================================ */
static void test_set_uint16_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3UInt16 values[] = {0, UINT16_MAX};

    fmi3Status status = harness->fmi3SetUInt16(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_UINT16);
    assert_int_equal(rec->values.uint16_values[0], 0);
    assert_int_equal(rec->values.uint16_values[1], UINT16_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetInt64 with boundary values
 * ============================================================================ */
static void test_set_int64_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3Int64 values[] = {INT64_MIN, 0, INT64_MAX};

    fmi3Status status = harness->fmi3SetInt64(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_INT64);
    assert_true(rec->values.int64_values[0] == INT64_MIN);
    assert_true(rec->values.int64_values[1] == 0);
    assert_true(rec->values.int64_values[2] == INT64_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetUInt64 with boundary values
 * ============================================================================ */
static void test_set_uint64_boundary(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3UInt64 values[] = {0, UINT64_MAX};

    fmi3Status status = harness->fmi3SetUInt64(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_UINT64);
    assert_true(rec->values.uint64_values[0] == 0);
    assert_true(rec->values.uint64_values[1] == UINT64_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetFloat32 basic
 * ============================================================================ */
static void test_set_float32_basic(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3Float32 values[] = {1.5f, -2.5f};

    fmi3Status status = harness->fmi3SetFloat32(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_FLOAT32);
    assert_int_equal(rec->n_values, 2);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetFloat32
 * ============================================================================ */
static void test_get_float32(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5, 6};
    fmi3Float32 values[2];

    fmi3Status status = harness->fmi3GetFloat32(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_FLOAT32);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 5);
    assert_int_equal(rec->value_references[1], 6);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetInt8
 * ============================================================================ */
static void test_get_int8(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {10, 11, 12};
    fmi3Int8 values[3];

    fmi3Status status = harness->fmi3GetInt8(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_INT8);
    assert_int_equal(rec->n_value_references, 3);
    assert_int_equal(rec->value_references[0], 10);
    assert_int_equal(rec->value_references[1], 11);
    assert_int_equal(rec->value_references[2], 12);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetUInt8
 * ============================================================================ */
static void test_get_uint8(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {20, 21};
    fmi3UInt8 values[2];

    fmi3Status status = harness->fmi3GetUInt8(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_UINT8);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 20);
    assert_int_equal(rec->value_references[1], 21);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetInt16
 * ============================================================================ */
static void test_get_int16(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {30, 31};
    fmi3Int16 values[2];

    fmi3Status status = harness->fmi3GetInt16(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_INT16);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 30);
    assert_int_equal(rec->value_references[1], 31);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetUInt16
 * ============================================================================ */
static void test_get_uint16(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {40, 41};
    fmi3UInt16 values[2];

    fmi3Status status = harness->fmi3GetUInt16(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_UINT16);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 40);
    assert_int_equal(rec->value_references[1], 41);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetUInt32
 * ============================================================================ */
static void test_get_uint32(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {50, 51};
    fmi3UInt32 values[2];

    fmi3Status status = harness->fmi3GetUInt32(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_UINT32);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 50);
    assert_int_equal(rec->value_references[1], 51);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetInt64
 * ============================================================================ */
static void test_get_int64(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {60, 61};
    fmi3Int64 values[2];

    fmi3Status status = harness->fmi3GetInt64(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_INT64);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 60);
    assert_int_equal(rec->value_references[1], 61);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetUInt64
 * ============================================================================ */
static void test_get_uint64(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {70, 71};
    fmi3UInt64 values[2];

    fmi3Status status = harness->fmi3GetUInt64(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_UINT64);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->value_references[0], 70);
    assert_int_equal(rec->value_references[1], 71);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Multiple integer types in sequence
 * ============================================================================ */
static void test_multiple_integer_types(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    /* Set various integer types */
    fmi3ValueReference vr[] = {0};

    fmi3Int8 i8 = 42;
    fmi3UInt8 u8 = 200;
    fmi3Int16 i16 = -1000;
    fmi3UInt16 u16 = 50000;
    fmi3Int32 i32 = -100000;

    harness->fmi3SetInt8(instance, vr, 1, &i8, 1);
    harness->fmi3SetUInt8(instance, vr, 1, &u8, 1);
    harness->fmi3SetInt16(instance, vr, 1, &i16, 1);
    harness->fmi3SetUInt16(instance, vr, 1, &u16, 1);
    harness->fmi3SetInt32(instance, vr, 1, &i32, 1);

    harness_read_call_log(harness);

    /* Should have 5 calls */
    assert_int_equal(harness_get_call_count(harness), 5);

    assert_int_equal(harness_get_call_record(harness, 0)->type, CALL_FMI3_SET_INT8);
    assert_int_equal(harness_get_call_record(harness, 1)->type, CALL_FMI3_SET_UINT8);
    assert_int_equal(harness_get_call_record(harness, 2)->type, CALL_FMI3_SET_INT16);
    assert_int_equal(harness_get_call_record(harness, 3)->type, CALL_FMI3_SET_UINT16);
    assert_int_equal(harness_get_call_record(harness, 4)->type, CALL_FMI3_SET_INT32);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_set_int32_boundary),
        cmocka_unit_test(test_get_int32),
        cmocka_unit_test(test_set_uint32_boundary),
        cmocka_unit_test(test_set_int8_boundary),
        cmocka_unit_test(test_set_uint8_boundary),
        cmocka_unit_test(test_set_int16_boundary),
        cmocka_unit_test(test_set_uint16_boundary),
        cmocka_unit_test(test_set_int64_boundary),
        cmocka_unit_test(test_set_uint64_boundary),
        cmocka_unit_test(test_set_float32_basic),
        cmocka_unit_test(test_get_float32),
        cmocka_unit_test(test_get_int8),
        cmocka_unit_test(test_get_uint8),
        cmocka_unit_test(test_get_int16),
        cmocka_unit_test(test_get_uint16),
        cmocka_unit_test(test_get_uint32),
        cmocka_unit_test(test_get_int64),
        cmocka_unit_test(test_get_uint64),
        cmocka_unit_test(test_multiple_integer_types),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
