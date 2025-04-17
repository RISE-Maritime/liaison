#ifdef _WIN32
#include <windows.h>
#undef ERROR
#else
#include <dlfcn.h>
#include <unistd.h>
#endif
#include <stdexcept>
#include <iostream>
#include <fstream>
#include <string>
#include <regex>
#include <thread>
#include <chrono>
#include <filesystem>
#include "zenoh.hxx"
#include "fmi3.pb.h"
#include "fmi3Functions.h"

#include <nlohmann/json.hpp>
using json = nlohmann::json;


// MACROS


#define SET_INSTANCE_REFERENCE(INPUT, INSTANCE) \
    auto placeholder = reinterpret_cast<Placeholder*>(INSTANCE); \
    INPUT.set_instance_index(placeholder->instance_index);  \

#define NOT_IMPLEMENTED \
    return fmi3Error; \

#define DEFINE_FMI3_SET_VALUE_FUNCTION(TYPE) \
fmi3Status fmi3Set##TYPE( \
    fmi3Instance instance, \
    const fmi3ValueReference valueReferences[], \
    size_t nValueReferences, \
    const fmi3##TYPE values[], \
    size_t nValues) { \
    proto::fmi3Set##TYPE##InputMessage input; \
    proto::fmi3StatusMessage output; \
    SET_INSTANCE_REFERENCE(input, instance); \
    for (size_t i = 0; i < nValueReferences; ++i) { \
        input.add_value_references(valueReferences[i]); \
    } \
    input.set_n_value_references(nValueReferences); \
    for (size_t i = 0; i < nValues; ++i) { \
        input.add_values(values[i]); \
    } \
    input.set_n_values(nValues); \
    QUERY("fmi3Set"#TYPE, input, output); \
    return transformToFmi3Status(output.status()); \
}

#define DEFINE_FMI3_GET_VALUE_FUNCTION(TYPE) \
fmi3Status fmi3Get##TYPE( \
    fmi3Instance instance, \
    const fmi3ValueReference valueReferences[], \
    size_t nValueReferences, \
    fmi3##TYPE values[], \
    size_t nValues) { \
    proto::fmi3Get##TYPE##InputMessage input; \
    proto::fmi3Get##TYPE##OutputMessage output; \
    SET_INSTANCE_REFERENCE(input, instance); \
    for (size_t i = 0; i < nValueReferences; ++i) { \
        input.add_value_references(valueReferences[i]); \
    } \
    input.set_n_value_references(nValueReferences); \
    QUERY("fmi3Get"#TYPE, input, output); \
    for (size_t i = 0; i < output.values_size(); ++i) { \
        values[i] = output.values(i); \
    } \
    nValues = output.values_size(); \
    return transformToFmi3Status(output.status()); \
}

#define BASE_QUERY(fmi3Function, input, output, errorReturnValue) \
    std::vector<uint8_t> input_wire(input.ByteSizeLong()); \
    input.SerializeToArray(input_wire.data(), input_wire.size()); \
    std::string expr = "rpc/" + placeholder->responderId + "/" + fmi3Function; \
    zenoh::Session::GetOptions options; \
    options.target = zenoh::QueryTarget::Z_QUERY_TARGET_ALL; \
    options.payload = zenoh::Bytes(std::move(input_wire)); \
    auto replies = placeholder->session->get(expr,"", zenoh::channels::FifoChannel(1), std::move(options)); \
    auto res = replies.recv(); \
    if (std::holds_alternative<zenoh::channels::RecvError>(res)) { \
        if (std::get<zenoh::channels::RecvError>(res) == zenoh::channels::RecvError::Z_DISCONNECTED) { \
            std::string error_msg = "Exception in " + std::string(fmi3Function) + ": '" + expr + "' is disconnected."; \
            placeholder->logMessage(placeholder->instanceEnvironment, fmi3Error, "Zenoh", error_msg.c_str()); \
        } else if (std::get<zenoh::channels::RecvError>(res) == zenoh::channels::RecvError::Z_NODATA) { \
            std::string error_msg = "Exception in " + std::string(fmi3Function) + ": No data received from '" + expr + "'."; \
            placeholder->logMessage(placeholder->instanceEnvironment, fmi3Error, "Zenoh", error_msg.c_str()); \
        } \
        return errorReturnValue; \
    } \
    const auto &sample = std::get<zenoh::Reply>(res).get_ok(); \
    const auto& output_payload = sample.get_payload(); \
    std::vector<uint8_t> output_wire = output_payload.as_vector(); \
    output.ParseFromArray(output_wire.data(), output_wire.size()); \


#define QUERY(fmi3Function, input, output) \
    BASE_QUERY(fmi3Function, input, output, fmi3Fatal) \

#define QUERY_INSTANCE(fmi3Function, input, output) \
    BASE_QUERY(fmi3Function, input, output, nullptr) \

#define QUERY_VOID(fmi3Function, input, output) \
    BASE_QUERY(fmi3Function, input, output,) \

// end of MACROS


fmi3Status transformToFmi3Status(proto::Status status) {
    switch (status) {
        case proto::OK: return fmi3OK;
        case proto::WARNING: return fmi3Warning;
        case proto::DISCARD: return fmi3Discard;
        case proto::ERROR: return fmi3Error;
        case proto::FATAL: return fmi3Fatal;
        default: return fmi3Fatal; 
    }
}

#ifdef _WIN32
std::string getBaseDirectory() {
  char path[MAX_PATH] = {0};
    HMODULE hModule = NULL;

    // Retrieve the handle for the DLL containing this function
    if (!GetModuleHandleEx(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | 
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, 
            reinterpret_cast<LPCSTR>(&getBaseDirectory), 
            &hModule)) {
        throw std::runtime_error("Failed to retrieve the shared library handle.");
    }

    // Retrieve the full path of the DLL
    if (GetModuleFileName(hModule, path, MAX_PATH) == 0) {
        throw std::runtime_error("Failed to retrieve shared library path.");
    }

    // Convert the path to a filesystem path and get the grandparent directory
    std::filesystem::path libraryPath(path);
    return libraryPath.parent_path().parent_path().string();
}
#else
std::string getBaseDirectory() {
    Dl_info dl_info;

    // Retreive the shared library path
    if (dladdr((void*)getBaseDirectory, &dl_info)) {
        std::filesystem::path libraryPath(dl_info.dli_fname);
        return libraryPath.parent_path().parent_path().string(); 
    }
    // Error handling
    throw std::runtime_error("Failed to retrieve shared library path");
}
#endif

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



class Placeholder {
public:
    Placeholder(fmi3InstanceEnvironment instanceEnvironment, fmi3LogMessageCallback logMessage) 
        : instanceEnvironment(instanceEnvironment)
        , logMessage(logMessage) {
            StartSession();
            addLogMessageSubscriber(instanceEnvironment, logMessage);
        }
                
    int instance_index;
    fmi3InstanceEnvironment instanceEnvironment;
    fmi3LogMessageCallback logMessage;
    std::unique_ptr<zenoh::Session> session;
    std::string responderId;


    void StartSession() {
        if (session) {
            return;
        }
        try {
            std::string baseDirectory = getBaseDirectory();
            std::string configFilePath = baseDirectory + "/config.json";
            if (!std::filesystem::exists(configFilePath)) {
                printDirectoryContents(baseDirectory);
            throw std::runtime_error("Failed to open config file at: " + configFilePath);
            }
            json config;
            config = json::parse(std::ifstream(configFilePath));
            responderId = config["responderId"];
            std::string zenohConfigString;
            if (config.contains("zenohConfig")) {
                json& zenohConfig = config["zenohConfig"];
                if (zenohConfig.contains("transport") && 
                zenohConfig["transport"].contains("link") && 
                zenohConfig["transport"]["link"].contains("tls")) {
                    json& tls = zenohConfig["transport"]["link"]["tls"];
                    if (tls.contains("connect_certificate")) {
                        tls["connect_certificate"] = baseDirectory + "/" + tls["connect_certificate"].get<std::string>();
                    }
                    if (tls.contains("connect_private_key")) {
                        tls["connect_private_key"] = baseDirectory + "/" + tls["connect_private_key"].get<std::string>();
                    }
                    if (tls.contains("root_ca_certificate")) {
                        tls["root_ca_certificate"] = baseDirectory + "/" + tls["root_ca_certificate"].get<std::string>();
                    }
                } 
                zenohConfigString = zenohConfig.dump();
            }
            zenoh::Config zenohConfig = !zenohConfigString.empty() ? 
                zenoh::Config::from_str(zenohConfigString) : 
                zenoh::Config::create_default();
            session = std::make_unique<zenoh::Session>(zenoh::Session::open(std::move(zenohConfig))); 
        } catch (const zenoh::ZException& e) {
            throw std::runtime_error(e.what());
        }
    }

    void addLogMessageSubscriber(fmi3InstanceEnvironment instanceEnvironment, fmi3LogMessageCallback logMessage) {
        auto logMessageCallback = [logMessage, instanceEnvironment](const zenoh::Sample& sample) { 
            proto::logMessage log_message; 
            const auto wire = sample.get_payload().as_vector(); 
            log_message.ParseFromArray(wire.data(), wire.size()); 
            logMessage( 
                instanceEnvironment, 
                transformToFmi3Status(log_message.status()), 
                log_message.category().c_str(), 
                log_message.message().c_str() 
            ); 
        }; 
        auto dropCallback = []() { 
        };
        std::string expr_fmi3LogMessage = "rpc/" + responderId + "/fmi3LogMessage"; 
        zenoh::KeyExpr keyexpr_fmi3LogMessage(expr_fmi3LogMessage); 
        auto subscriber = session->declare_subscriber(keyexpr_fmi3LogMessage, logMessageCallback, dropCallback); 
    }

    void SetInstanceIndex(int index) {
        instance_index = index;
    }

    ~Placeholder() {
        if (session) {
            session->close();
        }
    }

};


/***************************************************
 
                FMI3 Functions

****************************************************/

extern "C" {

/***************************************************
Types for Common Functions
****************************************************/    

/* Inquire version numbers and set debug logging */

const char* fmi3GetVersion() {
    return "3.0";
}


fmi3Status fmi3SetDebugLogging(
    fmi3Instance instance,
    fmi3Boolean loggingOn,
    size_t nCategories,
    const fmi3String categories[]) {

    proto::fmi3SetDebugLoggingMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE_REFERENCE(input, instance)
    input.set_logging_on(loggingOn);
    input.set_n_categories(nCategories);
    for (int i = 0; i < nCategories; ++i) {
        input.add_categories(categories[i]); 
    }
    
    QUERY("fmi3SetDebugLogging", input, output)

    return transformToFmi3Status(output.status());
}


/* Creation and destruction of FMU instances */

fmi3Instance fmi3InstantiateModelExchange(
    fmi3String                     instanceName,
    fmi3String                     instantiationToken,
    fmi3String                     resourcePath,
    fmi3Boolean                    visible,
    fmi3Boolean                    loggingOn,
    fmi3InstanceEnvironment        instanceEnvironment,
    fmi3LogMessageCallback         logMessage) {
    

    Placeholder* placeholder;
    try {
        placeholder = new Placeholder(instanceEnvironment, logMessage);
    } catch (const std::exception& e) {
        logMessage(instanceEnvironment, fmi3Fatal, "Zenoh", e.what());
        return nullptr;
    }
   

    proto::fmi3InstantiateModelExchangeMessage input;
    proto::fmi3InstanceMessage output;

    input.set_instance_name(instanceName);
    input.set_instantiation_token(instantiationToken);
    input.set_resource_path(resourcePath);
    input.set_visible(visible);
    input.set_logging_on(loggingOn);
    
    QUERY_INSTANCE("fmi3InstantiateModelExchange", input, output)

    placeholder->SetInstanceIndex(output.instance_index());
    return reinterpret_cast<fmi3Instance>(placeholder);
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
    // TODO: implement the usage of fmi3InstanceEnvironment,
    // and fmi3IntermediateUpdateCallback.
    
    
    Placeholder* placeholder;
    try {
        placeholder = new Placeholder(instanceEnvironment, logMessage);
    } catch (const std::exception& e) {
        logMessage(instanceEnvironment, fmi3Fatal, "Zenoh", e.what());
        return nullptr;
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
    
  
    QUERY_INSTANCE("fmi3InstantiateCoSimulation", input, output)
   

    placeholder->SetInstanceIndex(output.instance_index());
    return reinterpret_cast<fmi3Instance>(placeholder);
}


fmi3Instance fmi3InstantiateScheduledExecution(
    fmi3String                     instanceName,
    fmi3String                     instantiationToken,
    fmi3String                     resourcePath,
    fmi3Boolean                    visible,
    fmi3Boolean                    loggingOn,
    fmi3InstanceEnvironment        instanceEnvironment,
    fmi3LogMessageCallback         logMessage,
    fmi3ClockUpdateCallback        clockUpdate,
    fmi3LockPreemptionCallback     lockPreemption,
    fmi3UnlockPreemptionCallback   unlockPreemption) {

    
    Placeholder* placeholder;
    try {
        placeholder = new Placeholder(instanceEnvironment, logMessage);
    } catch (const std::exception& e) {
        logMessage(instanceEnvironment, fmi3Fatal, "Zenoh", e.what());
        return nullptr;
    }
    
    proto::fmi3InstantiateScheduledExecutionMessage input;
    proto::fmi3InstanceMessage output;

    input.set_instance_name(instanceName);
    input.set_instantiation_token(instantiationToken);
    input.set_resource_path(resourcePath);
    input.set_visible(visible);
    input.set_logging_on(loggingOn);
    // TODO: implement functionality for instanceEnvironment, clockUpdate, lockPeemption, and unlockPreemption

       
    QUERY_INSTANCE("fmi3InstantiateScheduledExecution", input, output)
 

    placeholder->SetInstanceIndex(output.instance_index());
    return reinterpret_cast<fmi3Instance>(placeholder);
}

void fmi3FreeInstance(fmi3Instance instance) {
    if (!instance) {
        return;
    }

    // Free the instance
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;
    SET_INSTANCE_REFERENCE(input, instance)
    QUERY_VOID("fmi3FreeInstance", input, output)

    // Clean up the placeholder (created by SET_INSTANCE_REFERENCE)
    delete placeholder;
    
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

    SET_INSTANCE_REFERENCE(input, instance)
    input.set_tolerance_defined(toleranceDefined);
    input.set_tolerance(tolerance);
    input.set_start_time(startTime);
    input.set_stop_time_defined(stopTimeDefined);
    input.set_stop_time(stopTime);
    
    QUERY("fmi3EnterInitializationMode", input, output)

    return transformToFmi3Status(output.status());
}

fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;
    
    SET_INSTANCE_REFERENCE(input, instance)

    QUERY("fmi3ExitInitializationMode", input, output)

    return transformToFmi3Status(output.status());
}


fmi3Status fmi3EnterEventMode(fmi3Instance instance) {
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE_REFERENCE(input, instance)

    QUERY("fmi3EnterEventMode", input, output)

    return transformToFmi3Status(output.status());
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE_REFERENCE(input, instance)
    
    QUERY("fmi3Terminate", input, output)

    return transformToFmi3Status(output.status());;
}

fmi3Status fmi3Reset(fmi3Instance instance) {
    proto::fmi3InstanceMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE_REFERENCE(input, instance)
    
    QUERY("fmi3Reset", input, output)

    return transformToFmi3Status(output.status());
}

/* Getting and setting variable values */


DEFINE_FMI3_GET_VALUE_FUNCTION(Float32)
DEFINE_FMI3_SET_VALUE_FUNCTION(Float32)

DEFINE_FMI3_GET_VALUE_FUNCTION(Float64)
DEFINE_FMI3_SET_VALUE_FUNCTION(Float64)

DEFINE_FMI3_GET_VALUE_FUNCTION(Int8)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int8)

DEFINE_FMI3_GET_VALUE_FUNCTION(UInt8)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt8)

