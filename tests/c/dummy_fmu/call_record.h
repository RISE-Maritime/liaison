#ifndef CALL_RECORD_H
#define CALL_RECORD_H

#include "fmi3PlatformTypes.h"
#include "fmi3FunctionTypes.h"
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Maximum sizes for recorded data */
#define MAX_CALL_RECORDS 1000
#define MAX_VALUE_REFERENCES 100
#define MAX_VALUES_SIZE 4096
#define MAX_STRING_LENGTH 1024
#define MAX_STRINGS 100
#define MAX_BINARY_SIZE 4096
#define MAX_CATEGORIES 32

/* Call type enumeration for each FMI3 function */
typedef enum {
    CALL_NONE = 0,
    /* Instantiation */
    CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE,
    CALL_FMI3_INSTANTIATE_CO_SIMULATION,
    CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION,
    CALL_FMI3_FREE_INSTANCE,
    /* Initialization */
    CALL_FMI3_ENTER_INITIALIZATION_MODE,
    CALL_FMI3_EXIT_INITIALIZATION_MODE,
    CALL_FMI3_ENTER_EVENT_MODE,
    CALL_FMI3_TERMINATE,
    CALL_FMI3_RESET,
    /* Getters */
    CALL_FMI3_GET_FLOAT32,
    CALL_FMI3_GET_FLOAT64,
    CALL_FMI3_GET_INT8,
    CALL_FMI3_GET_UINT8,
    CALL_FMI3_GET_INT16,
    CALL_FMI3_GET_UINT16,
    CALL_FMI3_GET_INT32,
    CALL_FMI3_GET_UINT32,
    CALL_FMI3_GET_INT64,
    CALL_FMI3_GET_UINT64,
    CALL_FMI3_GET_BOOLEAN,
    CALL_FMI3_GET_STRING,
    CALL_FMI3_GET_BINARY,
    CALL_FMI3_GET_CLOCK,
    /* Setters */
    CALL_FMI3_SET_FLOAT32,
    CALL_FMI3_SET_FLOAT64,
    CALL_FMI3_SET_INT8,
    CALL_FMI3_SET_UINT8,
    CALL_FMI3_SET_INT16,
    CALL_FMI3_SET_UINT16,
    CALL_FMI3_SET_INT32,
    CALL_FMI3_SET_UINT32,
    CALL_FMI3_SET_INT64,
    CALL_FMI3_SET_UINT64,
    CALL_FMI3_SET_BOOLEAN,
    CALL_FMI3_SET_STRING,
    CALL_FMI3_SET_BINARY,
    CALL_FMI3_SET_CLOCK,
    /* Simulation */
    CALL_FMI3_DO_STEP,
    /* Debug */
    CALL_FMI3_SET_DEBUG_LOGGING,
    CALL_FMI3_GET_VERSION,
    /* Count */
    CALL_TYPE_COUNT
} CallType;

/* Structure to record parameters of a single FMI3 function call */
typedef struct {
    CallType type;
    int instance_index;

    /* For instantiation functions */
    char instance_name[MAX_STRING_LENGTH];
    char instantiation_token[MAX_STRING_LENGTH];
    char resource_path[MAX_STRING_LENGTH];
    fmi3Boolean visible;
    fmi3Boolean logging_on;
    fmi3Boolean event_mode_used;
    fmi3Boolean early_return_allowed;
    size_t n_required_intermediate_variables;
    fmi3ValueReference required_intermediate_variables[MAX_VALUE_REFERENCES];

    /* For Get/Set value functions */
    fmi3ValueReference value_references[MAX_VALUE_REFERENCES];
    size_t n_value_references;
    size_t n_values;

    /* Storage for different value types */
    union {
        fmi3Float32 float32_values[MAX_VALUES_SIZE / sizeof(fmi3Float32)];
        fmi3Float64 float64_values[MAX_VALUES_SIZE / sizeof(fmi3Float64)];
        fmi3Int8 int8_values[MAX_VALUES_SIZE / sizeof(fmi3Int8)];
        fmi3UInt8 uint8_values[MAX_VALUES_SIZE / sizeof(fmi3UInt8)];
        fmi3Int16 int16_values[MAX_VALUES_SIZE / sizeof(fmi3Int16)];
        fmi3UInt16 uint16_values[MAX_VALUES_SIZE / sizeof(fmi3UInt16)];
        fmi3Int32 int32_values[MAX_VALUES_SIZE / sizeof(fmi3Int32)];
        fmi3UInt32 uint32_values[MAX_VALUES_SIZE / sizeof(fmi3UInt32)];
        fmi3Int64 int64_values[MAX_VALUES_SIZE / sizeof(fmi3Int64)];
        fmi3UInt64 uint64_values[MAX_VALUES_SIZE / sizeof(fmi3UInt64)];
        fmi3Boolean boolean_values[MAX_VALUES_SIZE / sizeof(fmi3Boolean)];
        fmi3Clock clock_values[MAX_VALUES_SIZE / sizeof(fmi3Clock)];
    } values;

    /* String values (stored separately) */
    char string_values[MAX_STRINGS][MAX_STRING_LENGTH];
    size_t n_strings;

    /* Binary values */
    fmi3Byte binary_data[MAX_BINARY_SIZE];
    size_t binary_sizes[MAX_VALUE_REFERENCES];
    size_t total_binary_size;

    /* For Enter/Exit initialization mode */
    fmi3Boolean tolerance_defined;
    fmi3Float64 tolerance;
    fmi3Float64 start_time;
    fmi3Boolean stop_time_defined;
    fmi3Float64 stop_time;

    /* For DoStep */
    fmi3Float64 current_communication_point;
    fmi3Float64 communication_step_size;
    fmi3Boolean no_set_fmu_state_prior_to_current_point;

    /* For SetDebugLogging */
    size_t n_categories;
    char categories[MAX_CATEGORIES][MAX_STRING_LENGTH];

} CallRecord;

