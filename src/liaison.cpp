#ifdef _WIN32
#include <windows.h>
#undef ERROR
#else
#include <dlfcn.h>
#include <dirent.h>
#include <unistd.h>
#endif
#include <vector>
#include <sys/stat.h>
#include <unordered_map>
#include <filesystem>
#include <sstream>
#include <zip.h>
#include <iostream>
#include <memory>

#include "zenoh.hxx"
#include "fmi3.pb.h"
#include "fmi3Functions.h"
#include <spdlog/spdlog.h>
#include "utils.hpp"

#include <nlohmann/json.hpp>
using json = nlohmann::json;

// MACROS

#define MAX_BINARY_SIZE 4096


#define DECLARE_QUERYABLE(FMI3FUNCTION, RESPONDER_ID) \
    std::string expr_##FMI3FUNCTION = "rpc/" + RESPONDER_ID + "/" + std::string(#FMI3FUNCTION); \
    zenoh::KeyExpr keyexpr_##FMI3FUNCTION(expr_##FMI3FUNCTION); \
    auto on_drop_queryable_##FMI3FUNCTION = []() { spdlog::debug("Destroying queryable for {}",#FMI3FUNCTION); }; \
    auto queryable_##FMI3FUNCTION = session->declare_queryable(keyexpr_##FMI3FUNCTION, std::function<void(const zenoh::Query&)>(callbacks::FMI3FUNCTION), on_drop_queryable_##FMI3FUNCTION); \

#define PARSE_QUERY(QUERY, INPUT) \
    auto input_payload = QUERY.get_payload(); \
    if (input_payload.has_value()) { \
        const auto input_wire =  input_payload->get().as_vector(); \
        INPUT.ParseFromArray(input_wire.data(), input_wire.size()); \
    } \

#define SERIALIZE_REPLY(QUERY, OUTPUT) \
    std::vector<uint8_t> output_wire(OUTPUT.ByteSizeLong()); \
    OUTPUT.SerializeToArray(output_wire.data(), output_wire.size()); \
    auto output_payload = zenoh::Bytes(std::move(output_wire)); \
    QUERY.reply(QUERY.get_keyexpr(), std::move(output_payload)); \

// Platform-specific loading/unloading of libraries and symbol resolution
#ifdef _WIN32
#define BIND_FMU_LIBRARY_FUNCTION(FMI3FUNCTION) \
    fmu::FMI3FUNCTION = (FMI3FUNCTION##TYPE*)GetProcAddress(fmuLibrary, #FMI3FUNCTION);
#else
#define BIND_FMU_LIBRARY_FUNCTION(FMI3FUNCTION) \
    fmu::FMI3FUNCTION = (FMI3FUNCTION##TYPE*)dlsym(fmuLibrary, #FMI3FUNCTION); \
    if (!fmu::FMI3FUNCTION) { \
        std::ostringstream oss; \
        oss << "Unable to load function " << #FMI3FUNCTION << ": " << dlerror(); \
        throw std::runtime_error(oss.str()); \
    }
#endif


#define DEFINE_FMI3_GET_VALUE_FUNCTION(TYPE) \
void fmi3Get##TYPE(const zenoh::Query& query) { \
    printQuery(query); \
\
    proto::fmi3Get##TYPE##InputMessage input; \
    PARSE_QUERY(query, input) \
\
    fmi3ValueReference* value_references = new fmi3ValueReference[input.n_value_references()]; \
    for (int i = 0; i < input.n_value_references(); i++) { \
        value_references[i] = input.value_references()[i]; \
    } \
    fmi3##TYPE* values = new fmi3##TYPE[input.n_value_references()]; \
    size_t nValues = input.n_value_references(); \
\
    fmi3Status status = fmu::fmi3Get##TYPE( \
        getInstance(input.instance_index()), \
        value_references, \
        input.n_value_references(), \
        values, \
        nValues \
    ); \
\
    proto::fmi3Get##TYPE##OutputMessage output; \
    for (int i = 0; i < input.n_value_references(); i++) { \
        output.add_values(values[i]); \
    } \
    output.set_n_values(nValues); \
    output.set_status(transformToProtoStatus(status)); \
\
    SERIALIZE_REPLY(query, output) \
}

#define DEFINE_FMI3_SET_VALUE_FUNCTION(TYPE) \
void fmi3Set##TYPE(const zenoh::Query& query) { \
    printQuery(query); \
\
    proto::fmi3Set##TYPE##InputMessage input; \
    PARSE_QUERY(query, input); \
\
    fmi3ValueReference* value_references = new fmi3ValueReference[input.n_value_references()]; \
    for (int i = 0; i < input.n_value_references(); i++) { \
        value_references[i] = input.value_references()[i]; \
    } \
    fmi3##TYPE* values = new fmi3##TYPE[input.n_value_references()]; \
    for (int i = 0; i < input.n_value_references(); i++) { \
        values[i] = input.values()[i]; \
    } \
\
    fmi3Status status = fmu::fmi3Set##TYPE( \
        getInstance(input.instance_index()), \
        value_references, \
        input.n_value_references(), \
        values, \
        input.n_values() \
    ); \
\
    proto::fmi3StatusMessage output = makeFmi3StatusMessage(status); \
    SERIALIZE_REPLY(query, output) \
}

// end of MACROS

std::shared_ptr<fmi3String> resourcePath;
std::shared_ptr<zenoh::Session> session;
std::shared_ptr<zenoh::Publisher> fmi3LogMessagePublisher;

