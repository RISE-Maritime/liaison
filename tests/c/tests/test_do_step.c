/*
 * Test suite for FMI3 DoStep function
 *
 * Tests fmi3DoStep parameter transmission with various step sizes and flags.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3DoStep with basic parameters
 * ============================================================================ */
static void test_do_step_basic(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    /* Initialize the instance */
    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    fmi3Boolean eventHandlingNeeded;
    fmi3Boolean terminateSimulation;
    fmi3Boolean earlyReturn;
    fmi3Float64 lastSuccessfulTime;

    fmi3Status status = harness->fmi3DoStep(
        instance,
        0.0,        /* currentCommunicationPoint */
        0.001,      /* communicationStepSize */
        fmi3True,   /* noSetFMUStatePriorToCurrentPoint */
        &eventHandlingNeeded,
        &terminateSimulation,
        &earlyReturn,
        &lastSuccessfulTime
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_DO_STEP);
    assert_true(fabs(rec->current_communication_point - 0.0) < 1e-10);
    assert_true(fabs(rec->communication_step_size - 0.001) < 1e-10);
    assert_true(rec->no_set_fmu_state_prior_to_current_point == fmi3True);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3DoStep with large step size
 * ============================================================================ */
static void test_do_step_large_step(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    fmi3Boolean eventHandlingNeeded;
    fmi3Boolean terminateSimulation;
    fmi3Boolean earlyReturn;
    fmi3Float64 lastSuccessfulTime;

    fmi3Status status = harness->fmi3DoStep(
        instance,
        0.0,
        100.0,      /* large step size */
        fmi3False,
        &eventHandlingNeeded,
        &terminateSimulation,
        &earlyReturn,
        &lastSuccessfulTime
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->communication_step_size - 100.0) < 1e-10);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3DoStep with small step size
 * ============================================================================ */
static void test_do_step_small_step(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    fmi3Boolean eventHandlingNeeded;
    fmi3Boolean terminateSimulation;
    fmi3Boolean earlyReturn;
    fmi3Float64 lastSuccessfulTime;

    fmi3Status status = harness->fmi3DoStep(
        instance,
        0.0,
        1e-9,       /* very small step size */
        fmi3True,
        &eventHandlingNeeded,
        &terminateSimulation,
        &earlyReturn,
        &lastSuccessfulTime
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->communication_step_size - 1e-9) < 1e-15);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3DoStep with non-zero communication point
 * ============================================================================ */
static void test_do_step_nonzero_comm_point(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    fmi3Boolean eventHandlingNeeded;
    fmi3Boolean terminateSimulation;
    fmi3Boolean earlyReturn;
    fmi3Float64 lastSuccessfulTime;

    fmi3Status status = harness->fmi3DoStep(
        instance,
        5.5,        /* non-zero communication point */
        0.1,
        fmi3False,
        &eventHandlingNeeded,
        &terminateSimulation,
        &earlyReturn,
        &lastSuccessfulTime
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(fabs(rec->current_communication_point - 5.5) < 1e-10);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: Multiple DoStep calls
 * ============================================================================ */
static void test_do_step_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    fmi3Boolean eventHandlingNeeded;
    fmi3Boolean terminateSimulation;
    fmi3Boolean earlyReturn;
    fmi3Float64 lastSuccessfulTime;

    /* Perform multiple steps */
    fmi3Float64 t = 0.0;
    fmi3Float64 dt = 0.01;

    for (int i = 0; i < 5; i++) {
        fmi3Status status = harness->fmi3DoStep(
            instance, t, dt, fmi3True,
            &eventHandlingNeeded, &terminateSimulation,
            &earlyReturn, &lastSuccessfulTime
        );
        assert_int_equal(status, fmi3OK);
        t += dt;
    }

    harness_read_call_log(harness);

    /* Should have 5 DoStep calls */
    assert_int_equal(harness_get_call_count(harness), 5);

    /* Verify progression of communication points */
    for (int i = 0; i < 5; i++) {
        const CallRecord* rec = harness_get_call_record(harness, i);
        assert_non_null(rec);
        assert_int_equal(rec->type, CALL_FMI3_DO_STEP);
        assert_true(fabs(rec->current_communication_point - (i * dt)) < 1e-10);
    }

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: DoStep with noSetFMUStatePriorToCurrentPoint = false
 * ============================================================================ */
static void test_do_step_state_can_be_set(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness->fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    harness->fmi3ExitInitializationMode(instance);

    harness_clear_call_log(harness);

    fmi3Boolean eventHandlingNeeded;
    fmi3Boolean terminateSimulation;
    fmi3Boolean earlyReturn;
    fmi3Float64 lastSuccessfulTime;

    fmi3Status status = harness->fmi3DoStep(
        instance,
        0.0,
        0.001,
        fmi3False,  /* state CAN be set prior to this point */
        &eventHandlingNeeded,
        &terminateSimulation,
        &earlyReturn,
        &lastSuccessfulTime
    );

    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_true(rec->no_set_fmu_state_prior_to_current_point == fmi3False);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_do_step_basic),
        cmocka_unit_test(test_do_step_large_step),
        cmocka_unit_test(test_do_step_small_step),
        cmocka_unit_test(test_do_step_nonzero_comm_point),
        cmocka_unit_test(test_do_step_multiple),
        cmocka_unit_test(test_do_step_state_can_be_set),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
