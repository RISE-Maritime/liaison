/*
 * Test suite for FMI3 Float64 value functions
 *
 * Tests fmi3GetFloat64 and fmi3SetFloat64 parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>
#include <float.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3SetFloat64 with single value
 * ============================================================================ */
static void test_set_float64_single(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Float64 values[] = {3.14159265358979};

    fmi3Status status = harness->fmi3SetFloat64(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_FLOAT64);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 0);
    assert_int_equal(rec->n_values, 1);
    assert_true(fabs(rec->values.float64_values[0] - 3.14159265358979) < 1e-10);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetFloat64 with multiple values
 * ============================================================================ */
static void test_set_float64_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2, 3};
    fmi3Float64 values[] = {1.0, 2.0, 3.0, 4.0};

    fmi3Status status = harness->fmi3SetFloat64(instance, vr, 4, values, 4);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_value_references, 4);
    assert_int_equal(rec->n_values, 4);

    for (size_t i = 0; i < 4; i++) {
        assert_int_equal(rec->value_references[i], i);
        assert_true(fabs(rec->values.float64_values[i] - (double)(i + 1)) < 1e-10);
    }

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetFloat64 with special values (NaN, Inf)
 * ============================================================================ */
static void test_set_float64_special_values(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2, 3};
    fmi3Float64 values[] = {
        INFINITY,
        -INFINITY,
        NAN,
        DBL_MAX
    };

    fmi3Status status = harness->fmi3SetFloat64(instance, vr, 4, values, 4);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_values, 4);

    assert_true(isinf(rec->values.float64_values[0]) && rec->values.float64_values[0] > 0);
    assert_true(isinf(rec->values.float64_values[1]) && rec->values.float64_values[1] < 0);
    assert_true(isnan(rec->values.float64_values[2]));
    assert_true(fabs(rec->values.float64_values[3] - DBL_MAX) < 1e-10 * DBL_MAX);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetFloat64 with very small values
 * ============================================================================ */
static void test_set_float64_small_values(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3Float64 values[] = {DBL_MIN, DBL_EPSILON};

    fmi3Status status = harness->fmi3SetFloat64(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->values.float64_values[0] - DBL_MIN) < DBL_MIN * 1e-10);
    assert_true(fabs(rec->values.float64_values[1] - DBL_EPSILON) < DBL_EPSILON * 1e-10);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetFloat64 with negative values
 * ============================================================================ */
static void test_set_float64_negative(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3Float64 values[] = {-1.0, -100.5, -0.0};

    fmi3Status status = harness->fmi3SetFloat64(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->values.float64_values[0] - (-1.0)) < 1e-10);
    assert_true(fabs(rec->values.float64_values[1] - (-100.5)) < 1e-10);
    /* -0.0 should equal 0.0 */
    assert_true(rec->values.float64_values[2] == 0.0);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetFloat64 with single value
 * ============================================================================ */
static void test_get_float64_single(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Float64 values[1];

    fmi3Status status = harness->fmi3GetFloat64(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_FLOAT64);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 0);
    assert_int_equal(rec->n_values, 1);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetFloat64 with multiple values
 * ============================================================================ */
static void test_get_float64_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {10, 20, 30};
    fmi3Float64 values[3];

    fmi3Status status = harness->fmi3GetFloat64(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_FLOAT64);
    assert_int_equal(rec->n_value_references, 3);
    assert_int_equal(rec->value_references[0], 10);
    assert_int_equal(rec->value_references[1], 20);
    assert_int_equal(rec->value_references[2], 30);
    assert_int_equal(rec->n_values, 3);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetFloat64 with large value reference
 * ============================================================================ */
static void test_set_float64_large_vr(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0xFFFFFFFF};  /* max uint32 */
    fmi3Float64 values[] = {42.0};

    fmi3Status status = harness->fmi3SetFloat64(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->value_references[0], 0xFFFFFFFF);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_set_float64_single),
        cmocka_unit_test(test_set_float64_multiple),
        cmocka_unit_test(test_set_float64_special_values),
        cmocka_unit_test(test_set_float64_small_values),
        cmocka_unit_test(test_set_float64_negative),
        cmocka_unit_test(test_get_float64_single),
        cmocka_unit_test(test_get_float64_multiple),
        cmocka_unit_test(test_set_float64_large_vr),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
