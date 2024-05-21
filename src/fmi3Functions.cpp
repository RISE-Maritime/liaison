#include <stdexcept>
#include "zenoh.hxx"
#include "zenohc.hxx"
#include "fmi3.pb.h"
#include "fmi3Functions.h"

#define TRY_CODE(FUNCTION_CALL, ERROR_MSG, ERROR_RETURN) \
    try {                                                          \
        FUNCTION_CALL                                             \
    } catch (const std::exception &e) {                            \
        std::cerr << ERROR_MSG << e.what() << std::endl;           \
        return ERROR_RETURN;                                       \
    } \

#define SET_INSTANCE(INPUT, INSTANCE) \
    Placeholder* placeholder = reinterpret_cast<Placeholder*>(INSTANCE); \
    INPUT.set_instance(placeholder->instance);  \

std::string responder_id = "demo";

// ZENOH 

std::unique_ptr<zenoh::Session> z_client;

// This macro makes a remote procedure call in the form of a query
// to Zenoh queryable. The argument fmi3Function is the name of the
// function to be called in the remote resource. An variable named 
// 'input' is the input to said function and should be an instance
// of a Protobuf message type.
// This macro requires that:
//  1.  A variable 'responder_id' of type std::string is declared
//      and defined following the key expression specification of 
//      Zenoh (e.g. std::string responder_id = "foo/bar").
//  1.  A variable 'input' of a certain Protobuf message type
//      is declared and defined (e.g. proto::fmi3InstanceMessage input; 
//      input.set_key(1);)
//  2. A variable 'output' of a certain Protobuf messate type 
//     is declared (e.g. proto::fmi3InstanceMessage output;).
#define ZENOH_FMI3_QUERY(fmi3Function) \
    size_t input_size = input.ByteSizeLong(); \
    std::vector<uint8_t> buffer(input_size); \
    input.SerializeToArray(buffer.data(), input_size); \
    std::string expr = "rpc/" + responder_id + "/" + fmi3Function; \
    auto [send, recv] = zenohc::reply_fifo_new(1);   \
    const char* params = ""; \
    zenoh::GetOptions options; \
    zenoh::Value value(buffer); \
    options.set_value(value); \
    z_client->get(          \
        expr,    \
        params,             \
        std::move(send),            \
        options);                        \
    zenoh::Reply reply(nullptr);  \
    recv(reply);  \
    if (reply.is_ok()) { \
        zenohc::Sample sample = zenohc::expect<zenoh::Sample>(reply.get()); \
        output.ParseFromArray(sample.payload.start, sample.payload.len); \
    } else {  \
        std::cerr << "Failed zenoh query for key expression: "<< expr << std::endl; \
    } \
    
// end of ZENOH

class Placeholder {
public:
    Placeholder(int id) {
        instance = id;
    }
    int instance;
};

fmi3Status extractFmi3Status(proto::fmi3StatusMessage status_message) {
    switch (status_message.status()) {
        case proto::OK: return fmi3OK;
        case proto::WARNING: return fmi3Warning;
        case proto::DISCARD: return fmi3Discard;
        case proto::ERROR: return fmi3Error;
        case proto::FATAL: return fmi3Fatal;
        default: throw std::invalid_argument("Invalid FMI3Status value");
    }
}

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
    // TODO: implement the usage of fmi3InstanceEnvironment, fmi3LogMessageCallback
    // and fmi3IntermediateUpdateCallback.
    
    // Start Zenoh Session
    zenoh::Config config;
    auto result = open(std::move(config));
    if (std::holds_alternative<zenoh::Session>(result)) {
        z_client = std::make_unique<zenoh::Session>(std::move(std::get<zenoh::Session>(result)));
        std::cout << "Zenoh Session started successfully." << std::endl;
    } else if (auto error = std::get_if<zenoh::ErrorMessage>(&result)) {
         throw std::runtime_error(std::string(error->as_string_view()));
    } else {
        std::cerr << "Error: Unknown." << std::endl;
    }
    
    proto::fmi3InstantiateCoSimulationMessage input;
    proto::fmi3InstanceMessage output;

    input.set_instance_name(instanceName);
    input.set_instantiation_token(instantiationToken);
    input.set_resource_path(resourcePath);
    input.set_visible(visible);
    input.set_logging_on(loggingOn);
    input.set_event_mode_used(eventModeUsed);
    input.set_early_return_allowed(earlyReturnAllowed);
    for (int i = 0; i < nRequiredIntermediateVariables; ++i) {
        input.add_required_intermediate_variables(requiredIntermediateVariables[i]); 
    }
    input.set_n_required_intermediate_variables(nRequiredIntermediateVariables);
    
    ZENOH_FMI3_QUERY("fmi3InstantiateCoSimulation")

    Placeholder* placeholder = new Placeholder(output.instance());
    return reinterpret_cast<fmi3Instance>(placeholder);
}


