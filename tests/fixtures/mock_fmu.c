// Mock FMU for testing purposes
// Implements a simple counter FMU with basic FMI 3.0 functions

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// FMI 3.0 types
typedef void* fmi3Instance;
typedef void* fmi3InstanceEnvironment;
typedef unsigned int fmi3ValueReference;
typedef double fmi3Float64;
typedef float fmi3Float32;
typedef int fmi3Int32;
typedef int fmi3Boolean;
typedef char fmi3Char;
typedef const fmi3Char* fmi3String;

// FMI 3.0 status codes
typedef enum {
    fmi3OK = 0,
    fmi3Warning = 1,
    fmi3Discard = 2,
    fmi3Error = 3,
    fmi3Fatal = 4
} fmi3Status;

// Callback function types
typedef void (*fmi3LogMessageCallback)(
    fmi3InstanceEnvironment instanceEnvironment,
    fmi3Status status,
    fmi3String category,
    fmi3String message);

typedef void (*fmi3IntermediateUpdateCallback)(
    fmi3InstanceEnvironment instanceEnvironment,
    fmi3Float64 intermediateUpdateTime,
    fmi3Boolean eventOccurred,
    fmi3Boolean clocksTicked,
    fmi3Boolean intermediateVariableSetAllowed,
    fmi3Boolean intermediateVariableGetAllowed,
    fmi3Boolean intermediateStepFinished,
    fmi3Boolean canReturnEarly,
    fmi3Boolean* earlyReturnRequested,
    fmi3Float64* earlyReturnTime);

// Internal state structure
typedef struct {
    char* instanceName;
    fmi3Float64 time;
    fmi3Float64 counter;
    fmi3Float64 stepSize;
    fmi3LogMessageCallback logCallback;
    fmi3InstanceEnvironment logCallbackEnvironment;
} MockFMUInstance;

// Version information
const char* fmi3GetVersion() {
    return "3.0";
}

// Instantiation for Co-Simulation
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

    MockFMUInstance* instance = (MockFMUInstance*)malloc(sizeof(MockFMUInstance));
    if (!instance) {
        return NULL;
    }

    instance->instanceName = strdup(instanceName);
    instance->time = 0.0;
    instance->counter = 0.0;
    instance->stepSize = 0.1;
    instance->logCallback = logMessage;
    instance->logCallbackEnvironment = instanceEnvironment;

    if (loggingOn && logMessage) {
        logMessage(instanceEnvironment, fmi3OK, "logAll", "Mock FMU instantiated successfully");
    }

    return (fmi3Instance)instance;
}

// Enter Initialization Mode
fmi3Status fmi3EnterInitializationMode(
    fmi3Instance instance,
    fmi3Boolean toleranceDefined,
    fmi3Float64 tolerance,
    fmi3Float64 startTime,
    fmi3Boolean stopTimeDefined,
    fmi3Float64 stopTime) {

    if (!instance) {
        return fmi3Error;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;
    fmu->time = startTime;

    if (fmu->logCallback) {
        fmu->logCallback(fmu->logCallbackEnvironment, fmi3OK, "logAll", "Entered initialization mode");
    }

    return fmi3OK;
}

// Exit Initialization Mode
fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    if (!instance) {
        return fmi3Error;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;

    if (fmu->logCallback) {
        fmu->logCallback(fmu->logCallbackEnvironment, fmi3OK, "logAll", "Exited initialization mode");
    }

    return fmi3OK;
}

// Enter Step Mode
fmi3Status fmi3EnterStepMode(fmi3Instance instance) {
    if (!instance) {
        return fmi3Error;
    }

    return fmi3OK;
}

// Do Step
fmi3Status fmi3DoStep(
    fmi3Instance instance,
    fmi3Float64 currentCommunicationPoint,
    fmi3Float64 communicationStepSize,
    fmi3Boolean noSetFMUStatePriorToCurrentPoint,
    fmi3Boolean* eventHandlingNeeded,
    fmi3Boolean* terminateSimulation,
    fmi3Boolean* earlyReturn,
    fmi3Float64* lastSuccessfulTime) {

    if (!instance) {
        return fmi3Error;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;

    // Update time and counter
    fmu->time = currentCommunicationPoint + communicationStepSize;
    fmu->counter += 1.0;

    *eventHandlingNeeded = 0;
    *terminateSimulation = 0;
    *earlyReturn = 0;
    *lastSuccessfulTime = fmu->time;

    return fmi3OK;
}

// Get Float64
fmi3Status fmi3GetFloat64(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64 values[],
    size_t nValues) {

    if (!instance) {
        return fmi3Error;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;

    for (size_t i = 0; i < nValueReferences; i++) {
        switch (valueReferences[i]) {
            case 0:  // time
                values[i] = fmu->time;
                break;
            case 1:  // counter
                values[i] = fmu->counter;
                break;
            default:
                return fmi3Error;
        }
    }

    return fmi3OK;
}

// Set Float64
fmi3Status fmi3SetFloat64(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const fmi3Float64 values[],
    size_t nValues) {

    if (!instance) {
        return fmi3Error;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;

    for (size_t i = 0; i < nValueReferences; i++) {
        switch (valueReferences[i]) {
            case 0:  // time (read-only, but we'll allow it for testing)
                fmu->time = values[i];
                break;
            case 1:  // counter
                fmu->counter = values[i];
                break;
            case 2:  // stepSize
                fmu->stepSize = values[i];
                break;
            default:
                return fmi3Error;
        }
    }

    return fmi3OK;
}

// Terminate
fmi3Status fmi3Terminate(fmi3Instance instance) {
    if (!instance) {
        return fmi3Error;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;

    if (fmu->logCallback) {
        fmu->logCallback(fmu->logCallbackEnvironment, fmi3OK, "logAll", "Mock FMU terminated");
    }

    return fmi3OK;
}

// Free Instance
void fmi3FreeInstance(fmi3Instance instance) {
    if (!instance) {
        return;
    }

    MockFMUInstance* fmu = (MockFMUInstance*)instance;

    if (fmu->instanceName) {
        free(fmu->instanceName);
    }

    free(fmu);
}