// Function to load and unload FMU library (platform-specific)
#ifdef _WIN32
HMODULE loadFmuLibrary(const std::string& libPath) {
    HMODULE handle = LoadLibrary(libPath.c_str());
    if (!handle) {
        DWORD errorCode = GetLastError();
        LPVOID errorMsg;
        FormatMessage(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            errorCode,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
            (LPSTR)&errorMsg,
            0,
            NULL
        );
        std::ostringstream oss;
        oss << "Failed to load FMU library '" << libPath << "': " << (LPSTR)errorMsg;
        LocalFree(errorMsg);
        throw std::runtime_error(oss.str());
    }
    return handle;
}

void unloadFmuLibrary(HMODULE handle) {
    FreeLibrary(handle);
}
#else
void* loadFmuLibrary(const std::string& libPath) {
    void* handle = dlopen(libPath.c_str(), RTLD_LAZY);
    if (!handle) {
        std::ostringstream oss;
        oss << "Failed to load FMU library '" << libPath << "': " << dlerror();
        throw std::runtime_error(oss.str());
    }
    return handle;
}

void unloadFmuLibrary(void* handle) {
    dlclose(handle);
}
#endif


// Map that holds the FMU instances
std::unordered_map<int, fmi3Instance> instances;
int nextIndex = 0;


const fmi3ValueReference* convertRepeatedFieldToCArray(const google::protobuf::RepeatedField<int>& repeatedField) {
    size_t size = repeatedField.size();
    fmi3ValueReference* cArray = new fmi3ValueReference[size];
    for (size_t i = 0; i < size; ++i) {
        cArray[i] = static_cast<fmi3ValueReference>(repeatedField.Get(i));
    }
    return cArray;
}

const char** convertRepeatedFieldToCArray(const google::protobuf::RepeatedPtrField<std::string>& repeatedField) {
    size_t size = repeatedField.size();
    char** cArray = new char*[size];

    for (size_t i = 0; i < size; ++i) {
        cArray[i] = new char[repeatedField.Get(i).size() + 1];  
        std::strcpy(cArray[i], repeatedField.Get(i).c_str());   
    }
    return const_cast<const char**>(cArray);
}


void printQuery(const zenoh::Query& query) {
    spdlog::debug("Query: {}", query.get_keyexpr().as_string_view());
}

fmi3Instance getInstance(int index) {
    auto it = instances.find(index);
    if (it == instances.end()) {
        throw std::out_of_range("Instance index out of range.");
    }
    return it->second;
}

proto::Status transformToProtoStatus(fmi3Status status) {
    switch (status) {
        case fmi3OK: 
            return proto::OK;
        case fmi3Warning: 
            return proto::WARNING;
        case fmi3Discard: 
            return proto::DISCARD;
        case fmi3Error: 
            return proto::ERROR;
        case fmi3Fatal: 
            return proto::FATAL;
        default: 
            throw std::invalid_argument("Invalid FMI3Status value");
    }
}

proto::fmi3StatusMessage makeFmi3StatusMessage(fmi3Status status) {
    proto::fmi3StatusMessage status_message;
    status_message.set_status(transformToProtoStatus(status));
    return status_message;
}

namespace fmu {
    fmi3SetDebugLoggingTYPE* fmi3SetDebugLogging;
    fmi3InstantiateCoSimulationTYPE* fmi3InstantiateCoSimulation;
    fmi3InstantiateModelExchangeTYPE* fmi3InstantiateModelExchange;
    fmi3InstantiateScheduledExecutionTYPE* fmi3InstantiateScheduledExecution;
    fmi3EnterEventModeTYPE* fmi3EnterEventMode;
    fmi3EnterInitializationModeTYPE* fmi3EnterInitializationMode;
    fmi3ExitInitializationModeTYPE* fmi3ExitInitializationMode;
    fmi3FreeInstanceTYPE* fmi3FreeInstance;
    fmi3DoStepTYPE* fmi3DoStep;
    fmi3SetFloat32TYPE* fmi3SetFloat32;
    fmi3GetFloat32TYPE* fmi3GetFloat32;
    fmi3SetFloat64TYPE* fmi3SetFloat64;
    fmi3GetFloat64TYPE* fmi3GetFloat64;
    fmi3SetInt8TYPE* fmi3SetInt8;
    fmi3GetInt8TYPE* fmi3GetInt8;
    fmi3SetUInt8TYPE* fmi3SetUInt8;
    fmi3GetUInt8TYPE* fmi3GetUInt8;
    fmi3SetInt16TYPE* fmi3SetInt16;
    fmi3GetInt16TYPE* fmi3GetInt16;
    fmi3SetUInt16TYPE* fmi3SetUInt16;
    fmi3GetUInt16TYPE* fmi3GetUInt16;
    fmi3SetInt32TYPE* fmi3SetInt32;
    fmi3GetInt32TYPE* fmi3GetInt32;
    fmi3SetUInt32TYPE* fmi3SetUInt32;
    fmi3GetUInt32TYPE* fmi3GetUInt32;
    fmi3SetInt64TYPE* fmi3SetInt64;
    fmi3GetInt64TYPE* fmi3GetInt64;
    fmi3SetUInt64TYPE* fmi3SetUInt64;
    fmi3GetUInt64TYPE* fmi3GetUInt64;
    fmi3SetBooleanTYPE* fmi3SetBoolean;
    fmi3GetBooleanTYPE* fmi3GetBoolean;
    fmi3SetStringTYPE* fmi3SetString;
    fmi3GetStringTYPE* fmi3GetString;
    fmi3SetClockTYPE* fmi3SetClock;
    fmi3GetClockTYPE* fmi3GetClock;
    fmi3SetBinaryTYPE* fmi3SetBinary;
    fmi3GetBinaryTYPE* fmi3GetBinary;
    fmi3ResetTYPE* fmi3Reset;
    fmi3TerminateTYPE* fmi3Terminate;
} 