DEFINE_FMI3_GET_VALUE_FUNCTION(Int16)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int16)

DEFINE_FMI3_GET_VALUE_FUNCTION(UInt16)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt16)

DEFINE_FMI3_GET_VALUE_FUNCTION(Int32)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int32)

DEFINE_FMI3_GET_VALUE_FUNCTION(UInt32)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt32)

DEFINE_FMI3_GET_VALUE_FUNCTION(Int64)
DEFINE_FMI3_SET_VALUE_FUNCTION(Int64)

DEFINE_FMI3_GET_VALUE_FUNCTION(UInt64)
DEFINE_FMI3_SET_VALUE_FUNCTION(UInt64)

DEFINE_FMI3_GET_VALUE_FUNCTION(Boolean)
DEFINE_FMI3_SET_VALUE_FUNCTION(Boolean)

DEFINE_FMI3_SET_VALUE_FUNCTION(String)
fmi3Status fmi3GetString(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3String values[],
    size_t nValues) {
    
    proto::fmi3GetStringInputMessage input;
    proto::fmi3GetStringOutputMessage output;

    SET_INSTANCE_REFERENCE(input, instance) 
    for (size_t i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]); 
    }
    input.set_n_value_references(nValueReferences);
    
    QUERY("fmi3GetString", input, output)

    for (int i = 0; i < output.n_values(); ++i) {
        values[i] = output.values(i).c_str(); 
    }
    nValues = output.n_values();
    
    return transformToFmi3Status(output.status());
}

