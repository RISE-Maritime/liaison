/*
 * DummyFMU - Mock FMI3 implementation that records all function calls
 *
 * This file implements all FMI3 functions, recording their parameters
 * to a call log for later verification by tests.
 */

#include "call_record.h"
#include "fmi3FunctionTypes.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

/* External declarations for return value getters */
extern void dummyFmu_getFloat64ReturnValues(fmi3Float64* values, size_t count);
extern void dummyFmu_getFloat32ReturnValues(fmi3Float32* values, size_t count);
extern void dummyFmu_getInt8ReturnValues(fmi3Int8* values, size_t count);
extern void dummyFmu_getUInt8ReturnValues(fmi3UInt8* values, size_t count);
extern void dummyFmu_getInt16ReturnValues(fmi3Int16* values, size_t count);
extern void dummyFmu_getUInt16ReturnValues(fmi3UInt16* values, size_t count);
extern void dummyFmu_getInt32ReturnValues(fmi3Int32* values, size_t count);
extern void dummyFmu_getUInt32ReturnValues(fmi3UInt32* values, size_t count);
extern void dummyFmu_getInt64ReturnValues(fmi3Int64* values, size_t count);
extern void dummyFmu_getUInt64ReturnValues(fmi3UInt64* values, size_t count);
extern void dummyFmu_getBooleanReturnValues(fmi3Boolean* values, size_t count);
extern void dummyFmu_getClockReturnValues(fmi3Clock* values, size_t count);
extern const char* dummyFmu_getStringReturnValue(size_t index);
extern void dummyFmu_getBinaryReturnValue(size_t index, const fmi3Byte** data, size_t* size);
extern void dummyFmu_getDoStepOutputs(fmi3Boolean* eventHandlingNeeded,
                                       fmi3Boolean* terminateSimulation,
                                       fmi3Boolean* earlyReturn,
                                       fmi3Float64* lastSuccessfulTime);

/* Instance counter */
static int g_next_instance_index = 1;

/* Dummy instance structure */
typedef struct {
    int index;
    fmi3InstanceEnvironment instanceEnvironment;
    fmi3LogMessageCallback logMessage;
} DummyInstance;

/* Helper to write call log after each call */
static void writeCallLog(void) {
    const char* path = dummyFmu_getCallLogPath();
    if (path && path[0] != '\0') {
        dummyFmu_writeCallLogToFile(path);
    }
}

/* ============================================================================
 * Common Functions
 * ============================================================================ */

const char* fmi3GetVersion(void) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_GET_VERSION);
    (void)rec;
    writeCallLog();
    return "3.0";
}

fmi3Status fmi3SetDebugLogging(fmi3Instance instance,
                                fmi3Boolean loggingOn,
                                size_t nCategories,
                                const fmi3String categories[]) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_SET_DEBUG_LOGGING);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->logging_on = loggingOn;
        rec->n_categories = nCategories < MAX_CATEGORIES ? nCategories : MAX_CATEGORIES;
        for (size_t i = 0; i < rec->n_categories; i++) {
            strncpy(rec->categories[i], categories[i], MAX_STRING_LENGTH - 1);
            rec->categories[i][MAX_STRING_LENGTH - 1] = '\0';
        }
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

/* ============================================================================
 * Instantiation Functions
 * ============================================================================ */