/* ============================================================================
 * Call Log API - Functions to control and access the call log
 * ============================================================================ */

/**
 * Clear the call log and reset the counter
 */
void dummyFmu_clearCallLog(void);

/**
 * Get the number of recorded calls
 */
size_t dummyFmu_getCallCount(void);

/**
 * Get a specific call record by index
 * Returns NULL if index is out of bounds
 */
const CallRecord* dummyFmu_getCallRecord(size_t index);

/**
 * Set the status to return from subsequent FMI function calls
 */
void dummyFmu_setNextReturnStatus(fmi3Status status);

/**
 * Get the currently configured return status
 */
fmi3Status dummyFmu_getNextReturnStatus(void);

/**
 * Configure values to return from Get functions
 */
void dummyFmu_setFloat64ReturnValues(const fmi3Float64* values, size_t count);
void dummyFmu_setFloat32ReturnValues(const fmi3Float32* values, size_t count);
void dummyFmu_setInt8ReturnValues(const fmi3Int8* values, size_t count);
void dummyFmu_setUInt8ReturnValues(const fmi3UInt8* values, size_t count);
void dummyFmu_setInt16ReturnValues(const fmi3Int16* values, size_t count);
void dummyFmu_setUInt16ReturnValues(const fmi3UInt16* values, size_t count);
void dummyFmu_setInt32ReturnValues(const fmi3Int32* values, size_t count);
void dummyFmu_setUInt32ReturnValues(const fmi3UInt32* values, size_t count);
void dummyFmu_setInt64ReturnValues(const fmi3Int64* values, size_t count);
void dummyFmu_setUInt64ReturnValues(const fmi3UInt64* values, size_t count);
void dummyFmu_setBooleanReturnValues(const fmi3Boolean* values, size_t count);
void dummyFmu_setClockReturnValues(const fmi3Clock* values, size_t count);
void dummyFmu_setStringReturnValues(const char** values, size_t count);
void dummyFmu_setBinaryReturnValues(const fmi3Byte* const* values, const size_t* sizes, size_t count);

/**
 * Configure DoStep output values
 */
void dummyFmu_setDoStepOutputs(fmi3Boolean eventHandlingNeeded,
                                fmi3Boolean terminateSimulation,
                                fmi3Boolean earlyReturn,
                                fmi3Float64 lastSuccessfulTime);

/**
 * Write call log to JSON file for cross-process access
 */
int dummyFmu_writeCallLogToFile(const char* filepath);

/**
 * Read call log from JSON file
 */
int dummyFmu_readCallLogFromFile(const char* filepath);

/**
 * Get the path to the call log file
 */
const char* dummyFmu_getCallLogPath(void);

/**
 * Set the path to the call log file
 */
void dummyFmu_setCallLogPath(const char* path);

/* ============================================================================
 * Internal functions for recording calls (used by fmi3Functions.c)
 * ============================================================================ */

/**
 * Add a new call record and return a pointer to fill in
 */
CallRecord* dummyFmu_addCallRecord(CallType type);

#ifdef __cplusplus
}
#endif

#endif /* CALL_RECORD_H */