fmi3Status fmi3SetBinary(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const size_t valueSizes[],
    const fmi3Binary values[],
    size_t nValues) {
    
    proto::fmi3SetBinaryInputMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE_REFERENCE(input, instance) 
    
    for (size_t i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]);
    }
    input.set_n_value_references(nValueReferences);

    size_t offset = 0;
    for (size_t i = 0; i < nValues; ++i) {
        std::string binaryValue(reinterpret_cast<const char*>(values + offset), valueSizes[i]);
        input.add_values(binaryValue);
        offset += valueSizes[i];
    }
    input.set_n_values(nValues);

    QUERY("fmi3SetBinary", input, output)

    return transformToFmi3Status(output.status());
}

fmi3Status fmi3GetBinary(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    size_t valueSizes[],
    fmi3Binary values[],
    size_t nValues) {

    proto::fmi3GetBinaryInputMessage input;
    proto::fmi3GetBinaryOutputMessage output;

    SET_INSTANCE_REFERENCE(input, instance) 
    
    for (size_t i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]); 
    }
    input.set_n_value_references(nValueReferences);
    
    QUERY("fmi3GetBinary", input, output)

    size_t offset = 0;
    for (size_t i = 0; i < output.n_values(); ++i) {
        const std::string& binaryValue = output.values(i);
        size_t binarySize = binaryValue.size();
        valueSizes[i] = binarySize;
        std::memcpy(values + offset, binaryValue.data(), binarySize);
        offset += binarySize;
    }
   
    return transformToFmi3Status(output.status());
}