fmi3Instance fmi3InstantiateModelExchange(
    fmi3String instanceName,
    fmi3String instantiationToken,
    fmi3String resourcePath,
    fmi3Boolean visible,
    fmi3Boolean loggingOn,
    fmi3InstanceEnvironment instanceEnvironment,
    fmi3LogMessageCallback logMessage) {

    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE);

    DummyInstance* instance = (DummyInstance*)malloc(sizeof(DummyInstance));
    if (!instance) return NULL;

    instance->index = g_next_instance_index++;
    instance->instanceEnvironment = instanceEnvironment;
    instance->logMessage = logMessage;

    if (rec) {
        rec->instance_index = instance->index;
        strncpy(rec->instance_name, instanceName ? instanceName : "", MAX_STRING_LENGTH - 1);
        strncpy(rec->instantiation_token, instantiationToken ? instantiationToken : "", MAX_STRING_LENGTH - 1);
        strncpy(rec->resource_path, resourcePath ? resourcePath : "", MAX_STRING_LENGTH - 1);
        rec->visible = visible;
        rec->logging_on = loggingOn;
    }

    writeCallLog();

    if (dummyFmu_getNextReturnStatus() == fmi3Fatal) {
        free(instance);
        return NULL;
    }

    return (fmi3Instance)instance;
}

fmi3Instance fmi3InstantiateCoSimulation(
    fmi3String instanceName,
    fmi3String instantiationToken,
    fmi3String resourcePath,
    fmi3Boolean visible,
    fmi3Boolean loggingOn,
    fmi3Boolean eventModeUsed,
    fmi3Boolean earlyReturnAllowed,
    const fmi3ValueReference requiredIntermediateVariables[],
    size_t nRequiredIntermediateVariables,
    fmi3InstanceEnvironment instanceEnvironment,
    fmi3LogMessageCallback logMessage,
    fmi3IntermediateUpdateCallback intermediateUpdate) {

    (void)intermediateUpdate;

    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_INSTANTIATE_CO_SIMULATION);

    DummyInstance* instance = (DummyInstance*)malloc(sizeof(DummyInstance));
    if (!instance) return NULL;

    instance->index = g_next_instance_index++;
    instance->instanceEnvironment = instanceEnvironment;
    instance->logMessage = logMessage;

    if (rec) {
        rec->instance_index = instance->index;
        strncpy(rec->instance_name, instanceName ? instanceName : "", MAX_STRING_LENGTH - 1);
        strncpy(rec->instantiation_token, instantiationToken ? instantiationToken : "", MAX_STRING_LENGTH - 1);
        strncpy(rec->resource_path, resourcePath ? resourcePath : "", MAX_STRING_LENGTH - 1);
        rec->visible = visible;
        rec->logging_on = loggingOn;
        rec->event_mode_used = eventModeUsed;
        rec->early_return_allowed = earlyReturnAllowed;
        rec->n_required_intermediate_variables = nRequiredIntermediateVariables < MAX_VALUE_REFERENCES ?
            nRequiredIntermediateVariables : MAX_VALUE_REFERENCES;
        if (requiredIntermediateVariables) {
            for (size_t i = 0; i < rec->n_required_intermediate_variables; i++) {
                rec->required_intermediate_variables[i] = requiredIntermediateVariables[i];
            }
        }
    }

    writeCallLog();

    if (dummyFmu_getNextReturnStatus() == fmi3Fatal) {
        free(instance);
        return NULL;
    }

    return (fmi3Instance)instance;
}

fmi3Instance fmi3InstantiateScheduledExecution(
    fmi3String instanceName,
    fmi3String instantiationToken,
    fmi3String resourcePath,
    fmi3Boolean visible,
    fmi3Boolean loggingOn,
    fmi3InstanceEnvironment instanceEnvironment,
    fmi3LogMessageCallback logMessage,
    fmi3ClockUpdateCallback clockUpdate,
    fmi3LockPreemptionCallback lockPreemption,
    fmi3UnlockPreemptionCallback unlockPreemption) {

    (void)clockUpdate;
    (void)lockPreemption;
    (void)unlockPreemption;

    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION);

    DummyInstance* instance = (DummyInstance*)malloc(sizeof(DummyInstance));
    if (!instance) return NULL;

    instance->index = g_next_instance_index++;
    instance->instanceEnvironment = instanceEnvironment;
    instance->logMessage = logMessage;

    if (rec) {
        rec->instance_index = instance->index;
        strncpy(rec->instance_name, instanceName ? instanceName : "", MAX_STRING_LENGTH - 1);
        strncpy(rec->instantiation_token, instantiationToken ? instantiationToken : "", MAX_STRING_LENGTH - 1);
        strncpy(rec->resource_path, resourcePath ? resourcePath : "", MAX_STRING_LENGTH - 1);
        rec->visible = visible;
        rec->logging_on = loggingOn;
    }

    writeCallLog();

    if (dummyFmu_getNextReturnStatus() == fmi3Fatal) {
        free(instance);
        return NULL;
    }

    return (fmi3Instance)instance;
}

