#include "call_record.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

/* ============================================================================
 * Global state
 * ============================================================================ */

static CallRecord g_call_log[MAX_CALL_RECORDS];
static size_t g_call_count = 0;
static fmi3Status g_next_return_status = fmi3OK;
static char g_call_log_path[1024] = "";
static int g_initialized = 0;

/* Constructor that runs when the shared library is loaded */
__attribute__((constructor))
static void dummyFmu_init(void) {
    if (g_initialized) return;
    g_initialized = 1;

    /* Read call log path from environment variable */
    const char* env_path = getenv("DUMMY_FMU_CALL_LOG");
    if (env_path && env_path[0] != '\0') {
        strncpy(g_call_log_path, env_path, sizeof(g_call_log_path) - 1);
        g_call_log_path[sizeof(g_call_log_path) - 1] = '\0';
        fprintf(stderr, "[DummyFMU] Call log path from env: %s\n", g_call_log_path);
    } else {
        strncpy(g_call_log_path, "/tmp/dummy_fmu_calls.json", sizeof(g_call_log_path) - 1);
        fprintf(stderr, "[DummyFMU] Using default call log path: %s\n", g_call_log_path);
    }
}

/* Return values for Get functions */
static fmi3Float64 g_float64_return_values[MAX_VALUES_SIZE / sizeof(fmi3Float64)];
static size_t g_float64_return_count = 0;

static fmi3Float32 g_float32_return_values[MAX_VALUES_SIZE / sizeof(fmi3Float32)];
static size_t g_float32_return_count = 0;

static fmi3Int8 g_int8_return_values[MAX_VALUES_SIZE / sizeof(fmi3Int8)];
static size_t g_int8_return_count = 0;

static fmi3UInt8 g_uint8_return_values[MAX_VALUES_SIZE / sizeof(fmi3UInt8)];
static size_t g_uint8_return_count = 0;

static fmi3Int16 g_int16_return_values[MAX_VALUES_SIZE / sizeof(fmi3Int16)];
static size_t g_int16_return_count = 0;

static fmi3UInt16 g_uint16_return_values[MAX_VALUES_SIZE / sizeof(fmi3UInt16)];
static size_t g_uint16_return_count = 0;

static fmi3Int32 g_int32_return_values[MAX_VALUES_SIZE / sizeof(fmi3Int32)];
static size_t g_int32_return_count = 0;

static fmi3UInt32 g_uint32_return_values[MAX_VALUES_SIZE / sizeof(fmi3UInt32)];
static size_t g_uint32_return_count = 0;

static fmi3Int64 g_int64_return_values[MAX_VALUES_SIZE / sizeof(fmi3Int64)];
static size_t g_int64_return_count = 0;

static fmi3UInt64 g_uint64_return_values[MAX_VALUES_SIZE / sizeof(fmi3UInt64)];
static size_t g_uint64_return_count = 0;

static fmi3Boolean g_boolean_return_values[MAX_VALUES_SIZE / sizeof(fmi3Boolean)];
static size_t g_boolean_return_count = 0;

static fmi3Clock g_clock_return_values[MAX_VALUES_SIZE / sizeof(fmi3Clock)];
static size_t g_clock_return_count = 0;

static char g_string_return_values[MAX_STRINGS][MAX_STRING_LENGTH];
static size_t g_string_return_count = 0;

static fmi3Byte g_binary_return_data[MAX_BINARY_SIZE];
static size_t g_binary_return_sizes[MAX_VALUE_REFERENCES];
static size_t g_binary_return_count = 0;
static size_t g_binary_return_offsets[MAX_VALUE_REFERENCES];

/* DoStep output values */
static fmi3Boolean g_do_step_event_handling_needed = fmi3False;
static fmi3Boolean g_do_step_terminate_simulation = fmi3False;
static fmi3Boolean g_do_step_early_return = fmi3False;
static fmi3Float64 g_do_step_last_successful_time = 0.0;

/* ============================================================================
 * Call Log API Implementation
 * ============================================================================ */

void dummyFmu_clearCallLog(void) {
    memset(g_call_log, 0, sizeof(g_call_log));
    g_call_count = 0;
    g_next_return_status = fmi3OK;
}

size_t dummyFmu_getCallCount(void) {
    return g_call_count;
}

const CallRecord* dummyFmu_getCallRecord(size_t index) {
    if (index >= g_call_count) {
        return NULL;
    }
    return &g_call_log[index];
}