fmi3Status fmi3EnterInitializationMode(
    fmi3Instance instance,
    fmi3Boolean toleranceDefined,
    fmi3Float64 tolerance,
    fmi3Float64 startTime,
    fmi3Boolean stopTimeDefined,
    fmi3Float64 stopTime) {

    proto::fmi3EnterInitializationModeMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE(input, instance)
    input.set_tolerance_defined(toleranceDefined);
    input.set_tolerance(tolerance);
    input.set_start_time(startTime);
    input.set_stop_time_defined(stopTimeDefined);
    input.set_stop_time(stopTime);
    
    ZENOH_FMI3_QUERY("fmi3EnterInitializationMode")

    return extractFmi3Status(output);
}

fmi3Status fmi3DoStep(fmi3Instance instance,
    fmi3Float64 currentCommunicationPoint,
    fmi3Float64 communicationStepSize,
    fmi3Boolean noSetFMUStatePriorToCurrentPoint,
    fmi3Boolean* eventHandlingNeeded,
    fmi3Boolean* terminateSimulation,
    fmi3Boolean* earlyReturn,
    fmi3Float64* lastSuccessfulTime) {

    proto::fmi3DoStepMessage input;
    proto::fmi3StatusMessage output;
    
    SET_INSTANCE(input, instance) 
    input.set_current_communication_point(currentCommunicationPoint);
    input.set_communication_step_size(communicationStepSize);
    input.set_no_set_fmu_state_prior_to_current_point(noSetFMUStatePriorToCurrentPoint);
    input.set_event_handling_needed(*eventHandlingNeeded);
    input.set_terminate_simulation(*terminateSimulation);
    input.set_early_return(*earlyReturn);
    input.set_last_successful_time(*lastSuccessfulTime);

    ZENOH_FMI3_QUERY("fmi3DoStep")
    
    return extractFmi3Status(output);
}

fmi3Status fmi3GetFloat64(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64 values[],
    size_t nValues) {
    
    proto::fmi3GetFloat64Message input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE(input, instance) 
    for (int i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]); 
    }
    input.set_n_value_references(nValueReferences);
    for (int i = 0; i < nValues; ++i) {
        input.add_values(values[i]); 
    }
    input.set_n_values(nValues);
    
    ZENOH_FMI3_QUERY("fmi3GetFloat64")
    
    return extractFmi3Status(output);
}


fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;
    
    SET_INSTANCE(input, instance)

    ZENOH_FMI3_QUERY("fmi3ExitInitializationMode")

    return extractFmi3Status(output);
}


void fmi3FreeInstance(fmi3Instance instance) {

    proto::fmi3InstanceMessage input;
    proto::voidMessage output;

    SET_INSTANCE(input, instance)

    ZENOH_FMI3_QUERY("fmi3FreeInstance")

    delete placeholder;
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE(input, instance)
    
    ZENOH_FMI3_QUERY("fmi3Terminate")

    return extractFmi3Status(output);;
}

fmi3Status fmi3SetDebugLogging(
    fmi3Instance instance,
    fmi3Boolean loggingOn,
    size_t nCategories,
    const fmi3String categories[]) {

    proto::fmi3SetDebugLoggingMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE(input, instance)
    input.set_logging_on(loggingOn);
    input.set_n_categories(nCategories);
    for (int i = 0; i < nCategories; ++i) {
        input.add_categories(categories[i]); 
    }
    
    ZENOH_FMI3_QUERY("fmi3SetDebugLogging")

    return extractFmi3Status(output);
}

}