void fmi3FreeInstance(fmi3Instance instance) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_FREE_INSTANCE);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
    }
    writeCallLog();
    if (instance) {
        free(instance);
    }
}

/* ============================================================================
 * Initialization Functions
 * ============================================================================ */

fmi3Status fmi3EnterInitializationMode(fmi3Instance instance,
                                        fmi3Boolean toleranceDefined,
                                        fmi3Float64 tolerance,
                                        fmi3Float64 startTime,
                                        fmi3Boolean stopTimeDefined,
                                        fmi3Float64 stopTime) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_ENTER_INITIALIZATION_MODE);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->tolerance_defined = toleranceDefined;
        rec->tolerance = tolerance;
        rec->start_time = startTime;
        rec->stop_time_defined = stopTimeDefined;
        rec->stop_time = stopTime;
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_EXIT_INITIALIZATION_MODE);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3EnterEventMode(fmi3Instance instance) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_ENTER_EVENT_MODE);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_TERMINATE);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3Reset(fmi3Instance instance) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_RESET);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

/* ============================================================================
 * Value Getter/Setter Macros
 * ============================================================================ */

#define DEFINE_FMI3_GET_VALUE_FUNCTION(TYPE, CTYPE, CALL_TYPE, GETTER_FUNC) \
fmi3Status fmi3Get##TYPE(fmi3Instance instance, \
                          const fmi3ValueReference valueReferences[], \
                          size_t nValueReferences, \
                          CTYPE values[], \
                          size_t nValues) { \
    CallRecord* rec = dummyFmu_addCallRecord(CALL_TYPE); \
    if (rec) { \
        DummyInstance* di = (DummyInstance*)instance; \
        rec->instance_index = di ? di->index : 0; \
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES; \
        for (size_t i = 0; i < rec->n_value_references; i++) { \
            rec->value_references[i] = valueReferences[i]; \
        } \
        rec->n_values = nValues; \
    } \
    GETTER_FUNC(values, nValues); \
    writeCallLog(); \
    return dummyFmu_getNextReturnStatus(); \
}

#define DEFINE_FMI3_SET_VALUE_FUNCTION(TYPE, CTYPE, CALL_TYPE, VALUE_ARRAY) \
fmi3Status fmi3Set##TYPE(fmi3Instance instance, \
                          const fmi3ValueReference valueReferences[], \
                          size_t nValueReferences, \
                          const CTYPE values[], \
                          size_t nValues) { \
    CallRecord* rec = dummyFmu_addCallRecord(CALL_TYPE); \
    if (rec) { \
        DummyInstance* di = (DummyInstance*)instance; \
        rec->instance_index = di ? di->index : 0; \
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES; \
        for (size_t i = 0; i < rec->n_value_references; i++) { \
            rec->value_references[i] = valueReferences[i]; \
        } \
        rec->n_values = nValues < (MAX_VALUES_SIZE / sizeof(CTYPE)) ? nValues : (MAX_VALUES_SIZE / sizeof(CTYPE)); \
        for (size_t i = 0; i < rec->n_values; i++) { \
            rec->values.VALUE_ARRAY[i] = values[i]; \
        } \
    } \
    writeCallLog(); \
    return dummyFmu_getNextReturnStatus(); \
}