namespace callbacks {

    void fmi3LogMessage(fmi3InstanceEnvironment instanceEnvironment,
                    fmi3Status status,
                    fmi3String category,
                    fmi3String message
                    ) {

        switch (status) {
            case fmi3OK:
                spdlog::info("[FMU log] fmi3OK: {} : {}", std::string(category), std::string(message));
                break;
            case fmi3Warning:
                spdlog::warn("[FMU log] fmi3Warning: {} : {}", std::string(category), std::string(message));
                break;
            case fmi3Discard:
                spdlog::warn("[FMU log] fmi3Discard: {} : {}", std::string(category), std::string(message));
                break;
            case fmi3Error:
                spdlog::error("[FMU log] fmi3Error: {} : {}", std::string(category), std::string(message));
                break;
            case fmi3Fatal:
                spdlog::critical("[FMU log] fmi3Fatal: {} : {}", std::string(category), std::string(message));
                break;
            default:
                spdlog::error("[FMU log] Unknown status: {} : {}", std::string(category), std::string(message));
                break;
        };

        // Publish log message to zenoh
        proto::logMessage log_message;
        log_message.set_status(transformToProtoStatus(status));
        log_message.set_category(category);
        log_message.set_message(message);
        std::vector<uint8_t> output_wire(log_message.ByteSizeLong()); 
        log_message.SerializeToArray(output_wire.data(), output_wire.size()); 
        fmi3LogMessagePublisher->put(zenoh::Bytes(std::move(output_wire)));
    }