fmi3Status fmi3SetClock(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const fmi3Clock values[]) {
    
    proto::fmi3SetClockInputMessage input;
    proto::fmi3StatusMessage output;

    SET_INSTANCE_REFERENCE(input, instance) 
    
    for (size_t i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]);
        input.add_values(values[i]); 
    }
    input.set_n_value_references(nValueReferences);
        
    QUERY("fmi3SetClock", input, output)

    return transformToFmi3Status(output.status());
}

fmi3Status fmi3GetClock(
    fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Clock values[]) {

    proto::fmi3GetClockInputMessage input;
    proto::fmi3GetClockOutputMessage output;

    SET_INSTANCE_REFERENCE(input, instance) 
    
    for (size_t i = 0; i < nValueReferences; ++i) {
        input.add_value_references(valueReferences[i]); 
    }
    input.set_n_value_references(nValueReferences);
    
    QUERY("fmi3GetClock", input, output)

    for (int i = 0; i < output.n_values(); ++i) {
        values[i] = output.values(i); 
    }
   
    return transformToFmi3Status(output.status());
}

/* Getting Variable Dependency Information */

fmi3Status fmi3GetNumberOfVariableDependencies(
    fmi3Instance instance,
    fmi3ValueReference valueReference,
    size_t* nDependencies) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetVariableDependencies(
    fmi3Instance instance,
    fmi3ValueReference dependent,
    size_t elementIndicesOfDependent[],
    fmi3ValueReference independents[],
    size_t elementIndicesOfIndependents[],
    fmi3DependencyKind dependencyKinds[],
    size_t nDependencies) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetFMUState(fmi3Instance instance, fmi3FMUState* state) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SetFMUState(fmi3Instance instance, fmi3FMUState state) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3FreeFMUState(fmi3Instance instance, fmi3FMUState* state) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SerializedFMUStateSize(fmi3Instance instance, fmi3FMUState state, size_t* size) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SerializeFMUState(fmi3Instance instance, fmi3FMUState state, fmi3Byte serializedState[], size_t size) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3DeserializeFMUState(fmi3Instance instance, const fmi3Byte serializedState[], size_t size, fmi3FMUState* state) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetDirectionalDerivative(
    fmi3Instance instance,
    const fmi3ValueReference unknowns[],
    size_t nUnknowns,
    const fmi3ValueReference knowns[],
    size_t nKnowns,
    const fmi3Float64 seed[],
    size_t nSeed,
    fmi3Float64 sensitivity[],
    size_t nSensitivity) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetAdjointDerivative(
    fmi3Instance instance,
    const fmi3ValueReference unknowns[],
    size_t nUnknowns,
    const fmi3ValueReference knowns[],
    size_t nKnowns,
    const fmi3Float64 seed[],
    size_t nSeed,
    fmi3Float64 sensitivity[],
    size_t nSensitivity) {
    NOT_IMPLEMENTED
}

