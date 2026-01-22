/*
 * Test suite for FMI3 Clock value functions
 *
 * Tests fmi3GetClock and fmi3SetClock parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3SetClock with active clock
 * ============================================================================ */
static void test_set_clock_active(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Clock values[] = {fmi3ClockActive};

    fmi3Status status = harness->fmi3SetClock(instance, vr, 1, values);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_CLOCK);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 0);
    assert_true(rec->values.clock_values[0] == fmi3ClockActive);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetClock with inactive clock
 * ============================================================================ */
static void test_set_clock_inactive(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Clock values[] = {fmi3ClockInactive};

    fmi3Status status = harness->fmi3SetClock(instance, vr, 1, values);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(rec->values.clock_values[0] == fmi3ClockInactive);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetClock with multiple clocks
 * ============================================================================ */
static void test_set_clock_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2, 3};
    fmi3Clock values[] = {fmi3ClockActive, fmi3ClockInactive, fmi3ClockActive, fmi3ClockInactive};

    fmi3Status status = harness->fmi3SetClock(instance, vr, 4, values);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_value_references, 4);
    assert_int_equal(rec->n_values, 4);
    assert_true(rec->values.clock_values[0] == fmi3ClockActive);
    assert_true(rec->values.clock_values[1] == fmi3ClockInactive);
    assert_true(rec->values.clock_values[2] == fmi3ClockActive);
    assert_true(rec->values.clock_values[3] == fmi3ClockInactive);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetClock with single clock
 * ============================================================================ */
static void test_get_clock_single(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Clock values[1];

    fmi3Status status = harness->fmi3GetClock(instance, vr, 1, values);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_CLOCK);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 0);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetClock with multiple clocks
 * ============================================================================ */
static void test_get_clock_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {10, 20, 30};
    fmi3Clock values[3];

    fmi3Status status = harness->fmi3GetClock(instance, vr, 3, values);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_CLOCK);
    assert_int_equal(rec->n_value_references, 3);
    assert_int_equal(rec->value_references[0], 10);
    assert_int_equal(rec->value_references[1], 20);
    assert_int_equal(rec->value_references[2], 30);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Clock set and get sequence
 * ============================================================================ */
static void test_clock_set_get_sequence(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0};
    fmi3Clock set_value = fmi3ClockActive;
    fmi3Clock get_value;

    harness->fmi3SetClock(instance, vr, 1, &set_value);
    harness->fmi3GetClock(instance, vr, 1, &get_value);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 2);

    const CallRecord* set_rec = harness_get_call_record(harness, 0);
    const CallRecord* get_rec = harness_get_call_record(harness, 1);

    assert_int_equal(set_rec->type, CALL_FMI3_SET_CLOCK);
    assert_int_equal(get_rec->type, CALL_FMI3_GET_CLOCK);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: All clocks active
 * ============================================================================ */
static void test_all_clocks_active(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3Clock values[] = {fmi3ClockActive, fmi3ClockActive, fmi3ClockActive};

    fmi3Status status = harness->fmi3SetClock(instance, vr, 3, values);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    for (size_t i = 0; i < 3; i++) {
        assert_true(rec->values.clock_values[i] == fmi3ClockActive);
    }

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_set_clock_active),
        cmocka_unit_test(test_set_clock_inactive),
        cmocka_unit_test(test_set_clock_multiple),
        cmocka_unit_test(test_get_clock_single),
        cmocka_unit_test(test_get_clock_multiple),
        cmocka_unit_test(test_clock_set_get_sequence),
        cmocka_unit_test(test_all_clocks_active),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