void dummyFmu_setNextReturnStatus(fmi3Status status) {
    g_next_return_status = status;
}

fmi3Status dummyFmu_getNextReturnStatus(void) {
    return g_next_return_status;
}

/* ============================================================================
 * Return Value Setters
 * ============================================================================ */

void dummyFmu_setFloat64ReturnValues(const fmi3Float64* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Float64)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Float64);
    }
    memcpy(g_float64_return_values, values, count * sizeof(fmi3Float64));
    g_float64_return_count = count;
}

void dummyFmu_setFloat32ReturnValues(const fmi3Float32* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Float32)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Float32);
    }
    memcpy(g_float32_return_values, values, count * sizeof(fmi3Float32));
    g_float32_return_count = count;
}

void dummyFmu_setInt8ReturnValues(const fmi3Int8* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Int8)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Int8);
    }
    memcpy(g_int8_return_values, values, count * sizeof(fmi3Int8));
    g_int8_return_count = count;
}

void dummyFmu_setUInt8ReturnValues(const fmi3UInt8* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3UInt8)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3UInt8);
    }
    memcpy(g_uint8_return_values, values, count * sizeof(fmi3UInt8));
    g_uint8_return_count = count;
}

void dummyFmu_setInt16ReturnValues(const fmi3Int16* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Int16)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Int16);
    }
    memcpy(g_int16_return_values, values, count * sizeof(fmi3Int16));
    g_int16_return_count = count;
}

void dummyFmu_setUInt16ReturnValues(const fmi3UInt16* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3UInt16)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3UInt16);
    }
    memcpy(g_uint16_return_values, values, count * sizeof(fmi3UInt16));
    g_uint16_return_count = count;
}

void dummyFmu_setInt32ReturnValues(const fmi3Int32* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Int32)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Int32);
    }
    memcpy(g_int32_return_values, values, count * sizeof(fmi3Int32));
    g_int32_return_count = count;
}

void dummyFmu_setUInt32ReturnValues(const fmi3UInt32* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3UInt32)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3UInt32);
    }
    memcpy(g_uint32_return_values, values, count * sizeof(fmi3UInt32));
    g_uint32_return_count = count;
}

void dummyFmu_setInt64ReturnValues(const fmi3Int64* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Int64)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Int64);
    }
    memcpy(g_int64_return_values, values, count * sizeof(fmi3Int64));
    g_int64_return_count = count;
}

void dummyFmu_setUInt64ReturnValues(const fmi3UInt64* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3UInt64)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3UInt64);
    }
    memcpy(g_uint64_return_values, values, count * sizeof(fmi3UInt64));
    g_uint64_return_count = count;
}

void dummyFmu_setBooleanReturnValues(const fmi3Boolean* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Boolean)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Boolean);
    }
    memcpy(g_boolean_return_values, values, count * sizeof(fmi3Boolean));
    g_boolean_return_count = count;
}

void dummyFmu_setClockReturnValues(const fmi3Clock* values, size_t count) {
    if (count > MAX_VALUES_SIZE / sizeof(fmi3Clock)) {
        count = MAX_VALUES_SIZE / sizeof(fmi3Clock);
    }
    memcpy(g_clock_return_values, values, count * sizeof(fmi3Clock));
    g_clock_return_count = count;
}

void dummyFmu_setStringReturnValues(const char** values, size_t count) {
    if (count > MAX_STRINGS) {
        count = MAX_STRINGS;
    }
    for (size_t i = 0; i < count; i++) {
        strncpy(g_string_return_values[i], values[i], MAX_STRING_LENGTH - 1);
        g_string_return_values[i][MAX_STRING_LENGTH - 1] = '\0';
    }
    g_string_return_count = count;
}

void dummyFmu_setBinaryReturnValues(const fmi3Byte* const* values, const size_t* sizes, size_t count) {
    if (count > MAX_VALUE_REFERENCES) {
        count = MAX_VALUE_REFERENCES;
    }
    size_t offset = 0;
    for (size_t i = 0; i < count; i++) {
        size_t size = sizes[i];
        if (offset + size > MAX_BINARY_SIZE) {
            size = MAX_BINARY_SIZE - offset;
        }
        memcpy(g_binary_return_data + offset, values[i], size);
        g_binary_return_sizes[i] = size;
        g_binary_return_offsets[i] = offset;
        offset += size;
    }
    g_binary_return_count = count;
}