/* Entering and exiting the Configuration or Reconfiguration Mode */

fmi3Status fmi3EnterConfigurationMode(fmi3Instance instance) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3ExitConfigurationMode(fmi3Instance instance) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetIntervalDecimal(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64 intervals[],
    fmi3IntervalQualifier qualifiers[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetIntervalFraction(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt64 counters[],
    fmi3UInt64 resolutions[],
    fmi3IntervalQualifier qualifiers[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetShiftDecimal(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3Float64 shifts[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetShiftFraction(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    fmi3UInt64 counters[],
    fmi3UInt64 resolutions[]) {
    NOT_IMPLEMENTED
}   

fmi3Status fmi3SetIntervalDecimal(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const fmi3Float64 intervals[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SetIntervalFraction(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const fmi3UInt64 counters[],
    const fmi3UInt64 resolutions[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SetShiftDecimal(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const fmi3Float64 shifts[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SetShiftFraction(fmi3Instance instance,
    const fmi3ValueReference valueReferences[],
    size_t nValueReferences,
    const fmi3UInt64 counters[],
    const fmi3UInt64 resolutions[]) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3EvaluateDiscreteStates(fmi3Instance instance) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3UpdateDiscreteStates(
    fmi3Instance instance,
    fmi3Boolean* discreteStatesNeedUpdate,
    fmi3Boolean* terminateSimulation,
    fmi3Boolean* nominalsOfContinuousStatesChanged,
    fmi3Boolean* valuesOfContinuousStatesChanged,
    fmi3Boolean* nextEventTimeDefined,
    fmi3Float64* nextEventTime) {
    NOT_IMPLEMENTED
}

/***************************************************
Types for Functions for Model Exchange
****************************************************/

fmi3Status fmi3EnterContinuousTimeMode(fmi3Instance instance) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3CompletedIntegratorStep(fmi3Instance instance,
    fmi3Boolean  noSetFMUStatePriorToCurrentPoint,
    fmi3Boolean* enterEventMode,
    fmi3Boolean* terminateSimulation) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SetTime(fmi3Instance instance, fmi3Float64 time) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3SetContinuousStates(fmi3Instance instance, const fmi3Float64 x[], size_t nx) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetContinuousStateDerivatives(fmi3Instance instance, fmi3Float64 derivatives[], size_t nx) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetEventIndicators(fmi3Instance instance, fmi3Float64 eventIndicators[], size_t ni) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetContinuousStates(fmi3Instance instance, fmi3Float64 x[], size_t nx) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetNominalsOfContinuousStates(fmi3Instance instance, fmi3Float64 x_nominal[], size_t nx) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetNumberOfEventIndicators(fmi3Instance instance, size_t* nz) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetNumberOfContinuousStates(fmi3Instance instance, size_t* nx) {
    NOT_IMPLEMENTED
}

/***************************************************
Types for Functions for Co-Simulation
****************************************************/

fmi3Status fmi3EnterStepMode(fmi3Instance instance) {
    NOT_IMPLEMENTED
}

fmi3Status fmi3GetOutputDerivatives(fmi3Instance instance, const fmi3ValueReference valueReferences[], size_t nValueReferences, const fmi3Int32 orders[], fmi3Float64 values[], size_t nValues) {
    NOT_IMPLEMENTED
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
    
    SET_INSTANCE_REFERENCE(input, instance) 
    input.set_current_communication_point(currentCommunicationPoint);
    input.set_communication_step_size(communicationStepSize);
    input.set_no_set_fmu_state_prior_to_current_point(noSetFMUStatePriorToCurrentPoint);
    input.set_event_handling_needed(*eventHandlingNeeded);
    input.set_terminate_simulation(*terminateSimulation);
    input.set_early_return(*earlyReturn);
    input.set_last_successful_time(*lastSuccessfulTime);

    QUERY("fmi3DoStep", input, output)
    
    return transformToFmi3Status(output.status());
}


/***************************************************
Types for Functions for Scheduled Execution
****************************************************/

fmi3Status fmi3ActivateModelPartition(fmi3Instance instance, fmi3ValueReference clockReference, fmi3Float64 activationTime) {
    NOT_IMPLEMENTED
}


} // end of "extern C"

