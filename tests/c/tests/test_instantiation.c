/*
 * Test suite for FMI3 instantiation functions
 *
 * Tests that fmi3InstantiateCoSimulation, fmi3InstantiateModelExchange,
 * fmi3InstantiateScheduledExecution, and fmi3FreeInstance correctly
 * transmit all parameters through Zenoh to the server.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3InstantiateCoSimulation transmits all parameters
 * ============================================================================ */
static void test_instantiate_co_simulation(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    /* Clear call log and create fresh instance */
    harness_clear_call_log(harness);

    fmi3Instance instance = harness->fmi3InstantiateCoSimulation(
        "MyCoSimInstance",
        "{test-token-12345}",
        "/path/to/resources",
        fmi3True,   /* visible */
        fmi3True,   /* loggingOn */
        fmi3True,   /* eventModeUsed */
        fmi3True,   /* earlyReturnAllowed */
        NULL,       /* requiredIntermediateVariables */
        0,          /* nRequiredIntermediateVariables */
        NULL,       /* instanceEnvironment */
        harness_log_callback,
        NULL        /* intermediateUpdate */
    );

    assert_non_null(instance);

    /* Read call log from server */
    harness_read_call_log(harness);

    /* Verify call was recorded */
    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_INSTANTIATE_CO_SIMULATION);
    assert_string_equal(rec->instance_name, "MyCoSimInstance");
    assert_string_equal(rec->instantiation_token, "{test-token-12345}");
    assert_true(rec->visible == fmi3True);
    assert_true(rec->logging_on == fmi3True);
    assert_true(rec->event_mode_used == fmi3True);
    assert_true(rec->early_return_allowed == fmi3True);

    /* Cleanup */
    harness->fmi3FreeInstance(instance);
}

/* ============================================================================
 * Test: fmi3InstantiateModelExchange transmits all parameters
 * ============================================================================ */
static void test_instantiate_model_exchange(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    harness_clear_call_log(harness);

    fmi3Instance instance = harness->fmi3InstantiateModelExchange(
        "MyMEInstance",
        "{me-token-67890}",
        "/me/resources",
        fmi3False,  /* visible */
        fmi3True,   /* loggingOn */
        NULL,       /* instanceEnvironment */
        harness_log_callback
    );

    assert_non_null(instance);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE);
    assert_string_equal(rec->instance_name, "MyMEInstance");
    assert_string_equal(rec->instantiation_token, "{me-token-67890}");
    assert_true(rec->visible == fmi3False);
    assert_true(rec->logging_on == fmi3True);

    harness->fmi3FreeInstance(instance);
}

/* ============================================================================
 * Test: fmi3InstantiateScheduledExecution transmits all parameters
 * ============================================================================ */
static void test_instantiate_scheduled_execution(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    harness_clear_call_log(harness);

    fmi3Instance instance = harness->fmi3InstantiateScheduledExecution(
        "MySEInstance",
        "{se-token-11111}",
        "/se/resources",
        fmi3True,   /* visible */
        fmi3False,  /* loggingOn */
        NULL,       /* instanceEnvironment */
        harness_log_callback,
        NULL,       /* clockUpdate */
        NULL,       /* lockPreemption */
        NULL        /* unlockPreemption */
    );

    assert_non_null(instance);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION);
    assert_string_equal(rec->instance_name, "MySEInstance");
    assert_string_equal(rec->instantiation_token, "{se-token-11111}");
    assert_true(rec->visible == fmi3True);
    assert_true(rec->logging_on == fmi3False);

    harness->fmi3FreeInstance(instance);
}

/* ============================================================================
 * Test: fmi3FreeInstance is called correctly
 * ============================================================================ */
static void test_free_instance(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    /* Create instance first */
    fmi3Instance instance = harness->fmi3InstantiateCoSimulation(
        "InstanceToFree",
        "{free-token}",
        "/free/resources",
        fmi3False, fmi3False, fmi3False, fmi3False,
        NULL, 0, NULL, harness_log_callback, NULL
    );
    assert_non_null(instance);

    /* Clear log to capture only FreeInstance */
    harness_clear_call_log(harness);

    /* Free the instance */
    harness->fmi3FreeInstance(instance);

    harness_read_call_log(harness);

    assert_int_equal(harness_get_call_count(harness), 1);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_FREE_INSTANCE);
}

/* ============================================================================
 * Test: Multiple instances can be created
 * ============================================================================ */
static void test_multiple_instances(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    harness_clear_call_log(harness);

    fmi3Instance instance1 = harness->fmi3InstantiateCoSimulation(
        "Instance1", "{token1}", "/res1",
        fmi3False, fmi3False, fmi3False, fmi3False,
        NULL, 0, NULL, harness_log_callback, NULL
    );
    assert_non_null(instance1);

    fmi3Instance instance2 = harness->fmi3InstantiateCoSimulation(
        "Instance2", "{token2}", "/res2",
        fmi3True, fmi3True, fmi3True, fmi3True,
        NULL, 0, NULL, harness_log_callback, NULL
    );
    assert_non_null(instance2);

    harness_read_call_log(harness);

    /* Should have 2 instantiation calls */
    assert_int_equal(harness_get_call_count(harness), 2);

    const CallRecord* rec1 = harness_get_call_record(harness, 0);
    const CallRecord* rec2 = harness_get_call_record(harness, 1);

    assert_string_equal(rec1->instance_name, "Instance1");
    assert_string_equal(rec2->instance_name, "Instance2");

    /* Instance indices should be different */
    assert_int_not_equal(rec1->instance_index, rec2->instance_index);

    harness->fmi3FreeInstance(instance1);
    harness->fmi3FreeInstance(instance2);
}

/* ============================================================================
 * Test: Empty instance name is handled
 * ============================================================================ */
static void test_empty_instance_name(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    harness_clear_call_log(harness);

    fmi3Instance instance = harness->fmi3InstantiateCoSimulation(
        "",  /* empty name */
        "{empty-name-token}",
        "/resources",
        fmi3False, fmi3False, fmi3False, fmi3False,
        NULL, 0, NULL, harness_log_callback, NULL
    );
    assert_non_null(instance);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_string_equal(rec->instance_name, "");

    harness->fmi3FreeInstance(instance);
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_instantiate_co_simulation),
        cmocka_unit_test(test_instantiate_model_exchange),
        cmocka_unit_test(test_instantiate_scheduled_execution),
        cmocka_unit_test(test_free_instance),
        cmocka_unit_test(test_multiple_instances),
        cmocka_unit_test(test_empty_instance_name),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