void dummyFmu_setDoStepOutputs(fmi3Boolean eventHandlingNeeded,
                                fmi3Boolean terminateSimulation,
                                fmi3Boolean earlyReturn,
                                fmi3Float64 lastSuccessfulTime) {
    g_do_step_event_handling_needed = eventHandlingNeeded;
    g_do_step_terminate_simulation = terminateSimulation;
    g_do_step_early_return = earlyReturn;
    g_do_step_last_successful_time = lastSuccessfulTime;
}

/* Getters for return values (used by fmi3Functions.c) */
void dummyFmu_getFloat64ReturnValues(fmi3Float64* values, size_t count) {
    size_t copy_count = (count < g_float64_return_count) ? count : g_float64_return_count;
    memcpy(values, g_float64_return_values, copy_count * sizeof(fmi3Float64));
}

void dummyFmu_getFloat32ReturnValues(fmi3Float32* values, size_t count) {
    size_t copy_count = (count < g_float32_return_count) ? count : g_float32_return_count;
    memcpy(values, g_float32_return_values, copy_count * sizeof(fmi3Float32));
}

void dummyFmu_getInt8ReturnValues(fmi3Int8* values, size_t count) {
    size_t copy_count = (count < g_int8_return_count) ? count : g_int8_return_count;
    memcpy(values, g_int8_return_values, copy_count * sizeof(fmi3Int8));
}

void dummyFmu_getUInt8ReturnValues(fmi3UInt8* values, size_t count) {
    size_t copy_count = (count < g_uint8_return_count) ? count : g_uint8_return_count;
    memcpy(values, g_uint8_return_values, copy_count * sizeof(fmi3UInt8));
}

void dummyFmu_getInt16ReturnValues(fmi3Int16* values, size_t count) {
    size_t copy_count = (count < g_int16_return_count) ? count : g_int16_return_count;
    memcpy(values, g_int16_return_values, copy_count * sizeof(fmi3Int16));
}

void dummyFmu_getUInt16ReturnValues(fmi3UInt16* values, size_t count) {
    size_t copy_count = (count < g_uint16_return_count) ? count : g_uint16_return_count;
    memcpy(values, g_uint16_return_values, copy_count * sizeof(fmi3UInt16));
}

void dummyFmu_getInt32ReturnValues(fmi3Int32* values, size_t count) {
    size_t copy_count = (count < g_int32_return_count) ? count : g_int32_return_count;
    memcpy(values, g_int32_return_values, copy_count * sizeof(fmi3Int32));
}

void dummyFmu_getUInt32ReturnValues(fmi3UInt32* values, size_t count) {
    size_t copy_count = (count < g_uint32_return_count) ? count : g_uint32_return_count;
    memcpy(values, g_uint32_return_values, copy_count * sizeof(fmi3UInt32));
}

void dummyFmu_getInt64ReturnValues(fmi3Int64* values, size_t count) {
    size_t copy_count = (count < g_int64_return_count) ? count : g_int64_return_count;
    memcpy(values, g_int64_return_values, copy_count * sizeof(fmi3Int64));
}

void dummyFmu_getUInt64ReturnValues(fmi3UInt64* values, size_t count) {
    size_t copy_count = (count < g_uint64_return_count) ? count : g_uint64_return_count;
    memcpy(values, g_uint64_return_values, copy_count * sizeof(fmi3UInt64));
}

void dummyFmu_getBooleanReturnValues(fmi3Boolean* values, size_t count) {
    size_t copy_count = (count < g_boolean_return_count) ? count : g_boolean_return_count;
    memcpy(values, g_boolean_return_values, copy_count * sizeof(fmi3Boolean));
}

void dummyFmu_getClockReturnValues(fmi3Clock* values, size_t count) {
    size_t copy_count = (count < g_clock_return_count) ? count : g_clock_return_count;
    memcpy(values, g_clock_return_values, copy_count * sizeof(fmi3Clock));
}

const char* dummyFmu_getStringReturnValue(size_t index) {
    if (index >= g_string_return_count) {
        return "";
    }
    return g_string_return_values[index];
}

void dummyFmu_getBinaryReturnValue(size_t index, const fmi3Byte** data, size_t* size) {
    if (index >= g_binary_return_count) {
        *data = NULL;
        *size = 0;
        return;
    }
    *data = g_binary_return_data + g_binary_return_offsets[index];
    *size = g_binary_return_sizes[index];
}