    void fmi3SetDebugLogging(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3SetDebugLoggingMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3SetDebugLogging(
            getInstance(input.instance_index()),
            input.logging_on(),
            input.n_categories(),
            convertRepeatedFieldToCArray(input.categories())
        );

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

    void fmi3InstantiateCoSimulation(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3InstantiateCoSimulationMessage input;
        PARSE_QUERY(query, input)

        fmi3Instance instance = fmu::fmi3InstantiateCoSimulation(
            input.instance_name().c_str(),
            input.instantiation_token().c_str(),
            *resourcePath,
            input.visible(),
            input.logging_on(),
            input.event_mode_used(),
            input.early_return_allowed(),
            convertRepeatedFieldToCArray(input.required_intermediate_variables()),
            input.n_required_intermediate_variables(),
            nullptr,
            fmi3LogMessage,
            nullptr
        );

        proto::fmi3InstanceMessage output;
        instances[nextIndex] = instance;
        output.set_instance_index(nextIndex);
        SERIALIZE_REPLY(query, output)
        nextIndex++;
    }

    void fmi3InstantiateModelExchange(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3InstantiateModelExchangeMessage input;
        PARSE_QUERY(query, input)

        fmi3Instance instance = fmu::fmi3InstantiateModelExchange(
            input.instance_name().c_str(),
            input.instantiation_token().c_str(),
            *resourcePath,
            input.visible(),
            input.logging_on(),
            nullptr,
            fmi3LogMessage
        );

        proto::fmi3InstanceMessage output;
        instances[nextIndex] = instance;
        
        output.set_instance_index(nextIndex);
        SERIALIZE_REPLY(query, output)
        nextIndex++;
    }
    
    void fmi3InstantiateScheduledExecution(const zenoh::Query& query) {
        printQuery(query);
        proto::fmi3InstantiateScheduledExecutionMessage input;
        PARSE_QUERY(query, input)
        fmi3Instance instance = fmu::fmi3InstantiateScheduledExecution(
            input.instance_name().c_str(),
            input.instantiation_token().c_str(),
            *resourcePath,
            input.visible(),
            input.logging_on(),
            nullptr,
            fmi3LogMessage,
            nullptr,
            nullptr,
            nullptr
        );

        proto::fmi3InstanceMessage output;
        instances[nextIndex] = instance;
        
        output.set_instance_index(nextIndex);
        SERIALIZE_REPLY(query, output)
        nextIndex++;
    }

    void fmi3EnterEventMode(const zenoh::Query& query) {
        printQuery(query);
        
        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3EnterEventMode(getInstance(input.instance_index()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }


    void fmi3EnterInitializationMode(const zenoh::Query& query) {
        printQuery(query);
        
        proto::fmi3EnterInitializationModeMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3EnterInitializationMode(
            getInstance(input.instance_index()),
            input.tolerance_defined(),
            input.tolerance(),
            input.start_time(),
            input.stop_time_defined(),
            input.stop_time()
        );

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

    void fmi3ExitInitializationMode(const zenoh::Query& query) {
        printQuery(query);
        
        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3ExitInitializationMode(getInstance(input.instance_index()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

    void fmi3FreeInstance(const zenoh::Query& query) {
        printQuery(query);
        
        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        try {
            fmu::fmi3FreeInstance(getInstance(input.instance_index()));
        } catch (std::runtime_error& error) {
            spdlog::error("Failed to free FMU instance.");
        }
        try {
            auto it = instances.find(input.instance_index());
            instances.erase(it); 
        } catch (std::runtime_error& error) {
            spdlog::error("Failed to erase instance from instances.");
        }

        proto::voidMessage output;
        SERIALIZE_REPLY(query, output)

    }

    void fmi3DoStep(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3DoStepMessage input;
        PARSE_QUERY(query, input)

        fmi3Boolean event_handling_needed = input.event_handling_needed();
        fmi3Boolean terminate_simulation = input.terminate_simulation();
        fmi3Boolean early_return = input.early_return();
        fmi3Float64 last_successful_time = input.last_successful_time();
        fmi3Status status = fmu::fmi3DoStep(
            getInstance(input.instance_index()),
            input.current_communication_point(),
            input.communication_step_size(),
            input.no_set_fmu_state_prior_to_current_point(),
            &event_handling_needed,
            &terminate_simulation,
            &early_return,
            &last_successful_time
        );

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

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

    void fmi3SetString(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3SetStringInputMessage input;
        
        PARSE_QUERY(query, input)

        fmi3ValueReference* value_references = new fmi3ValueReference[input.n_value_references()];    
        for (int i = 0; i < input.n_value_references(); i++) {
            value_references[i] = input.value_references()[i];
        }
        fmi3String* values = new fmi3String[input.n_value_references()];
        for (int i = 0; i < input.n_value_references(); i++) {
            values[i] = input.values()[i].c_str();
        }

        fmi3Status status = fmu::fmi3SetString(
            getInstance(input.instance_index()),
            value_references,
            input.n_value_references(),
            values,
            input.n_values()
        );
        
        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }
    DEFINE_FMI3_GET_VALUE_FUNCTION(String)

    void fmi3SetClock(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3SetClockInputMessage input;
        
        PARSE_QUERY(query, input)
        fmi3ValueReference* value_references = new fmi3ValueReference[input.n_value_references()];    
        fmi3Clock* values = new fmi3Clock[input.n_value_references()];
        for (int i = 0; i < input.n_value_references(); i++) {
            value_references[i] = input.value_references()[i];
            values[i] = input.values()[i];
        }
        
        fmi3Status status = fmu::fmi3SetClock(
            getInstance(input.instance_index()),
            value_references,
            input.n_value_references(),
            values        
        );
        
        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

    void fmi3GetClock(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3GetClockInputMessage input;
        PARSE_QUERY(query, input)

        fmi3ValueReference* value_references = new fmi3ValueReference[input.n_value_references()];
        for (int i = 0; i < input.n_value_references(); i++) {
            value_references[i] = input.value_references()[i];
        }
        fmi3Clock* values = new fmi3Clock[input.n_value_references()];
        size_t nValues = input.n_value_references();

        fmi3Status status = fmu::fmi3GetClock(
            getInstance(input.instance_index()),
            value_references,
            input.n_value_references(),
            values
        );

        proto::fmi3GetClockOutputMessage output;
        for (int i = 0; i < input.n_value_references(); i++) {
            output.add_values(values[i]);
        }
        output.set_status(transformToProtoStatus(status));

        SERIALIZE_REPLY(query, output)
    }



    void fmi3SetBinary(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3SetBinaryInputMessage input;
        
        PARSE_QUERY(query, input)

        size_t nValueReferences = input.n_value_references();
        fmi3ValueReference* value_references = new fmi3ValueReference[nValueReferences];
        size_t* value_sizes = new size_t[nValueReferences];
        std::vector<uint8_t> values;

        size_t offset = 0;
        for (size_t i = 0; i < nValueReferences; ++i) {
            value_references[i] = input.value_references()[i];
            const std::string& binaryValue = input.values(i);
            value_sizes[i] = binaryValue.size();
            values.insert(values.end(), binaryValue.begin(), binaryValue.end());
        }

        fmi3Status status = fmu::fmi3SetBinary(
            getInstance(input.instance_index()),
            value_references,
            input.n_value_references(),
            value_sizes,
            reinterpret_cast<const fmi3Binary*>(values.data()),
            values.size()        
        );
        
        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

    void fmi3GetBinary(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3GetBinaryInputMessage input;
        PARSE_QUERY(query, input)

        size_t nValueReferences = input.n_value_references();
        fmi3ValueReference* value_references = new fmi3ValueReference[input.n_value_references()];
        size_t* value_sizes = new size_t[nValueReferences];
        fmi3Binary* values = new fmi3Binary[nValueReferences * MAX_BINARY_SIZE]; // Assuming MAX_BINARY_SIZE is defined
        size_t n_value = 0;

        for (size_t i = 0; i < nValueReferences; ++i) {
            value_references[i] = input.value_references()[i];
        }

        fmi3Status status = fmu::fmi3GetBinary(
            getInstance(input.instance_index()),
            value_references,
            nValueReferences,
            value_sizes,
            values,
            n_value
        );

        proto::fmi3GetBinaryOutputMessage output;
        size_t offset = 0;
        for (size_t i = 0; i < nValueReferences; ++i) {
            std::string binaryValue(reinterpret_cast<const char*>(values + offset), value_sizes[i]);
            output.add_values(binaryValue);
            offset += value_sizes[i];
        }
        output.set_status(transformToProtoStatus(status));

        SERIALIZE_REPLY(query, output)
    }

    
    void fmi3Reset(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3Reset(getInstance(input.instance_index()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }


    void fmi3Terminate(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3Terminate(getInstance(input.instance_index()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

}

std::string constructLibraryPath(const std::string& tempPath, const std::string& modelName) {
#ifdef _WIN32
    #ifdef _WIN64
        return tempPath + "/binaries/x86_64-windows/" + modelName + ".dll";
    #else
        return tempPath + "/binaries/x86-windows/" + modelName + ".dll";
    #endif
#else
    return tempPath + "/binaries/x86_64-linux/" + modelName + ".so";
#endif
}

int startServer(const std::string& fmuPath, const std::string& responderId, const std::string& zenohConfigPath, bool debug) {
    spdlog::info("\n"
             "====================================\n"
             "Serving FMU\n"
             "====================================\n"
             "FMU: {}\n"
             "Responder ID: {}\n"
             "{}"
             "{}"
             "====================================",
             fmuPath, 
             responderId, 
             (!zenohConfigPath.empty() ? fmt::format("Zenoh config file: {}\n", zenohConfigPath) : ""),
             (debug ? "DEBUG ENABLED\n" : ""));

    // Load the FMU library
    std::filesystem::path fmuFilePath(fmuPath);
    std::string modelName = fmuFilePath.stem().string();
    std::string tempPath = unzipFmu(fmuPath);
    std::string libPath = constructLibraryPath(tempPath, modelName);

    // Set the resource path
    std::string resourcePathStr = tempPath + "/resources";
    resourcePath = std::make_shared<fmi3String>(resourcePathStr.c_str());
    
    // Load the FMU library dynamically
    auto fmuLibrary = loadFmuLibrary(libPath);

    // Bind FMU library functions
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetDebugLogging)
    BIND_FMU_LIBRARY_FUNCTION(fmi3InstantiateCoSimulation)
    BIND_FMU_LIBRARY_FUNCTION(fmi3InstantiateModelExchange)
    BIND_FMU_LIBRARY_FUNCTION(fmi3InstantiateScheduledExecution)
    BIND_FMU_LIBRARY_FUNCTION(fmi3EnterEventMode)
    BIND_FMU_LIBRARY_FUNCTION(fmi3EnterInitializationMode)
    BIND_FMU_LIBRARY_FUNCTION(fmi3ExitInitializationMode)
    BIND_FMU_LIBRARY_FUNCTION(fmi3FreeInstance)
    BIND_FMU_LIBRARY_FUNCTION(fmi3DoStep)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetFloat32)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetFloat32)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetFloat64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetFloat64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetInt8)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetInt8)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetUInt8)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetUInt8)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetInt16)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetInt16)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetUInt16)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetUInt16)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetInt32)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetInt32)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetUInt32)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetUInt32)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetInt64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetInt64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetUInt64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetUInt64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetBoolean)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetBoolean)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetString)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetString)
    BIND_FMU_LIBRARY_FUNCTION(fmi3SetClock)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetClock)
    BIND_FMU_LIBRARY_FUNCTION(fmi3Reset)
    BIND_FMU_LIBRARY_FUNCTION(fmi3Terminate)

    // Start Zenoh Session
    zenoh::Config zconfig = zenohConfigPath.empty() ? zenoh::Config::create_default() : zenoh::Config::from_file(zenohConfigPath);
    session = std::make_shared<zenoh::Session>(zenoh::Session::open(std::move(zconfig)));

    // LogMessage publisher declaration
    std::string expr_fmi3LogMessage = "rpc/" + responderId + "/fmi3LogMessage";
    zenoh::KeyExpr keyexpr_fmi3LogMessage(expr_fmi3LogMessage);
    fmi3LogMessagePublisher = std::make_shared<zenoh::Publisher>(session->declare_publisher(keyexpr_fmi3LogMessage));

    // Queryable declarations
    DECLARE_QUERYABLE(fmi3SetDebugLogging, responderId) 
    DECLARE_QUERYABLE(fmi3InstantiateCoSimulation, responderId)
    DECLARE_QUERYABLE(fmi3InstantiateModelExchange, responderId)
    DECLARE_QUERYABLE(fmi3InstantiateScheduledExecution, responderId)
    DECLARE_QUERYABLE(fmi3EnterEventMode, responderId)
    DECLARE_QUERYABLE(fmi3EnterInitializationMode, responderId)
    DECLARE_QUERYABLE(fmi3ExitInitializationMode, responderId)
    DECLARE_QUERYABLE(fmi3FreeInstance, responderId)
    DECLARE_QUERYABLE(fmi3DoStep, responderId)
    DECLARE_QUERYABLE(fmi3SetFloat32, responderId)
    DECLARE_QUERYABLE(fmi3GetFloat32, responderId)
    DECLARE_QUERYABLE(fmi3SetFloat64, responderId)
    DECLARE_QUERYABLE(fmi3GetFloat64, responderId)
    DECLARE_QUERYABLE(fmi3SetInt8, responderId)
    DECLARE_QUERYABLE(fmi3GetInt8, responderId)
    DECLARE_QUERYABLE(fmi3SetUInt8, responderId)
    DECLARE_QUERYABLE(fmi3GetUInt8, responderId)
    DECLARE_QUERYABLE(fmi3SetInt16, responderId)
    DECLARE_QUERYABLE(fmi3GetInt16, responderId)
    DECLARE_QUERYABLE(fmi3SetUInt16, responderId)
    DECLARE_QUERYABLE(fmi3GetUInt16, responderId)
    DECLARE_QUERYABLE(fmi3SetInt32, responderId)
    DECLARE_QUERYABLE(fmi3GetInt32, responderId)
    DECLARE_QUERYABLE(fmi3SetUInt32, responderId)
    DECLARE_QUERYABLE(fmi3GetUInt32, responderId)
    DECLARE_QUERYABLE(fmi3SetInt64, responderId)
    DECLARE_QUERYABLE(fmi3GetInt64, responderId)
    DECLARE_QUERYABLE(fmi3SetUInt64, responderId)
    DECLARE_QUERYABLE(fmi3GetUInt64, responderId)
    DECLARE_QUERYABLE(fmi3SetBoolean, responderId)
    DECLARE_QUERYABLE(fmi3GetBoolean, responderId)
    DECLARE_QUERYABLE(fmi3SetString, responderId)
    DECLARE_QUERYABLE(fmi3GetString, responderId)
    DECLARE_QUERYABLE(fmi3SetClock, responderId)
    DECLARE_QUERYABLE(fmi3GetClock, responderId)
    DECLARE_QUERYABLE(fmi3SetBinary, responderId)
    DECLARE_QUERYABLE(fmi3GetBinary, responderId)
    DECLARE_QUERYABLE(fmi3Reset, responderId)
    DECLARE_QUERYABLE(fmi3Terminate, responderId)

    spdlog::info("Liaison server is now listening!");
    spdlog::info("Enter 'q' to quit...");
    int c = 0;
    while (c != 'q') {
        c = getchar();
        if (c == -1) {
#ifdef _WIN32
        Sleep(100);
#else
        usleep(100000); 
#endif
        }
    }

    // Reset shared pointers
    resourcePath.reset();
    session.reset();
    fmi3LogMessagePublisher.reset();

    // Unload the FMU library before exiting
    if (fmuLibrary) {
        unloadFmuLibrary(fmuLibrary);
        fmuLibrary = nullptr;
    }



    return 0;
}