/* Getter implementations */
DEFINE_FMI3_GET_VALUE_FUNCTION(Float64, fmi3Float64, CALL_FMI3_GET_FLOAT64, dummyFmu_getFloat64ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(Float32, fmi3Float32, CALL_FMI3_GET_FLOAT32, dummyFmu_getFloat32ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(Int8, fmi3Int8, CALL_FMI3_GET_INT8, dummyFmu_getInt8ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(UInt8, fmi3UInt8, CALL_FMI3_GET_UINT8, dummyFmu_getUInt8ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(Int16, fmi3Int16, CALL_FMI3_GET_INT16, dummyFmu_getInt16ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(UInt16, fmi3UInt16, CALL_FMI3_GET_UINT16, dummyFmu_getUInt16ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(Int32, fmi3Int32, CALL_FMI3_GET_INT32, dummyFmu_getInt32ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(UInt32, fmi3UInt32, CALL_FMI3_GET_UINT32, dummyFmu_getUInt32ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(Int64, fmi3Int64, CALL_FMI3_GET_INT64, dummyFmu_getInt64ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(UInt64, fmi3UInt64, CALL_FMI3_GET_UINT64, dummyFmu_getUInt64ReturnValues)
DEFINE_FMI3_GET_VALUE_FUNCTION(Boolean, fmi3Boolean, CALL_FMI3_GET_BOOLEAN, dummyFmu_getBooleanReturnValues)

/* Setter implementations */
DEFINE_FMI3_SET_VALUE_FUNCTION(Float64, fmi3Float64, CALL_FMI3_SET_FLOAT64, float64_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(Float32, fmi3Float32, CALL_FMI3_SET_FLOAT32, float32_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int8, fmi3Int8, CALL_FMI3_SET_INT8, int8_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt8, fmi3UInt8, CALL_FMI3_SET_UINT8, uint8_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int16, fmi3Int16, CALL_FMI3_SET_INT16, int16_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt16, fmi3UInt16, CALL_FMI3_SET_UINT16, uint16_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int32, fmi3Int32, CALL_FMI3_SET_INT32, int32_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt32, fmi3UInt32, CALL_FMI3_SET_UINT32, uint32_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int64, fmi3Int64, CALL_FMI3_SET_INT64, int64_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt64, fmi3UInt64, CALL_FMI3_SET_UINT64, uint64_values)
DEFINE_FMI3_SET_VALUE_FUNCTION(Boolean, fmi3Boolean, CALL_FMI3_SET_BOOLEAN, boolean_values)

/* Clock functions (special signature without nValues) */
fmi3Status fmi3GetClock(fmi3Instance instance,
                        const fmi3ValueReference valueReferences[],
                        size_t nValueReferences,
                        fmi3Clock values[]) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_GET_CLOCK);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES;
        for (size_t i = 0; i < rec->n_value_references; i++) {
            rec->value_references[i] = valueReferences[i];
        }
        rec->n_values = nValueReferences;
    }
    dummyFmu_getClockReturnValues(values, nValueReferences);
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3SetClock(fmi3Instance instance,
                        const fmi3ValueReference valueReferences[],
                        size_t nValueReferences,
                        const fmi3Clock values[]) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_SET_CLOCK);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES;
        for (size_t i = 0; i < rec->n_value_references; i++) {
            rec->value_references[i] = valueReferences[i];
        }
        rec->n_values = nValueReferences < (MAX_VALUES_SIZE / sizeof(fmi3Clock)) ?
            nValueReferences : (MAX_VALUES_SIZE / sizeof(fmi3Clock));
        for (size_t i = 0; i < rec->n_values; i++) {
            rec->values.clock_values[i] = values[i];
        }
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

/* ============================================================================
 * String Functions (special handling)
 * ============================================================================ */

/* Static storage for string return values */
static const char* g_string_ptrs[MAX_STRINGS];

fmi3Status fmi3GetString(fmi3Instance instance,
                          const fmi3ValueReference valueReferences[],
                          size_t nValueReferences,
                          fmi3String values[],
                          size_t nValues) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_GET_STRING);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES;
        for (size_t i = 0; i < rec->n_value_references; i++) {
            rec->value_references[i] = valueReferences[i];
        }
        rec->n_values = nValues;
    }
    /* Return configured string values */
    for (size_t i = 0; i < nValues; i++) {
        g_string_ptrs[i] = dummyFmu_getStringReturnValue(i);
        values[i] = g_string_ptrs[i];
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3SetString(fmi3Instance instance,
                          const fmi3ValueReference valueReferences[],
                          size_t nValueReferences,
                          const fmi3String values[],
                          size_t nValues) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_SET_STRING);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES;
        for (size_t i = 0; i < rec->n_value_references; i++) {
            rec->value_references[i] = valueReferences[i];
        }
        rec->n_values = nValues;
        rec->n_strings = nValues < MAX_STRINGS ? nValues : MAX_STRINGS;
        for (size_t i = 0; i < rec->n_strings; i++) {
            if (values[i]) {
                strncpy(rec->string_values[i], values[i], MAX_STRING_LENGTH - 1);
                rec->string_values[i][MAX_STRING_LENGTH - 1] = '\0';
            } else {
                rec->string_values[i][0] = '\0';
            }
        }
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

/* ============================================================================
 * Binary Functions (special handling)
 * ============================================================================ */

static fmi3Byte g_binary_return_storage[MAX_BINARY_SIZE];
static const fmi3Byte* g_binary_ptrs[MAX_VALUE_REFERENCES];

fmi3Status fmi3GetBinary(fmi3Instance instance,
                          const fmi3ValueReference valueReferences[],
                          size_t nValueReferences,
                          size_t valueSizes[],
                          fmi3Binary values[],
                          size_t nValues) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_GET_BINARY);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES;
        for (size_t i = 0; i < rec->n_value_references; i++) {
            rec->value_references[i] = valueReferences[i];
        }
        rec->n_values = nValues;
    }
    /* Return configured binary values */
    size_t offset = 0;
    for (size_t i = 0; i < nValues; i++) {
        const fmi3Byte* data;
        size_t size;
        dummyFmu_getBinaryReturnValue(i, &data, &size);
        if (data && size > 0 && offset + size <= MAX_BINARY_SIZE) {
            memcpy(g_binary_return_storage + offset, data, size);
            g_binary_ptrs[i] = g_binary_return_storage + offset;
            valueSizes[i] = size;
            offset += size;
        } else {
            g_binary_ptrs[i] = NULL;
            valueSizes[i] = 0;
        }
        values[i] = g_binary_ptrs[i];
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

fmi3Status fmi3SetBinary(fmi3Instance instance,
                          const fmi3ValueReference valueReferences[],
                          size_t nValueReferences,
                          const size_t valueSizes[],
                          const fmi3Binary values[],
                          size_t nValues) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_SET_BINARY);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->n_value_references = nValueReferences < MAX_VALUE_REFERENCES ? nValueReferences : MAX_VALUE_REFERENCES;
        for (size_t i = 0; i < rec->n_value_references; i++) {
            rec->value_references[i] = valueReferences[i];
        }
        rec->n_values = nValues;
        /* Store binary data */
        size_t offset = 0;
        for (size_t i = 0; i < nValues && i < MAX_VALUE_REFERENCES; i++) {
            rec->binary_sizes[i] = valueSizes[i];
            if (values[i] && valueSizes[i] > 0 && offset + valueSizes[i] <= MAX_BINARY_SIZE) {
                memcpy(rec->binary_data + offset, values[i], valueSizes[i]);
                offset += valueSizes[i];
            }
        }
        rec->total_binary_size = offset;
    }
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

/* ============================================================================
 * DoStep Function
 * ============================================================================ */

fmi3Status fmi3DoStep(fmi3Instance instance,
                       fmi3Float64 currentCommunicationPoint,
                       fmi3Float64 communicationStepSize,
                       fmi3Boolean noSetFMUStatePriorToCurrentPoint,
                       fmi3Boolean* eventHandlingNeeded,
                       fmi3Boolean* terminateSimulation,
                       fmi3Boolean* earlyReturn,
                       fmi3Float64* lastSuccessfulTime) {
    CallRecord* rec = dummyFmu_addCallRecord(CALL_FMI3_DO_STEP);
    if (rec) {
        DummyInstance* di = (DummyInstance*)instance;
        rec->instance_index = di ? di->index : 0;
        rec->current_communication_point = currentCommunicationPoint;
        rec->communication_step_size = communicationStepSize;
        rec->no_set_fmu_state_prior_to_current_point = noSetFMUStatePriorToCurrentPoint;
    }
    /* Set output values */
    dummyFmu_getDoStepOutputs(eventHandlingNeeded, terminateSimulation,
                               earlyReturn, lastSuccessfulTime);
    writeCallLog();
    return dummyFmu_getNextReturnStatus();
}

/* ============================================================================
 * Unimplemented Functions (return fmi3Error)
 * ============================================================================ */

fmi3Status fmi3GetNumberOfVariableDependencies(fmi3Instance instance,
                                                fmi3ValueReference valueReference,
                                                size_t* nDependencies) {
    (void)instance; (void)valueReference; (void)nDependencies;
    return fmi3Error;
}

fmi3Status fmi3GetVariableDependencies(fmi3Instance instance,
                                        fmi3ValueReference dependent,
                                        size_t elementIndicesOfDependent[],
                                        fmi3ValueReference independents[],
                                        size_t elementIndicesOfIndependents[],
                                        fmi3DependencyKind dependencyKinds[],
                                        size_t nDependencies) {
    (void)instance; (void)dependent; (void)elementIndicesOfDependent;
    (void)independents; (void)elementIndicesOfIndependents;
    (void)dependencyKinds; (void)nDependencies;
    return fmi3Error;
}

fmi3Status fmi3GetFMUState(fmi3Instance instance, fmi3FMUState* FMUState) {
    (void)instance; (void)FMUState;
    return fmi3Error;
}

fmi3Status fmi3SetFMUState(fmi3Instance instance, fmi3FMUState FMUState) {
    (void)instance; (void)FMUState;
    return fmi3Error;
}

fmi3Status fmi3FreeFMUState(fmi3Instance instance, fmi3FMUState* FMUState) {
    (void)instance; (void)FMUState;
    return fmi3Error;
}

fmi3Status fmi3SerializedFMUStateSize(fmi3Instance instance,
                                       fmi3FMUState FMUState,
                                       size_t* size) {
    (void)instance; (void)FMUState; (void)size;
    return fmi3Error;
}

fmi3Status fmi3SerializeFMUState(fmi3Instance instance,
                                  fmi3FMUState FMUState,
                                  fmi3Byte serializedState[],
                                  size_t size) {
    (void)instance; (void)FMUState; (void)serializedState; (void)size;
    return fmi3Error;
}

fmi3Status fmi3DeserializeFMUState(fmi3Instance instance,
                                    const fmi3Byte serializedState[],
                                    size_t size,
                                    fmi3FMUState* FMUState) {
    (void)instance; (void)serializedState; (void)size; (void)FMUState;
    return fmi3Error;
}

fmi3Status fmi3GetDirectionalDerivative(fmi3Instance instance,
                                         const fmi3ValueReference unknowns[],
                                         size_t nUnknowns,
                                         const fmi3ValueReference knowns[],
                                         size_t nKnowns,
                                         const fmi3Float64 seed[],
                                         size_t nSeed,
                                         fmi3Float64 sensitivity[],
                                         size_t nSensitivity) {
    (void)instance; (void)unknowns; (void)nUnknowns; (void)knowns;
    (void)nKnowns; (void)seed; (void)nSeed; (void)sensitivity; (void)nSensitivity;
    return fmi3Error;
}

fmi3Status fmi3GetAdjointDerivative(fmi3Instance instance,
                                     const fmi3ValueReference unknowns[],
                                     size_t nUnknowns,
                                     const fmi3ValueReference knowns[],
                                     size_t nKnowns,
                                     const fmi3Float64 seed[],
                                     size_t nSeed,
                                     fmi3Float64 sensitivity[],
                                     size_t nSensitivity) {
    (void)instance; (void)unknowns; (void)nUnknowns; (void)knowns;
    (void)nKnowns; (void)seed; (void)nSeed; (void)sensitivity; (void)nSensitivity;
    return fmi3Error;
}

fmi3Status fmi3EnterConfigurationMode(fmi3Instance instance) {
    (void)instance;
    return fmi3Error;
}

fmi3Status fmi3ExitConfigurationMode(fmi3Instance instance) {
    (void)instance;
    return fmi3Error;
}

fmi3Status fmi3GetIntervalDecimal(fmi3Instance instance,
                                   const fmi3ValueReference valueReferences[],
                                   size_t nValueReferences,
                                   fmi3Float64 intervals[],
                                   fmi3IntervalQualifier qualifiers[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences;
    (void)intervals; (void)qualifiers;
    return fmi3Error;
}

fmi3Status fmi3GetIntervalFraction(fmi3Instance instance,
                                    const fmi3ValueReference valueReferences[],
                                    size_t nValueReferences,
                                    fmi3UInt64 counters[],
                                    fmi3UInt64 resolutions[],
                                    fmi3IntervalQualifier qualifiers[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences;
    (void)counters; (void)resolutions; (void)qualifiers;
    return fmi3Error;
}

fmi3Status fmi3GetShiftDecimal(fmi3Instance instance,
                                const fmi3ValueReference valueReferences[],
                                size_t nValueReferences,
                                fmi3Float64 shifts[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences; (void)shifts;
    return fmi3Error;
}

fmi3Status fmi3GetShiftFraction(fmi3Instance instance,
                                 const fmi3ValueReference valueReferences[],
                                 size_t nValueReferences,
                                 fmi3UInt64 counters[],
                                 fmi3UInt64 resolutions[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences;
    (void)counters; (void)resolutions;
    return fmi3Error;
}

fmi3Status fmi3SetIntervalDecimal(fmi3Instance instance,
                                   const fmi3ValueReference valueReferences[],
                                   size_t nValueReferences,
                                   const fmi3Float64 intervals[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences; (void)intervals;
    return fmi3Error;
}

fmi3Status fmi3SetIntervalFraction(fmi3Instance instance,
                                    const fmi3ValueReference valueReferences[],
                                    size_t nValueReferences,
                                    const fmi3UInt64 counters[],
                                    const fmi3UInt64 resolutions[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences;
    (void)counters; (void)resolutions;
    return fmi3Error;
}

fmi3Status fmi3SetShiftDecimal(fmi3Instance instance,
                                const fmi3ValueReference valueReferences[],
                                size_t nValueReferences,
                                const fmi3Float64 shifts[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences; (void)shifts;
    return fmi3Error;
}

fmi3Status fmi3SetShiftFraction(fmi3Instance instance,
                                 const fmi3ValueReference valueReferences[],
                                 size_t nValueReferences,
                                 const fmi3UInt64 counters[],
                                 const fmi3UInt64 resolutions[]) {
    (void)instance; (void)valueReferences; (void)nValueReferences;
    (void)counters; (void)resolutions;
    return fmi3Error;
}

fmi3Status fmi3EvaluateDiscreteStates(fmi3Instance instance) {
    (void)instance;
    return fmi3Error;
}

fmi3Status fmi3UpdateDiscreteStates(fmi3Instance instance,
                                     fmi3Boolean* discreteStatesNeedUpdate,
                                     fmi3Boolean* terminateSimulation,
                                     fmi3Boolean* nominalsOfContinuousStatesChanged,
                                     fmi3Boolean* valuesOfContinuousStatesChanged,
                                     fmi3Boolean* nextEventTimeDefined,
                                     fmi3Float64* nextEventTime) {
    (void)instance; (void)discreteStatesNeedUpdate; (void)terminateSimulation;
    (void)nominalsOfContinuousStatesChanged; (void)valuesOfContinuousStatesChanged;
    (void)nextEventTimeDefined; (void)nextEventTime;
    return fmi3Error;
}

fmi3Status fmi3EnterContinuousTimeMode(fmi3Instance instance) {
    (void)instance;
    return fmi3Error;
}

fmi3Status fmi3CompletedIntegratorStep(fmi3Instance instance,
                                        fmi3Boolean noSetFMUStatePriorToCurrentPoint,
                                        fmi3Boolean* enterEventMode,
                                        fmi3Boolean* terminateSimulation) {
    (void)instance; (void)noSetFMUStatePriorToCurrentPoint;
    (void)enterEventMode; (void)terminateSimulation;
    return fmi3Error;
}

fmi3Status fmi3SetTime(fmi3Instance instance, fmi3Float64 time) {
    (void)instance; (void)time;
    return fmi3Error;
}

fmi3Status fmi3SetContinuousStates(fmi3Instance instance,
                                    const fmi3Float64 continuousStates[],
                                    size_t nContinuousStates) {
    (void)instance; (void)continuousStates; (void)nContinuousStates;
    return fmi3Error;
}

fmi3Status fmi3GetContinuousStateDerivatives(fmi3Instance instance,
                                              fmi3Float64 derivatives[],
                                              size_t nContinuousStates) {
    (void)instance; (void)derivatives; (void)nContinuousStates;
    return fmi3Error;
}

fmi3Status fmi3GetEventIndicators(fmi3Instance instance,
                                   fmi3Float64 eventIndicators[],
                                   size_t nEventIndicators) {
    (void)instance; (void)eventIndicators; (void)nEventIndicators;
    return fmi3Error;
}

fmi3Status fmi3GetContinuousStates(fmi3Instance instance,
                                    fmi3Float64 continuousStates[],
                                    size_t nContinuousStates) {
    (void)instance; (void)continuousStates; (void)nContinuousStates;
    return fmi3Error;
}

fmi3Status fmi3GetNominalsOfContinuousStates(fmi3Instance instance,
                                              fmi3Float64 nominals[],
                                              size_t nContinuousStates) {
    (void)instance; (void)nominals; (void)nContinuousStates;
    return fmi3Error;
}

fmi3Status fmi3GetNumberOfEventIndicators(fmi3Instance instance,
                                           size_t* nEventIndicators) {
    (void)instance; (void)nEventIndicators;
    return fmi3Error;
}

fmi3Status fmi3GetNumberOfContinuousStates(fmi3Instance instance,
                                            size_t* nContinuousStates) {
    (void)instance; (void)nContinuousStates;
    return fmi3Error;
}

fmi3Status fmi3EnterStepMode(fmi3Instance instance) {
    (void)instance;
    return fmi3Error;
}

fmi3Status fmi3GetOutputDerivatives(fmi3Instance instance,
                                     const fmi3ValueReference valueReferences[],
                                     size_t nValueReferences,
                                     const fmi3Int32 orders[],
                                     fmi3Float64 values[],
                                     size_t nValues) {
    (void)instance; (void)valueReferences; (void)nValueReferences;
    (void)orders; (void)values; (void)nValues;
    return fmi3Error;
}

fmi3Status fmi3ActivateModelPartition(fmi3Instance instance,
                                       fmi3ValueReference clockReference,
                                       fmi3Float64 activationTime) {
    (void)instance; (void)clockReference; (void)activationTime;
    return fmi3Error;
}