void dummyFmu_getDoStepOutputs(fmi3Boolean* eventHandlingNeeded,
                                fmi3Boolean* terminateSimulation,
                                fmi3Boolean* earlyReturn,
                                fmi3Float64* lastSuccessfulTime) {
    if (eventHandlingNeeded) *eventHandlingNeeded = g_do_step_event_handling_needed;
    if (terminateSimulation) *terminateSimulation = g_do_step_terminate_simulation;
    if (earlyReturn) *earlyReturn = g_do_step_early_return;
    if (lastSuccessfulTime) *lastSuccessfulTime = g_do_step_last_successful_time;
}

/* ============================================================================
 * Call Log Path Management
 * ============================================================================ */

const char* dummyFmu_getCallLogPath(void) {
    return g_call_log_path;
}

void dummyFmu_setCallLogPath(const char* path) {
    strncpy(g_call_log_path, path, sizeof(g_call_log_path) - 1);
    g_call_log_path[sizeof(g_call_log_path) - 1] = '\0';
}

/* ============================================================================
 * Internal Functions
 * ============================================================================ */

/* Check if the call log file indicates a reset request and clear if so */
static void checkResetRequest(void) {
    if (g_call_log_path[0] != '\0') {
        FILE* f = fopen(g_call_log_path, "r");
        if (f) {
            char buf[256];
            size_t n = fread(buf, 1, sizeof(buf)-1, f);
            fclose(f);
            buf[n] = '\0';
            /* Look for "call_count": 0 pattern indicating reset request */
            if (strstr(buf, "\"call_count\": 0") != NULL) {
                /* Clear in-memory log but don't recurse */
                memset(g_call_log, 0, sizeof(g_call_log));
                g_call_count = 0;
                g_next_return_status = fmi3OK;
            }
        }
    }
}

CallRecord* dummyFmu_addCallRecord(CallType type) {
    /* Check for reset request before adding new record */
    checkResetRequest();

    if (g_call_count >= MAX_CALL_RECORDS) {
        return NULL;
    }
    CallRecord* record = &g_call_log[g_call_count];
    memset(record, 0, sizeof(CallRecord));
    record->type = type;
    g_call_count++;
    return record;
}

/* ============================================================================
 * JSON Serialization/Deserialization for cross-process communication
 * ============================================================================ */

static const char* callTypeToString(CallType type) {
    switch (type) {
        case CALL_NONE: return "NONE";
        case CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE: return "INSTANTIATE_MODEL_EXCHANGE";
        case CALL_FMI3_INSTANTIATE_CO_SIMULATION: return "INSTANTIATE_CO_SIMULATION";
        case CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION: return "INSTANTIATE_SCHEDULED_EXECUTION";
        case CALL_FMI3_FREE_INSTANCE: return "FREE_INSTANCE";
        case CALL_FMI3_ENTER_INITIALIZATION_MODE: return "ENTER_INITIALIZATION_MODE";
        case CALL_FMI3_EXIT_INITIALIZATION_MODE: return "EXIT_INITIALIZATION_MODE";
        case CALL_FMI3_ENTER_EVENT_MODE: return "ENTER_EVENT_MODE";
        case CALL_FMI3_TERMINATE: return "TERMINATE";
        case CALL_FMI3_RESET: return "RESET";
        case CALL_FMI3_GET_FLOAT32: return "GET_FLOAT32";
        case CALL_FMI3_GET_FLOAT64: return "GET_FLOAT64";
        case CALL_FMI3_GET_INT8: return "GET_INT8";
        case CALL_FMI3_GET_UINT8: return "GET_UINT8";
        case CALL_FMI3_GET_INT16: return "GET_INT16";
        case CALL_FMI3_GET_UINT16: return "GET_UINT16";
        case CALL_FMI3_GET_INT32: return "GET_INT32";
        case CALL_FMI3_GET_UINT32: return "GET_UINT32";
        case CALL_FMI3_GET_INT64: return "GET_INT64";
        case CALL_FMI3_GET_UINT64: return "GET_UINT64";
        case CALL_FMI3_GET_BOOLEAN: return "GET_BOOLEAN";
        case CALL_FMI3_GET_STRING: return "GET_STRING";
        case CALL_FMI3_GET_BINARY: return "GET_BINARY";
        case CALL_FMI3_GET_CLOCK: return "GET_CLOCK";
        case CALL_FMI3_SET_FLOAT32: return "SET_FLOAT32";
        case CALL_FMI3_SET_FLOAT64: return "SET_FLOAT64";
        case CALL_FMI3_SET_INT8: return "SET_INT8";
        case CALL_FMI3_SET_UINT8: return "SET_UINT8";
        case CALL_FMI3_SET_INT16: return "SET_INT16";
        case CALL_FMI3_SET_UINT16: return "SET_UINT16";
        case CALL_FMI3_SET_INT32: return "SET_INT32";
        case CALL_FMI3_SET_UINT32: return "SET_UINT32";
        case CALL_FMI3_SET_INT64: return "SET_INT64";
        case CALL_FMI3_SET_UINT64: return "SET_UINT64";
        case CALL_FMI3_SET_BOOLEAN: return "SET_BOOLEAN";
        case CALL_FMI3_SET_STRING: return "SET_STRING";
        case CALL_FMI3_SET_BINARY: return "SET_BINARY";
        case CALL_FMI3_SET_CLOCK: return "SET_CLOCK";
        case CALL_FMI3_DO_STEP: return "DO_STEP";
        case CALL_FMI3_SET_DEBUG_LOGGING: return "SET_DEBUG_LOGGING";
        case CALL_FMI3_GET_VERSION: return "GET_VERSION";
        default: return "UNKNOWN";
    }
}