void makeFmu(const std::string& fmuPath, const std::string& responderId, const std::string& zenohConfigPath) {
    spdlog::info("\n"
             "====================================\n"
             "Making Liaison FMU\n"
             "====================================\n"
             "FMU: {}\n"
             "Responder ID: {}\n"
             "{}"
             "====================================",
             fmuPath, 
             responderId, 
             (!zenohConfigPath.empty() ? fmt::format("Zenoh config file: {}\n", zenohConfigPath) : ""));

    std::filesystem::path fmuFilePath(fmuPath);
    std::string modelName = fmuFilePath.stem().string();
    std::string tempPath = unzipFmu(fmuPath);

    if (tempPath.empty()) {
        throw std::runtime_error("Failed to unzip the FMU.");
        return;
    }
    
    std::string outputFmuPath = "./" + modelName + "Liaison.fmu";

    // Create the ZIP/FMU archive
    int error = 0;
    zip_t* fmu = zip_open(outputFmuPath.c_str(), ZIP_CREATE | ZIP_TRUNCATE, &error);
    if (!fmu) {
        std::ostringstream oss;
        oss << "Failed to create the Liaison FMU: " << zip_strerror(fmu);
        throw std::runtime_error(oss.str());
    }

    // Add the renamed DLL files to the FMU inside the binaries/platform directory
    size_t nDynamicLibraryErrors = 0;
    std::filesystem::path binariesPath("./binaries");
    if (!std::filesystem::exists(binariesPath)) {
        zip_discard(fmu);
        throw std::runtime_error("Required directory './binaries' does not exist.");
    }
    std::string originalLinuxDynamicLibraryPath ="./binaries/x86_64-linux/libliaisonfmu.so";
    std::string renamedLinuxDynamicLibraryPath = "binaries/x86_64-linux/" + modelName + ".so";
    try {
        addFileToFmu(fmu, originalLinuxDynamicLibraryPath, renamedLinuxDynamicLibraryPath);
    } catch (std::runtime_error& error) {
        std::ostringstream oss;
        oss << "Failed adding Liaison Linux dynamic library file to the Liaison FMU: " << error.what();
        spdlog::warn(oss.str());
        nDynamicLibraryErrors++;
    }    
    
    std::string originalWindowsDynamicLibraryPath ="./binaries/x86_64-windows/liaisonfmu.dll";
    std::string renamedWindowsDynamicLibraryPath = "binaries/x86_64-windows/" + modelName + ".dll";  
    try {
        addFileToFmu(fmu, originalWindowsDynamicLibraryPath, renamedWindowsDynamicLibraryPath);
    } catch (std::runtime_error& error) {
        std::ostringstream oss;
        oss << "Failed adding Liaison Windows dynamic library file to the Liaison FMU: " << error.what();
        spdlog::warn(oss.str());
        nDynamicLibraryErrors++;
    }

    if (nDynamicLibraryErrors == 2) {
        zip_discard(fmu);
        throw std::runtime_error("Failed adding ANY Liaison dynamic library file to the Liaison FMU. At least one is required.");
    }

    // Add the modelDescription.xml file to the FMU at the base directory
    std::string modelDescriptionPath = tempPath + "/modelDescription.xml";
    try {
        addFileToFmu(fmu, modelDescriptionPath, "modelDescription.xml");
    } catch (std::runtime_error& error) {
        zip_discard(fmu);
        std::ostringstream oss;
        oss << "Failed adding modelDescription.xml to the Liaison FMU: " << error.what();
        throw std::runtime_error(oss.str());
    }

    // Read the zenoh config file
    json zenohConfig;
    if (!zenohConfigPath.empty()) {
        zenohConfig = json::parse(std::ifstream(zenohConfigPath));
        zenohConfig["metadata"]["name"] = modelName;

        // Determine if pem files are used
        if (zenohConfig.contains("transport") && 
            zenohConfig["transport"].contains("link") && 
            zenohConfig["transport"]["link"].contains("tls")) {
            
            json& tls = zenohConfig["transport"]["link"]["tls"];
            
            // Use value() method with default empty string if key doesn't exist
            std::string connectCertificatePath = tls.value("connect_certificate", "");
            std::string connectPrivateKeyPath = tls.value("connect_private_key", "");
            std::string rootCaCertificatePath = tls.value("root_ca_certificate", "");
            
            // Only log if we have actual values
            if (!connectCertificatePath.empty()) {
                if (!std::filesystem::exists(connectCertificatePath)) {
                    std::ostringstream oss;
                    oss << "Connect certificate file does not exist at: " << connectCertificatePath;
                    throw std::runtime_error(oss.str());
                }
                std::string connectCertificateFileName = std::filesystem::path(connectCertificatePath).filename().string();
                try {
                    addFileToFmu(fmu, connectCertificatePath, "binaries/" + connectCertificateFileName);
                    tls["connect_certificate"] = connectCertificateFileName;
                    spdlog::info("  Added : {}", connectCertificateFileName);
                } catch (std::runtime_error& error) {
                    zip_discard(fmu);
                    std::ostringstream oss;
                    oss << "Failed adding connect certificate file to the Liaison FMU: " << error.what();
                    throw std::runtime_error(oss.str());
                }
            }
            if (!connectPrivateKeyPath.empty()){
                if (!std::filesystem::exists(connectPrivateKeyPath)) {
                    std::ostringstream oss;
                    oss << "Connect private key file does not exist at: " << connectPrivateKeyPath;
                    throw std::runtime_error(oss.str());
                }
                std::string connectPrivateKeyFileName = std::filesystem::path(connectPrivateKeyPath).filename().string();
                try {
                    addFileToFmu(fmu, connectPrivateKeyPath, "binaries/" + connectPrivateKeyFileName);
                    tls["connect_private_key"] = connectPrivateKeyFileName;
                    spdlog::info("  Added : {}", connectPrivateKeyFileName);
                } catch (std::runtime_error& error) {
                    zip_discard(fmu);
                    std::ostringstream oss;
                    oss << "Failed adding connect private key file to the Liaison FMU: " << error.what();
                    throw std::runtime_error(oss.str());
                }
            }
            if (!rootCaCertificatePath.empty()){
                if (!std::filesystem::exists(rootCaCertificatePath)) {
                    std::ostringstream oss;
                    oss << "Root ca certificate file does not exist at: " << rootCaCertificatePath;
                    throw std::runtime_error(oss.str());
                }
                std::string rootCaCertificateFileName = std::filesystem::path(rootCaCertificatePath).filename().string();
                try {
                    addFileToFmu(fmu, rootCaCertificatePath, "binaries/" + rootCaCertificateFileName);
                    tls["root_ca_certificate"] = rootCaCertificateFileName;
                    spdlog::info("  Added : {}", rootCaCertificateFileName);
                } catch (std::runtime_error& error) {
                    zip_discard(fmu);
                    std::ostringstream oss;
                    oss << "Failed adding root CA certificate file to the Liaison FMU: " << error.what();
                    throw std::runtime_error(oss.str());
                }
            }
        }
    }

    // Create the config file
    json config;
    config["responderId"] = responderId;
    config["name"] = modelName;
    if (!zenohConfig.empty()) {
        config["zenohConfig"] = zenohConfig;
    }
    std::string configFilePath = tempPath + "/config.json";
    std::ofstream o(configFilePath);
    o << std::setw(4) << config << std::endl;
    o.close();

    try {
        addFileToFmu(fmu, configFilePath, "binaries/config.json");
    } catch (std::runtime_error& error) {
        zip_discard(fmu);
        std::ostringstream oss;
        oss << "Failed adding Liaison config file to the Liaison FMU: " << error.what();
        throw std::runtime_error(oss.str());
    }

    // Close the zip archive
    if (zip_close(fmu) < 0) {
        std::ostringstream oss;
        oss << "Failed to finalize FMU zip archive: " << zip_strerror(fmu);
        throw std::runtime_error(oss.str());
        return;
    }

    spdlog::info("Liaison FMU successfully created! ");
}


