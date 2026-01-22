/*
 * Test suite for FMI3 Binary value functions
 *
 * Tests fmi3GetBinary and fmi3SetBinary parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3SetBinary with simple data
 * ============================================================================ */
static void test_set_binary_simple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Byte data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    size_t sizes[] = {5};
    fmi3Binary values[] = {data};

    fmi3Status status = harness->fmi3SetBinary(instance, vr, 1, sizes, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_BINARY);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 0);
    assert_int_equal(rec->binary_sizes[0], 5);
    assert_memory_equal(rec->binary_data, data, 5);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBinary with embedded null bytes
 * ============================================================================ */
static void test_set_binary_with_nulls(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Byte data[] = {0x00, 0x01, 0x00, 0x02, 0x00};  /* Embedded nulls */
    size_t sizes[] = {5};
    fmi3Binary values[] = {data};

    fmi3Status status = harness->fmi3SetBinary(instance, vr, 1, sizes, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->binary_sizes[0], 5);
    assert_memory_equal(rec->binary_data, data, 5);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBinary with empty data
 * ============================================================================ */
static void test_set_binary_empty(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Byte data[] = {0};  /* Empty */
    size_t sizes[] = {0};
    fmi3Binary values[] = {data};

    fmi3Status status = harness->fmi3SetBinary(instance, vr, 1, sizes, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_BINARY);
    assert_int_equal(rec->binary_sizes[0], 0);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBinary with all byte values
 * ============================================================================ */
static void test_set_binary_all_bytes(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Byte data[256];
    for (int i = 0; i < 256; i++) {
        data[i] = (fmi3Byte)i;
    }
    size_t sizes[] = {256};
    fmi3Binary values[] = {data};

    fmi3Status status = harness->fmi3SetBinary(instance, vr, 1, sizes, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->binary_sizes[0], 256);
    assert_memory_equal(rec->binary_data, data, 256);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetBinary with single value
 * ============================================================================ */
static void test_get_binary_single(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    size_t sizes[1];
    fmi3Binary values[1];

    fmi3Status status = harness->fmi3GetBinary(instance, vr, 1, sizes, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_BINARY);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 0);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetBinary with multiple values
 * ============================================================================ */
static void test_get_binary_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    size_t sizes[3];
    fmi3Binary values[3];

    fmi3Status status = harness->fmi3GetBinary(instance, vr, 3, sizes, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_BINARY);
    assert_int_equal(rec->n_value_references, 3);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBinary with large data
 * ============================================================================ */
static void test_set_binary_large(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Byte data[1024];
    for (size_t i = 0; i < 1024; i++) {
        data[i] = (fmi3Byte)(i & 0xFF);
    }
    size_t sizes[] = {1024};
    fmi3Binary values[] = {data};

    fmi3Status status = harness->fmi3SetBinary(instance, vr, 1, sizes, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->binary_sizes[0], 1024);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_set_binary_simple),
        cmocka_unit_test(test_set_binary_with_nulls),
        cmocka_unit_test(test_set_binary_empty),
        cmocka_unit_test(test_set_binary_all_bytes),
        cmocka_unit_test(test_get_binary_single),
        cmocka_unit_test(test_get_binary_multiple),
        cmocka_unit_test(test_set_binary_large),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