static CallType stringToCallType(const char* str) {
    if (strcmp(str, "NONE") == 0) return CALL_NONE;
    if (strcmp(str, "INSTANTIATE_MODEL_EXCHANGE") == 0) return CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE;
    if (strcmp(str, "INSTANTIATE_CO_SIMULATION") == 0) return CALL_FMI3_INSTANTIATE_CO_SIMULATION;
    if (strcmp(str, "INSTANTIATE_SCHEDULED_EXECUTION") == 0) return CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION;
    if (strcmp(str, "FREE_INSTANCE") == 0) return CALL_FMI3_FREE_INSTANCE;
    if (strcmp(str, "ENTER_INITIALIZATION_MODE") == 0) return CALL_FMI3_ENTER_INITIALIZATION_MODE;
    if (strcmp(str, "EXIT_INITIALIZATION_MODE") == 0) return CALL_FMI3_EXIT_INITIALIZATION_MODE;
    if (strcmp(str, "ENTER_EVENT_MODE") == 0) return CALL_FMI3_ENTER_EVENT_MODE;
    if (strcmp(str, "TERMINATE") == 0) return CALL_FMI3_TERMINATE;
    if (strcmp(str, "RESET") == 0) return CALL_FMI3_RESET;
    if (strcmp(str, "GET_FLOAT32") == 0) return CALL_FMI3_GET_FLOAT32;
    if (strcmp(str, "GET_FLOAT64") == 0) return CALL_FMI3_GET_FLOAT64;
    if (strcmp(str, "GET_INT8") == 0) return CALL_FMI3_GET_INT8;
    if (strcmp(str, "GET_UINT8") == 0) return CALL_FMI3_GET_UINT8;
    if (strcmp(str, "GET_INT16") == 0) return CALL_FMI3_GET_INT16;
    if (strcmp(str, "GET_UINT16") == 0) return CALL_FMI3_GET_UINT16;
    if (strcmp(str, "GET_INT32") == 0) return CALL_FMI3_GET_INT32;
    if (strcmp(str, "GET_UINT32") == 0) return CALL_FMI3_GET_UINT32;
    if (strcmp(str, "GET_INT64") == 0) return CALL_FMI3_GET_INT64;
    if (strcmp(str, "GET_UINT64") == 0) return CALL_FMI3_GET_UINT64;
    if (strcmp(str, "GET_BOOLEAN") == 0) return CALL_FMI3_GET_BOOLEAN;
    if (strcmp(str, "GET_STRING") == 0) return CALL_FMI3_GET_STRING;
    if (strcmp(str, "GET_BINARY") == 0) return CALL_FMI3_GET_BINARY;
    if (strcmp(str, "GET_CLOCK") == 0) return CALL_FMI3_GET_CLOCK;
    if (strcmp(str, "SET_FLOAT32") == 0) return CALL_FMI3_SET_FLOAT32;
    if (strcmp(str, "SET_FLOAT64") == 0) return CALL_FMI3_SET_FLOAT64;
    if (strcmp(str, "SET_INT8") == 0) return CALL_FMI3_SET_INT8;
    if (strcmp(str, "SET_UINT8") == 0) return CALL_FMI3_SET_UINT8;
    if (strcmp(str, "SET_INT16") == 0) return CALL_FMI3_SET_INT16;
    if (strcmp(str, "SET_UINT16") == 0) return CALL_FMI3_SET_UINT16;
    if (strcmp(str, "SET_INT32") == 0) return CALL_FMI3_SET_INT32;
    if (strcmp(str, "SET_UINT32") == 0) return CALL_FMI3_SET_UINT32;
    if (strcmp(str, "SET_INT64") == 0) return CALL_FMI3_SET_INT64;
    if (strcmp(str, "SET_UINT64") == 0) return CALL_FMI3_SET_UINT64;
    if (strcmp(str, "SET_BOOLEAN") == 0) return CALL_FMI3_SET_BOOLEAN;
    if (strcmp(str, "SET_STRING") == 0) return CALL_FMI3_SET_STRING;
    if (strcmp(str, "SET_BINARY") == 0) return CALL_FMI3_SET_BINARY;
    if (strcmp(str, "SET_CLOCK") == 0) return CALL_FMI3_SET_CLOCK;
    if (strcmp(str, "DO_STEP") == 0) return CALL_FMI3_DO_STEP;
    if (strcmp(str, "SET_DEBUG_LOGGING") == 0) return CALL_FMI3_SET_DEBUG_LOGGING;
    if (strcmp(str, "GET_VERSION") == 0) return CALL_FMI3_GET_VERSION;
    return CALL_NONE;
}

