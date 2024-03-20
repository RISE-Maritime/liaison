#include "fmi3Functions.h"
#include <stdexcept>
#include <grpcpp/grpcpp.h>
#include "fmi3.grpc.pb.h"
#include "fmi3Functions.h"


#define TRY_CODE(FUNCTION_CALL, ERROR_MSG, ERROR_RETURN) \
    try {                                                          \
        FUNCTION_CALL                                             \
    } catch (const std::exception &e) {                            \
        std::cerr << ERROR_MSG << e.what() << std::endl;           \
        return ERROR_RETURN;                                       \
    }

// gRPC client
std::unique_ptr<proto::fmi3::Stub> client;

class FmuInstance {
public:
    FmuInstance(int k) {
        key = k;
    }
    int key;
};

extern "C" {

const char* fmi3GetVersion() {
    return "3.0";
}

fmi3Instance fmi3InstantiateCoSimulation(
    fmi3String                     instanceName,
    fmi3String                     instantiationToken,
    fmi3String                     resourcePath,
    fmi3Boolean                    visible,
    fmi3Boolean                    loggingOn,
    fmi3Boolean                    eventModeUsed,
    fmi3Boolean                    earlyReturnAllowed,
    const fmi3ValueReference       requiredIntermediateVariables[],
    size_t                         nRequiredIntermediateVariables,
    fmi3InstanceEnvironment        instanceEnvironment,
    fmi3LogMessageCallback         logMessage,
    fmi3IntermediateUpdateCallback intermediateUpdate) {
    
    client = proto::fmi3::NewStub(grpc::CreateChannel("localhost:50051", grpc::InsecureChannelCredentials()));

    proto::Empty request;
    proto::Instance reply;
    grpc::ClientContext context;
    grpc::Status status = client->instantiateCoSimulation(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
        return nullptr;
    }
    FmuInstance* fmu_instance = new FmuInstance(reply.key());
    return reinterpret_cast<fmi3Instance>(fmu_instance);
}

fmi3Status fmi3EnterInitializationMode(
    fmi3Instance instance,
    fmi3Boolean toleranceDefined,
    fmi3Float64 tolerance,
    fmi3Float64 startTime,
    fmi3Boolean stopTimeDefined,
    fmi3Float64 stopTime) {

    FmuInstance* fmu_instance = reinterpret_cast<FmuInstance*>(instance);
    proto::enterInitializationModeRequest request;
    request.set_key(fmu_instance->key);
    request.set_tolerancedefined(toleranceDefined);
    request.set_tolerance(tolerance);
    request.set_starttime(startTime);
    request.set_stoptimedefined(stopTimeDefined);
    request.set_stoptime(stopTime);
    proto::Empty reply;
    grpc::ClientContext context;
    grpc::Status status = client->enterInitializationMode(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
        return fmi3Error;
    } 
    return fmi3OK;

}



fmi3Status fmi3DoStep(fmi3Instance instance,
    fmi3Float64 currentCommunicationPoint,
    fmi3Float64 communicationStepSize,
    fmi3Boolean noSetFMUStatePriorToCurrentPoint,
    fmi3Boolean* eventHandlingNeeded,
    fmi3Boolean* terminateSimulation,
    fmi3Boolean* earlyReturn,
    fmi3Float64* lastSuccessfulTime) {

    proto::Instance request;
    FmuInstance* fmu_instance = reinterpret_cast<FmuInstance*>(instance);
    request.set_key(fmu_instance->key);  
    proto::Empty reply;
    grpc::ClientContext context;
    grpc::Status status = client->doStep(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
        return fmi3Error;
    } 
    return fmi3OK;
}

fmi3Status fmi3GetFloat64(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64 values[],
    size_t nValues) {
    
    proto::getValue64Request request;
    FmuInstance* fmu_instance = reinterpret_cast<FmuInstance*>(instance);
    request.set_key(fmu_instance->key);  

    for (size_t i = 0; i < nValueReferences; ++i) {
        request.add_valuereferences(valueReferences[i]);  
    }

    proto::getFloat64Reply reply;
    grpc::ClientContext context;
    grpc::Status status = client->getFloat64(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
        return fmi3Error;
    }

    for (int i = 0; i < reply.values_size(); ++i) {
        values[i] = reply.values(i);
    }

    return fmi3OK;
}




fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    proto::Instance request;
    proto::Empty reply;
    grpc::ClientContext context;
    grpc::Status status = client->exitInitializationMode(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
        return fmi3Error;
    }
    return fmi3OK;
}


void fmi3FreeInstance(fmi3Instance instance) {
    FmuInstance* fmu_instance = reinterpret_cast<FmuInstance*>(instance);
    proto::Instance request;
    request.set_key(fmu_instance->key);
    proto::Empty reply;
    grpc::ClientContext context;
    grpc::Status status = client->freeInstance(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
    } 
    delete fmu_instance;
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    proto::Instance request;
    proto::Empty reply;
    grpc::ClientContext context;
    grpc::Status status = client->terminate(&context, request, &reply);
    if (!status.ok()) {
        std::cerr << status.error_message() << std::endl;
        return fmi3Error;
    }

    return fmi3OK;
}

fmi3Status fmi3SetDebugLogging(
    fmi3Instance instance,
    fmi3Boolean loggingOn,
    size_t nCategories,
    const fmi3String categories[]) {
    
    // Missing body
    return fmi3OK;
}

}


