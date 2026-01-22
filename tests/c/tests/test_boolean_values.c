/*
 * Test suite for FMI3 Boolean value functions
 *
 * Tests fmi3GetBoolean and fmi3SetBoolean parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3SetBoolean with true and false
 * ============================================================================ */
static void test_set_boolean_basic(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1};
    fmi3Boolean values[] = {fmi3True, fmi3False};

    fmi3Status status = harness->fmi3SetBoolean(instance, vr, 2, values, 2);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_BOOLEAN);
    assert_int_equal(rec->n_value_references, 2);
    assert_int_equal(rec->n_values, 2);
    assert_true(rec->values.boolean_values[0] == fmi3True);
    assert_true(rec->values.boolean_values[1] == fmi3False);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBoolean with single true
 * ============================================================================ */
static void test_set_boolean_single_true(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {4};
    fmi3Boolean values[] = {fmi3True};

    fmi3Status status = harness->fmi3SetBoolean(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 4);
    assert_true(rec->values.boolean_values[0] == fmi3True);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBoolean with single false
 * ============================================================================ */
static void test_set_boolean_single_false(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3Boolean values[] = {fmi3False};

    fmi3Status status = harness->fmi3SetBoolean(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(rec->values.boolean_values[0] == fmi3False);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetBoolean with multiple values
 * ============================================================================ */
static void test_set_boolean_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2, 3, 4};
    fmi3Boolean values[] = {fmi3True, fmi3False, fmi3True, fmi3True, fmi3False};

    fmi3Status status = harness->fmi3SetBoolean(instance, vr, 5, values, 5);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_values, 5);
    assert_true(rec->values.boolean_values[0] == fmi3True);
    assert_true(rec->values.boolean_values[1] == fmi3False);
    assert_true(rec->values.boolean_values[2] == fmi3True);
    assert_true(rec->values.boolean_values[3] == fmi3True);
    assert_true(rec->values.boolean_values[4] == fmi3False);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetBoolean with single value
 * ============================================================================ */
static void test_get_boolean_single(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {4};
    fmi3Boolean values[1];

    fmi3Status status = harness->fmi3GetBoolean(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_BOOLEAN);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 4);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetBoolean with multiple values
 * ============================================================================ */
static void test_get_boolean_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {10, 20, 30};
    fmi3Boolean values[3];

    fmi3Status status = harness->fmi3GetBoolean(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_BOOLEAN);
    assert_int_equal(rec->n_value_references, 3);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Boolean set and get sequence
 * ============================================================================ */
static void test_boolean_set_get_sequence(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {4};
    fmi3Boolean set_value = fmi3True;
    fmi3Boolean get_value;

    harness->fmi3SetBoolean(instance, vr, 1, &set_value, 1);
    harness->fmi3GetBoolean(instance, vr, 1, &get_value, 1);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 2);

    const CallRecord* set_rec = harness_get_call_record(harness, 0);
    const CallRecord* get_rec = harness_get_call_record(harness, 1);

    assert_int_equal(set_rec->type, CALL_FMI3_SET_BOOLEAN);
    assert_int_equal(get_rec->type, CALL_FMI3_GET_BOOLEAN);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_set_boolean_basic),
        cmocka_unit_test(test_set_boolean_single_true),
        cmocka_unit_test(test_set_boolean_single_false),
        cmocka_unit_test(test_set_boolean_multiple),
        cmocka_unit_test(test_get_boolean_single),
        cmocka_unit_test(test_get_boolean_multiple),
        cmocka_unit_test(test_boolean_set_get_sequence),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