int dummyFmu_writeCallLogToFile(const char* filepath) {
    FILE* f = fopen(filepath, "w");
    if (!f) {
        return -1;
    }

    fprintf(f, "{\n  \"call_count\": %zu,\n  \"calls\": [\n", g_call_count);

    for (size_t i = 0; i < g_call_count; i++) {
        const CallRecord* rec = &g_call_log[i];
        fprintf(f, "    {\n");
        fprintf(f, "      \"type\": \"%s\",\n", callTypeToString(rec->type));
        fprintf(f, "      \"instance_index\": %d,\n", rec->instance_index);

        /* Instantiation fields */
        if (rec->type >= CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE &&
            rec->type <= CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION) {
            fprintf(f, "      \"instance_name\": \"%s\",\n", rec->instance_name);
            fprintf(f, "      \"instantiation_token\": \"%s\",\n", rec->instantiation_token);
            fprintf(f, "      \"resource_path\": \"%s\",\n", rec->resource_path);
            fprintf(f, "      \"visible\": %s,\n", rec->visible ? "true" : "false");
            fprintf(f, "      \"logging_on\": %s,\n", rec->logging_on ? "true" : "false");
            if (rec->type == CALL_FMI3_INSTANTIATE_CO_SIMULATION) {
                fprintf(f, "      \"event_mode_used\": %s,\n", rec->event_mode_used ? "true" : "false");
                fprintf(f, "      \"early_return_allowed\": %s,\n", rec->early_return_allowed ? "true" : "false");
            }
        }

        /* Value references and counts */
        if (rec->n_value_references > 0) {
            fprintf(f, "      \"value_references\": [");
            for (size_t j = 0; j < rec->n_value_references; j++) {
                fprintf(f, "%u%s", rec->value_references[j],
                        j < rec->n_value_references - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
            fprintf(f, "      \"n_value_references\": %zu,\n", rec->n_value_references);
        }
        if (rec->n_values > 0) {
            fprintf(f, "      \"n_values\": %zu,\n", rec->n_values);
        }

        /* Type-specific values */
        if (rec->type == CALL_FMI3_SET_FLOAT64 || rec->type == CALL_FMI3_GET_FLOAT64) {
            fprintf(f, "      \"float64_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%.17g%s", rec->values.float64_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_FLOAT32 || rec->type == CALL_FMI3_GET_FLOAT32) {
            fprintf(f, "      \"float32_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%.9g%s", (double)rec->values.float32_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_INT32 || rec->type == CALL_FMI3_GET_INT32) {
            fprintf(f, "      \"int32_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%d%s", rec->values.int32_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_UINT32 || rec->type == CALL_FMI3_GET_UINT32) {
            fprintf(f, "      \"uint32_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%u%s", rec->values.uint32_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_INT8 || rec->type == CALL_FMI3_GET_INT8) {
            fprintf(f, "      \"int8_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%d%s", (int)rec->values.int8_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_UINT8 || rec->type == CALL_FMI3_GET_UINT8) {
            fprintf(f, "      \"uint8_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%u%s", (unsigned)rec->values.uint8_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_INT16 || rec->type == CALL_FMI3_GET_INT16) {
            fprintf(f, "      \"int16_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%d%s", (int)rec->values.int16_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_UINT16 || rec->type == CALL_FMI3_GET_UINT16) {
            fprintf(f, "      \"uint16_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%u%s", (unsigned)rec->values.uint16_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_INT64 || rec->type == CALL_FMI3_GET_INT64) {
            fprintf(f, "      \"int64_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%lld%s", (long long)rec->values.int64_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_UINT64 || rec->type == CALL_FMI3_GET_UINT64) {
            fprintf(f, "      \"uint64_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%llu%s", (unsigned long long)rec->values.uint64_values[j],
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_CLOCK || rec->type == CALL_FMI3_GET_CLOCK) {
            fprintf(f, "      \"clock_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%s%s", rec->values.clock_values[j] == fmi3ClockActive ? "true" : "false",
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }
        if (rec->type == CALL_FMI3_SET_BOOLEAN || rec->type == CALL_FMI3_GET_BOOLEAN) {
            fprintf(f, "      \"boolean_values\": [");
            for (size_t j = 0; j < rec->n_values; j++) {
                fprintf(f, "%s%s", rec->values.boolean_values[j] ? "true" : "false",
                        j < rec->n_values - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }

        /* String values */
        if (rec->n_strings > 0) {
            fprintf(f, "      \"n_strings\": %zu,\n", rec->n_strings);
            fprintf(f, "      \"string_values\": [");
            for (size_t j = 0; j < rec->n_strings; j++) {
                fprintf(f, "\"%s\"%s", rec->string_values[j],
                        j < rec->n_strings - 1 ? ", " : "");
            }
            fprintf(f, "],\n");
        }

        /* Initialization mode parameters */
        if (rec->type == CALL_FMI3_ENTER_INITIALIZATION_MODE) {
            fprintf(f, "      \"tolerance_defined\": %s,\n", rec->tolerance_defined ? "true" : "false");
            fprintf(f, "      \"tolerance\": %.17g,\n", rec->tolerance);
            fprintf(f, "      \"start_time\": %.17g,\n", rec->start_time);
            fprintf(f, "      \"stop_time_defined\": %s,\n", rec->stop_time_defined ? "true" : "false");
            fprintf(f, "      \"stop_time\": %.17g,\n", rec->stop_time);
        }

        /* DoStep parameters */
        if (rec->type == CALL_FMI3_DO_STEP) {
            fprintf(f, "      \"current_communication_point\": %.17g,\n", rec->current_communication_point);
            fprintf(f, "      \"communication_step_size\": %.17g,\n", rec->communication_step_size);
            fprintf(f, "      \"no_set_fmu_state_prior_to_current_point\": %s,\n",
                    rec->no_set_fmu_state_prior_to_current_point ? "true" : "false");
        }

        /* SetDebugLogging parameters */
        if (rec->type == CALL_FMI3_SET_DEBUG_LOGGING) {
            fprintf(f, "      \"logging_on\": %s,\n", rec->logging_on ? "true" : "false");
            fprintf(f, "      \"n_categories\": %zu,\n", rec->n_categories);
            if (rec->n_categories > 0) {
                fprintf(f, "      \"categories\": [");
                for (size_t j = 0; j < rec->n_categories; j++) {
                    fprintf(f, "\"%s\"%s", rec->categories[j],
                            j < rec->n_categories - 1 ? ", " : "");
                }
                fprintf(f, "],\n");
            }
        }

        fprintf(f, "      \"_end\": true\n");
        fprintf(f, "    }%s\n", i < g_call_count - 1 ? "," : "");
    }

    fprintf(f, "  ]\n}\n");
    fclose(f);
    return 0;
}

int dummyFmu_readCallLogFromFile(const char* filepath) {
    /* Reading JSON in pure C is complex; for simplicity we'll rely on
       the test harness reading the file and parsing it with nlohmann/json.
       This function is a placeholder that can be implemented if needed. */
    (void)filepath;
    return -1;  /* Not implemented in pure C */
}