void printUsage() {
    std::cout <<"Usage:\n";
    std::cout <<"  liaison --serve <Path to FMU> <Responder Id>\n";
    std::cout <<"  liaison --make-fmu <Path to FMU> <Responder Id>\n";
    std::cout <<"  liaison --make-fmu <Path to FMU> <Responder Id> --debug\n";
    std::cout <<"  liaison --serve <Path to FMU> <Responder Id> --zenoh-config <Path to Zenoh config file>\n";
    std::cout <<"  liaison --make-fmu <Path to FMU> <Responder Id> --zenoh-config <Path to Zenoh config file>\n";
    std::cout <<"  liaison --serve <Path to FMU> <Responder Id> --python-env <Path to Python environment>\n";
    std::cout <<"  liaison --serve <Path to FMU> <Responder Id> --pyhton-lib <Path to Python library>\n";
}

void loadPythonLibFromVenv(const std::string& venvPath) {
    // Open pyvenv.cfg
    std::string cfgPath = venvPath + "/pyvenv.cfg";
    FILE* cfgFile = fopen(cfgPath.c_str(), "r");
  

    // Extract home path and version
    char line[1024];
    std::string homePath;
    std::string version;
    while (fgets(line, sizeof(line), cfgFile)) {
        std::string lineStr(line);
        if (lineStr.find("home = ") != std::string::npos) {
            homePath = lineStr.substr(lineStr.find("home = ") + 7);
            if (!homePath.empty() && homePath.back() == '\n') {
                homePath.pop_back();
            }
        } else if (lineStr.find("version = ") != std::string::npos) {
            version = lineStr.substr(lineStr.find("version = ") + 10);
            if (!version.empty() && version.back() == '\n') {
                version.pop_back();
            }
        }
    }
    fclose(cfgFile);

    if (homePath.empty() || version.empty()) {
        throw std::runtime_error("Could not retrieve values for 'home' or 'version' from 'pyvenv.cfg'");
        return;
    }

    // Remove 'bin' from the end of the homePath if present
    if (homePath.length() >= 4 && homePath.substr(homePath.length() - 4) == "/bin") {
        homePath = homePath.substr(0, homePath.length() - 4);
    }

    // Limit the version to major.minor
    size_t dotPos = version.find('.');
    if (dotPos != std::string::npos) {
        size_t nextDotPos = version.find('.', dotPos + 1);
        if (nextDotPos != std::string::npos) {
            version = version.substr(0, nextDotPos);
        }
    }

    // If windows, remove the dot between major and minor
#ifdef _WIN32
    version.erase(std::remove(version.begin(), version.end(), '.'), version.end());
#endif
    spdlog::debug("Using Python home: {}", homePath);
    spdlog::debug("Using Python version: {}", version);
   
    // Set PYTHONPATH to site-packages within venvPath
#ifdef _WIN32
    std::string pythonPath = venvPath + "\\Lib\\site-packages";
    if (!std::filesystem::exists(pythonPath)) {
        throw std::runtime_error("Could not find site-packages directory at the expected location: " + pythonPath);
    }
    _putenv_s("PYTHONPATH", pythonPath.c_str());
#else
    std::string pythonPath = venvPath + "/lib/python" + version + "/site-packages";
    if (!std::filesystem::exists(pythonPath)) {
        throw std::runtime_error("Could not find site-packages directory at the expeced location: " + pythonPath);
    }
    setenv("PYTHONPATH", pythonPath.c_str(), 1);
#endif

    // Set LD_PRELOAD to home
#ifdef _WIN32
    std::string pythonLibPath = homePath + "\\python" + version + ".dll";
    _putenv_s("LD_PRELOAD", pythonLibPath.c_str());
#else
    std::string pythonLibPath = homePath + "/lib/libpython" + version + ".so";
    setenv("LD_PRELOAD", pythonLibPath.c_str(), 1);
#endif

}

