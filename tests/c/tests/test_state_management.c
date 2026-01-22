/*
 * Test suite for FMI3 state management functions
 *
 * Tests fmi3Reset, fmi3Terminate, and fmi3SetDebugLogging parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3Reset
 * ============================================================================ */
static void test_reset(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3Reset(instance);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_RESET);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3Terminate
 * ============================================================================ */
static void test_terminate(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3Terminate(instance);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_TERMINATE);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetDebugLogging with logging on
 * ============================================================================ */
static void test_set_debug_logging_on(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3String categories[] = {"logAll"};

    fmi3Status status = harness->fmi3SetDebugLogging(instance, fmi3True, 1, categories);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_DEBUG_LOGGING);
    assert_true(rec->logging_on == fmi3True);
    assert_int_equal(rec->n_categories, 1);
    assert_string_equal(rec->categories[0], "logAll");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetDebugLogging with logging off
 * ============================================================================ */
static void test_set_debug_logging_off(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3Status status = harness->fmi3SetDebugLogging(instance, fmi3False, 0, NULL);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(rec->logging_on == fmi3False);
    assert_int_equal(rec->n_categories, 0);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetDebugLogging with multiple categories
 * ============================================================================ */
static void test_set_debug_logging_multiple_categories(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3String categories[] = {"logEvents", "logSolver", "logFMI"};

    fmi3Status status = harness->fmi3SetDebugLogging(instance, fmi3True, 3, categories);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_categories, 3);
    assert_string_equal(rec->categories[0], "logEvents");
    assert_string_equal(rec->categories[1], "logSolver");
    assert_string_equal(rec->categories[2], "logFMI");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Full simulation lifecycle
 * ============================================================================ */
static void test_full_lifecycle(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    /* Create instance */
    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    /* Set debug logging */
    fmi3String categories[] = {"logAll"};
    harness->fmi3SetDebugLogging(instance, fmi3True, 1, categories);

    /* Enter initialization */
    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    /* Do a step */
    fmi3Boolean eventHandlingNeeded, terminateSimulation, earlyReturn;
    fmi3Float64 lastSuccessfulTime;
    harness->fmi3DoStep(instance, 0.0, 0.001, fmi3True,
                        &eventHandlingNeeded, &terminateSimulation,
                        &earlyReturn, &lastSuccessfulTime);

    /* Terminate */
    harness->fmi3Terminate(instance);

    harness_read_call_log(harness);

    /* Verify the sequence of calls */
    assert_int_equal(harness_get_call_count(harness), 5);

    assert_int_equal(harness_get_call_record(harness, 0)->type, CALL_FMI3_SET_DEBUG_LOGGING);
    assert_int_equal(harness_get_call_record(harness, 1)->type, CALL_FMI3_ENTER_INITIALIZATION_MODE);
    assert_int_equal(harness_get_call_record(harness, 2)->type, CALL_FMI3_EXIT_INITIALIZATION_MODE);
    assert_int_equal(harness_get_call_record(harness, 3)->type, CALL_FMI3_DO_STEP);
    assert_int_equal(harness_get_call_record(harness, 4)->type, CALL_FMI3_TERMINATE);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Reset and reinitialize
 * ============================================================================ */
static void test_reset_reinitialize(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    /* First initialization */
    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    /* Reset */
    fmi3Status status = harness->fmi3Reset(instance);
    assert_int_equal(status, fmi3OK);

    /* Reinitialize */
    harness->fmi3EnterInitializationMode(instance, fmi3True, 1e-5, 5.0, fmi3True, 20.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_read_call_log(harness);

    /* Should have reset + init sequence */
    assert_int_equal(harness_get_call_count(harness), 3);

    assert_int_equal(harness_get_call_record(harness, 0)->type, CALL_FMI3_RESET);
    assert_int_equal(harness_get_call_record(harness, 1)->type, CALL_FMI3_ENTER_INITIALIZATION_MODE);
    assert_int_equal(harness_get_call_record(harness, 2)->type, CALL_FMI3_EXIT_INITIALIZATION_MODE);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Multiple resets
 * ============================================================================ */
static void test_multiple_resets(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    /* Multiple resets */
    harness->fmi3Reset(instance);
    harness->fmi3Reset(instance);
    harness->fmi3Reset(instance);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 3);

    for (int i = 0; i < 3; i++) {
        assert_int_equal(harness_get_call_record(harness, i)->type, CALL_FMI3_RESET);
    }

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_reset),
        cmocka_unit_test(test_terminate),
        cmocka_unit_test(test_set_debug_logging_on),
        cmocka_unit_test(test_set_debug_logging_off),
        cmocka_unit_test(test_set_debug_logging_multiple_categories),
        cmocka_unit_test(test_full_lifecycle),
        cmocka_unit_test(test_reset_reinitialize),
        cmocka_unit_test(test_multiple_resets),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
