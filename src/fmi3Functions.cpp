#include <stdexcept>
#include <iostream>
#include <fstream>
#include <string>
#include <regex>
#include <dlfcn.h>
#include <filesystem>
#include "zenoh.hxx"
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
    INPUT.set_instance_index(placeholder->instance_index);  \


// ZENOH 

std::unique_ptr<zenoh::Session> session;

#define QUERY(fmi3Function, input, output, responderId) \
    std::vector<uint8_t> input_wire(input.ByteSizeLong()); \
    input.SerializeToArray(input_wire.data(), input_wire.size()); \
    std::string expr = "rpc/" + responderId + "/" + fmi3Function; \
    zenoh::Session::GetOptions options; \
    options.target = zenoh::QueryTarget::Z_QUERY_TARGET_ALL; \
    options.payload = zenoh::Bytes(std::move(input_wire)); \
    auto replies = session->get(expr,"", zenoh::channels::FifoChannel(1), std::move(options)); \
    auto res = replies.recv(); \
    const auto &sample = std::get<zenoh::Reply>(res).get_ok(); \
    const auto& output_payload = sample.get_payload(); \
    std::vector<uint8_t> output_wire = output_payload.as_vector(); \
    output.ParseFromArray(output_wire.data(), output_wire.size()); \

// end of ZENOH

class Placeholder {
public:
    Placeholder(int index) {
        instance_index = index;
    }
    int instance_index;
};

std::string responderId;

fmi3Status transformToFmi3Status(proto::Status status) {
    switch (status) {
        case proto::OK: return fmi3OK;
        case proto::WARNING: return fmi3Warning;
        case proto::DISCARD: return fmi3Discard;
        case proto::ERROR: return fmi3Error;
        case proto::FATAL: return fmi3Fatal;
        default: throw std::invalid_argument("Invalid status value");
    }
}

std::string getBaseDirectory() {
    Dl_info dl_info;
    dladdr((void*)getBaseDirectory, &dl_info);
    std::filesystem::path libraryPath(dl_info.dli_fname);
    return libraryPath.parent_path().parent_path().string(); 
}

void printDirectoryContents(const std::string& directoryPath) {
    try {
        // Check if the given path exists and is a directory
        if (!std::filesystem::exists(directoryPath)) {
            throw std::runtime_error("Directory does not exist: " + directoryPath);
        }

        if (!std::filesystem::is_directory(directoryPath)) {
            throw std::runtime_error("Path is not a directory: " + directoryPath);
        }

        // Iterate through the directory
        for (const auto& entry : std::filesystem::directory_iterator(directoryPath)) {
            // Print the path of the current entry
            std::cout << entry.path().string() << std::endl;
        }
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
}

void readResponderId() {
    
    std::string baseDirectory = getBaseDirectory();
    std::string responderIdPath = baseDirectory + "/responderId";
    std::ifstream file(responderIdPath);
    if (!file.is_open()) {
       printDirectoryContents(baseDirectory);
       throw std::runtime_error("Failed to open responderId file at: " + responderIdPath);
    }

    std::string line;
    std::regex regexPattern(R"(responderId='([^']+)')");
    std::smatch match;

    while (std::getline(file, line)) {
        if (std::regex_search(line, match, regexPattern) && match.size() > 1) {
            responderId = match.str(1);
            return;
        }
    }

    throw std::runtime_error("responderId not found in file: " + responderIdPath);
    
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

    // Read the responder id
    readResponderId();
    
    // Start Zenoh Session
    try {
        zenoh::Config config = zenoh::Config::create_default();
        session = std::make_unique<zenoh::Session>(zenoh::Session::open(std::move(config)));
        std::cout << "Zenoh Session started successfully." << std::endl;
    } catch (const std::exception &e) {
        std::cerr << "Failed to start Zenoh Session: " << e.what() << std::endl;
        throw; // Re-throw the exception to propagate the error
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
    
    QUERY("fmi3InstantiateCoSimulation", input, output, responderId)

    std::cout << "Instance: " << output.instance_index() << std::endl;

    Placeholder* placeholder = new Placeholder(output.instance_index());
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
    
    QUERY("fmi3EnterInitializationMode", input, output, responderId)

    return transformToFmi3Status(output.status());
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

    QUERY("fmi3DoStep", input, output, responderId)
    
    return transformToFmi3Status(output.status());
}

fmi3Status fmi3GetFloat64(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64 values[],
    size_t nValues) {
    
    proto::fmi3GetFloat64InputMessage input;
    proto::fmi3GetFloat64OutputMessage output;

    SET_INSTANCE(input, instance) 
    for (int i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]); 
    }
    input.set_n_value_references(nValueReferences);
    
    QUERY("fmi3GetFloat64", input, output, responderId)

    for (int i = 0; i < output.n_values(); ++i) {
        values[i] = output.values()[i]; 
    }
    nValues = output.n_values();
    
    return transformToFmi3Status(output.status());
}


fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;
    
    SET_INSTANCE(input, instance)

    QUERY("fmi3ExitInitializationMode", input, output, responderId)

    return transformToFmi3Status(output.status());
}


void fmi3FreeInstance(fmi3Instance instance) {

    proto::fmi3InstanceMessage input;
    proto::voidMessage output;

    SET_INSTANCE(input, instance)

    QUERY("fmi3FreeInstance", input, output, responderId)

    delete placeholder;

    session.reset();
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE(input, instance)
    
    QUERY("fmi3Terminate", input, output, responderId)

    return transformToFmi3Status(output.status());;
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
    
    QUERY("fmi3SetDebugLogging", input, output, responderId)

    return transformToFmi3Status(output.status());
}

}

