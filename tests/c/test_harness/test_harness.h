#ifndef TEST_HARNESS_H
#define TEST_HARNESS_H

#include "fmi3PlatformTypes.h"
#include "fmi3FunctionTypes.h"
#include "call_record.h"
#include <stdbool.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Test Harness Configuration
 * ============================================================================ */

#define HARNESS_MAX_PATH 1024
#define HARNESS_SERVER_STARTUP_WAIT_MS 2000
#define HARNESS_RESPONDER_ID "test-harness"

/* ============================================================================
 * Test Harness Structure
 * ============================================================================ */

typedef struct {
    /* Server process management */
    pid_t server_pid;
    bool server_running;

    /* LiaisonFMU library handle and function pointers */
    void* liaison_fmu_lib;

    /* Paths */
    char liaison_binary_path[HARNESS_MAX_PATH];
    char liaison_fmu_lib_path[HARNESS_MAX_PATH];
    char dummy_fmu_lib_path[HARNESS_MAX_PATH];
    char dummy_fmu_dir[HARNESS_MAX_PATH];
    char call_log_path[HARNESS_MAX_PATH];
    char config_json_path[HARNESS_MAX_PATH];

    /* FMI3 function pointers (from LiaisonFMU) */
    fmi3GetVersionTYPE* fmi3GetVersion;
    fmi3SetDebugLoggingTYPE* fmi3SetDebugLogging;
    fmi3InstantiateModelExchangeTYPE* fmi3InstantiateModelExchange;
    fmi3InstantiateCoSimulationTYPE* fmi3InstantiateCoSimulation;
    fmi3InstantiateScheduledExecutionTYPE* fmi3InstantiateScheduledExecution;
    fmi3FreeInstanceTYPE* fmi3FreeInstance;
    fmi3EnterInitializationModeTYPE* fmi3EnterInitializationMode;
    fmi3ExitInitializationModeTYPE* fmi3ExitInitializationMode;
    fmi3EnterEventModeTYPE* fmi3EnterEventMode;
    fmi3TerminateTYPE* fmi3Terminate;
    fmi3ResetTYPE* fmi3Reset;
    fmi3GetFloat32TYPE* fmi3GetFloat32;
    fmi3GetFloat64TYPE* fmi3GetFloat64;
    fmi3GetInt8TYPE* fmi3GetInt8;
    fmi3GetUInt8TYPE* fmi3GetUInt8;
    fmi3GetInt16TYPE* fmi3GetInt16;
    fmi3GetUInt16TYPE* fmi3GetUInt16;
    fmi3GetInt32TYPE* fmi3GetInt32;
    fmi3GetUInt32TYPE* fmi3GetUInt32;
    fmi3GetInt64TYPE* fmi3GetInt64;
    fmi3GetUInt64TYPE* fmi3GetUInt64;
    fmi3GetBooleanTYPE* fmi3GetBoolean;
    fmi3GetStringTYPE* fmi3GetString;
    fmi3GetBinaryTYPE* fmi3GetBinary;
    fmi3GetClockTYPE* fmi3GetClock;
    fmi3SetFloat32TYPE* fmi3SetFloat32;
    fmi3SetFloat64TYPE* fmi3SetFloat64;
    fmi3SetInt8TYPE* fmi3SetInt8;
    fmi3SetUInt8TYPE* fmi3SetUInt8;
    fmi3SetInt16TYPE* fmi3SetInt16;
    fmi3SetUInt16TYPE* fmi3SetUInt16;
    fmi3SetInt32TYPE* fmi3SetInt32;
    fmi3SetUInt32TYPE* fmi3SetUInt32;
    fmi3SetInt64TYPE* fmi3SetInt64;
    fmi3SetUInt64TYPE* fmi3SetUInt64;
    fmi3SetBooleanTYPE* fmi3SetBoolean;
    fmi3SetStringTYPE* fmi3SetString;
    fmi3SetBinaryTYPE* fmi3SetBinary;
    fmi3SetClockTYPE* fmi3SetClock;
    fmi3DoStepTYPE* fmi3DoStep;

    /* Current FMI instance */
    fmi3Instance current_instance;

} TestHarness;

/* ============================================================================
 * Harness Lifecycle Functions
 * ============================================================================ */

/**
 * Initialize the test harness
 * Reads paths from environment variables set by CMake
 * Returns 0 on success, -1 on failure
 */
int harness_init(TestHarness* harness);

/**
 * Start the liaison server with the dummy FMU
 * Returns 0 on success, -1 on failure
 */
int harness_start_server(TestHarness* harness);

/**
 * Stop the liaison server
 * Returns 0 on success, -1 on failure
 */
int harness_stop_server(TestHarness* harness);

/**
 * Load the LiaisonFMU library and bind all function pointers
 * Returns 0 on success, -1 on failure
 */
int harness_load_liaison_fmu(TestHarness* harness);

/**
 * Cleanup all resources
 */
void harness_cleanup(TestHarness* harness);

/* ============================================================================
 * Call Log Access Functions
 * ============================================================================ */

/**
 * Clear the call log on the server side
 * (Writes empty JSON to the call log file)
 */
int harness_clear_call_log(TestHarness* harness);

/**
 * Read the call log from the server
 * Returns the number of calls, or -1 on error
 */
int harness_read_call_log(TestHarness* harness);

/**
 * Get the number of recorded calls
 */
size_t harness_get_call_count(TestHarness* harness);

/**
 * Get a call record by index
 * Returns NULL if index is out of bounds
 */
const CallRecord* harness_get_call_record(TestHarness* harness, size_t index);

/* ============================================================================
 * Convenience Functions for Tests
 * ============================================================================ */

/**
 * Create a default FMI instance for testing
 * Uses Co-Simulation mode with standard parameters
 */
fmi3Instance harness_create_instance(TestHarness* harness);

/**
 * Free the current test instance
 */
void harness_free_instance(TestHarness* harness);

/**
 * Log callback for tests (prints to stderr)
 */
void harness_log_callback(fmi3InstanceEnvironment instanceEnvironment,
                          fmi3Status status,
                          fmi3String category,
                          fmi3String message);

/* ============================================================================
 * CMocka Integration Helpers
 * ============================================================================ */

/**
 * Setup function for test groups
 * Initializes harness, starts server, loads library
 */
int harness_group_setup(void** state);

/**
 * Teardown function for test groups
 * Stops server, cleans up harness
 */
int harness_group_teardown(void** state);

/**
 * Setup function for individual tests
 * Clears call log, creates fresh instance
 */
int harness_test_setup(void** state);

/**
 * Teardown function for individual tests
 * Frees instance
 */
int harness_test_teardown(void** state);

#ifdef __cplusplus
}
#endif

#endif /* TEST_HARNESS_H */