void loadPythonLibFromConda(const std::string& venvPath) {

#ifdef _WIN32
    std::string pythonLibPath = venvPath + "\\Python3.dll";
    if (!std::filesystem::exists(pythonLibPath)) {
        throw std::runtime_error("Could not find Python library at the expected location: " + pythonLibPath);
    }
    _putenv_s("LD_PRELOAD", pythonLibPath.c_str());
#else
    std::string pythonLibPath = venvPath + "/lib/libpython3.so";
    if (!std::filesystem::exists(pythonLibPath)) {
        throw std::runtime_error("Could not find Python library at the expected location: " + pythonLibPath);
    }
    setenv("LD_PRELOAD", pythonLibPath.c_str(), 1);
#endif

}



int main(int argc, char* argv[]) {
    bool debug = false;
    try {
        // Parse command line arguments
        if (argc < 4) {
            throw std::invalid_argument("Invalid number of arguments.");
        }
        std::string option = argv[1];
        std::string fmuPath = argv[2];
        std::string responderId = argv[3];

        // Parse optional flags
        std::string zenohConfigPath;
        std::string pythonEnvPath;
        for (int i = 4; i < argc; ++i) {
            std::string arg = argv[i];
            if (arg == "--debug") {
                debug = true;
                zenoh::init_log_from_env_or("debug");
                spdlog::set_level(spdlog::level::debug);
            } else if (arg == "--zenoh-config" && i + 1 < argc) {
                zenohConfigPath = argv[++i];
                if (!std::filesystem::exists(zenohConfigPath)) {
                    std::ostringstream oss;
                    oss << "Zenoh config file does not exist at: " << zenohConfigPath;
                    throw std::runtime_error(oss.str());
                }
            } else if (arg == "--python-env" && i + 1 < argc) {
                pythonEnvPath = argv[++i];
                if (!std::filesystem::is_directory(pythonEnvPath)) {
                    std::ostringstream oss;
                    oss << "Python environment directory does not exist at: " << pythonEnvPath;
                    throw std::runtime_error(oss.str());
                }
            } else {
                std::ostringstream oss;
                oss << "Unknown argument: " << arg;
                throw std::invalid_argument(oss.str());
            }
        }

        // Re-execute the process if either python-env or python-lib flag is provided,
        // but only if it hasn't been already re-executed.
        if (getenv("LIAISON_RELAUNCHED") == nullptr && !pythonEnvPath.empty()) {
        
            // Check if pyenv.cfg exists to determine if it's a venv
            std::string cfgPath = pythonEnvPath + "/pyvenv.cfg";
            if (std::filesystem::exists(cfgPath)) {
                // It's a venv environment
                loadPythonLibFromVenv(pythonEnvPath);
            } else {
                spdlog::debug("Could not find a 'pyvenv.cfg', assuming Conda virtual environment.");
                // Assume it's a Conda environment
                loadPythonLibFromConda(pythonEnvPath);
            }

            // Set a marker to prevent reexecution.
            #ifdef _WIN32
            _putenv_s("LIAISON_RELAUNCHED", "1");
            #else
            setenv("LIAISON_RELAUNCHED", "1", 1);
            #endif

            // Rebuild the argument list and pass all original arguments to the new process.
            #ifdef _WIN32
            STARTUPINFO si = {sizeof(STARTUPINFO)};
            PROCESS_INFORMATION pi;
            if (!CreateProcess(NULL, GetCommandLine(), NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi)) {
                std::ostringstream oss;
                oss << "Re-execution to preload the Python shared library failed: " << GetLastError();
                throw std::runtime_error(oss.str());
            }
            WaitForSingleObject(pi.hProcess, INFINITE);
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            #else
            execvp(argv[0], argv);
            #endif
            // If we reach this point, the re-execution failed.
            throw std::runtime_error("Re-execution to preload the Python shared library failed."); 
        } 

        if ((getenv("LIAISON_RELAUNCHED") != nullptr)) {
            spdlog::debug("Re-executed to preload the Python shared library: {}",getenv("LD_PRELOAD"));
        }
    
        if (option == "--serve") {
            startServer(fmuPath, responderId, zenohConfigPath, debug);
        } else if (option == "--make-fmu") {
            makeFmu(fmuPath, responderId, zenohConfigPath);
        } else {
            std::ostringstream oss;
            oss << "Unknown argument:: " << option;
            throw std::invalid_argument(oss.str());
        }
    } catch (const std::invalid_argument& e) {
        spdlog::error(e.what());
        printUsage();
        return 1;
    } catch (const std::exception& e) {
        spdlog::error(e.what());
        return 1;
    }
    return 0;
}
