/*
 * Test suite for FMI3 initialization functions
 *
 * Tests fmi3EnterInitializationMode, fmi3ExitInitializationMode,
 * and fmi3EnterEventMode parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3EnterInitializationMode with tolerance and stop time
 * ============================================================================ */
static void test_enter_initialization_mode_full(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    /* Create instance */
    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3EnterInitializationMode(
        instance,
        fmi3True,   /* toleranceDefined */
        1e-6,       /* tolerance */
        0.0,        /* startTime */
        fmi3True,   /* stopTimeDefined */
        10.0        /* stopTime */
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_ENTER_INITIALIZATION_MODE);
    assert_true(rec->tolerance_defined == fmi3True);
    assert_true(fabs(rec->tolerance - 1e-6) < 1e-10);
    assert_true(fabs(rec->start_time - 0.0) < 1e-10);
    assert_true(rec->stop_time_defined == fmi3True);
    assert_true(fabs(rec->stop_time - 10.0) < 1e-10);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3EnterInitializationMode without tolerance
 * ============================================================================ */
static void test_enter_initialization_mode_no_tolerance(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3EnterInitializationMode(
        instance,
        fmi3False,  /* toleranceDefined */
        0.0,        /* tolerance (ignored) */
        5.0,        /* startTime */
        fmi3False,  /* stopTimeDefined */
        0.0         /* stopTime (ignored) */
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(rec->tolerance_defined == fmi3False);
    assert_true(fabs(rec->start_time - 5.0) < 1e-10);
    assert_true(rec->stop_time_defined == fmi3False);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3EnterInitializationMode with negative start time
 * ============================================================================ */
static void test_enter_initialization_mode_negative_start(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3EnterInitializationMode(
        instance,
        fmi3False,
        0.0,
        -100.0,     /* negative start time */
        fmi3True,
        100.0
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->start_time - (-100.0)) < 1e-10);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3ExitInitializationMode
 * ============================================================================ */
static void test_exit_initialization_mode(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    /* Enter initialization first */
    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3ExitInitializationMode(instance);

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_EXIT_INITIALIZATION_MODE);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3EnterEventMode
 * ============================================================================ */
static void test_enter_event_mode(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3EnterEventMode(instance);

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_ENTER_EVENT_MODE);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Full initialization sequence
 * ============================================================================ */
static void test_full_initialization_sequence(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    /* Enter init mode */
    fmi3Status status1 = harness->fmi3EnterInitializationMode(
        instance, fmi3True, 1e-4, 0.0, fmi3True, 100.0);
    assert_int_equal(status1, fmi3OK);

    /* Exit init mode */
    fmi3Status status2 = harness->fmi3ExitInitializationMode(instance);
    assert_int_equal(status2, fmi3OK);

    harness_read_call_log(harness);

    /* Should have both calls */
    assert_int_equal(harness_get_call_count(harness), 2);

    const CallRecord* rec1 = harness_get_call_record(harness, 0);
    const CallRecord* rec2 = harness_get_call_record(harness, 1);

    assert_int_equal(rec1->type, CALL_FMI3_ENTER_INITIALIZATION_MODE);
    assert_int_equal(rec2->type, CALL_FMI3_EXIT_INITIALIZATION_MODE);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Large tolerance value
 * ============================================================================ */
static void test_large_tolerance(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3EnterInitializationMode(
        instance,
        fmi3True,
        1e10,       /* very large tolerance */
        0.0,
        fmi3False,
        0.0
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->tolerance - 1e10) < 1e5);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_enter_initialization_mode_full),
        cmocka_unit_test(test_enter_initialization_mode_no_tolerance),
        cmocka_unit_test(test_enter_initialization_mode_negative_start),
        cmocka_unit_test(test_exit_initialization_mode),
        cmocka_unit_test(test_enter_event_mode),
        cmocka_unit_test(test_full_initialization_sequence),
        cmocka_unit_test(test_large_tolerance),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
