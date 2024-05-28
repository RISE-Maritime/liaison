#include <dlfcn.h> 
#include <unistd.h>
#include <sys/stat.h>
#include <unordered_map>
#include <zip.h>
#include <fstream>
#include <filesystem>
#include <dlfcn.h>
#include "zenoh.hxx"
#include "fmi3.pb.h"
#include "fmi3Functions.h"
#include "fmi3Logging.hpp"


// MACROS

#define DECLARE_QUERYABLE(FMI3FUNCTION) \
    std::string expr_##FMI3FUNCTION = "rpc/" + responder_id + "/" + std::string(#FMI3FUNCTION); \
    std::cout << expr_##FMI3FUNCTION << std::endl; \
    zenoh::KeyExprView keyexpr_##FMI3FUNCTION(expr_##FMI3FUNCTION); \
    auto queryable_##FMI3FUNCTION = zenoh::expect<zenoh::Queryable>(z_server.declare_queryable(keyexpr_##FMI3FUNCTION,callbacks::FMI3FUNCTION));

#define PARSE_QUERY(QUERY, INPUT) \
    auto query_value = QUERY.get_value(); \
    INPUT.ParseFromArray(query_value.payload.start, query_value.payload.len); \

#define SERIALIZE_REPLY(QUERY, OUTPUT) \
    size_t output_size = OUTPUT.ByteSizeLong(); \
    std::vector<uint8_t> buffer(output_size); \
    OUTPUT.SerializeToArray(buffer.data(), output_size); \
    zenoh::QueryReplyOptions options; \
    options.set_encoding(zenoh::Encoding(Z_ENCODING_PREFIX_APP_CUSTOM)); \
    QUERY.reply(QUERY.get_keyexpr(), buffer); \

#define BIND_FMU_LIBRARY_FUNCTION(FMI3FUNCTION) \
    fmu::FMI3FUNCTION = (FMI3FUNCTION##TYPE*)dlsym(fmuLibrary, #FMI3FUNCTION);

// end of MACROS

// Map that holds the fmi3 instances
std::unordered_map<int, fmi3Instance> instances;
int nextIndex = 0;

void createDirectories(const std::string& path) {
    std::filesystem::create_directories(path);
}

std::string createTempDirectory() {
    char tempDirTemplate[] = "/tmp/liaison.XXXXXX";
    char* tempDirPath = mkdtemp(tempDirTemplate);
    if (tempDirPath == nullptr) {
        throw std::runtime_error("Failed to create temporary directory.");
    }
    return std::string(tempDirPath);
}

void unzipFmu(const std::string& fmuPath, const std::string& outputDir) {
    int err = 0;
    zip *z = zip_open(fmuPath.c_str(), 0, &err);
    if (z == nullptr) {
        throw std::runtime_error("Failed to open FMU zip file.");
    }

    // Create output directory if it doesn't exist
    createDirectories(outputDir);

    struct zip_stat st;
    zip_stat_init(&st);
    zip_file *zf = nullptr;

    for (int i = 0; i < zip_get_num_entries(z, 0); ++i) {
        if (zip_stat_index(z, i, 0, &st) == 0) {
            std::string outPath = outputDir + "/" + st.name;

            // If it's a directory, create it
            if (outPath.back() == '/') {
                createDirectories(outPath);
            } else {
                zf = zip_fopen_index(z, i, 0);
                if (!zf) {
                    zip_close(z);
                    throw std::runtime_error("Failed to open file in zip archive.");
                }

                // Ensure the directory for the file exists
                createDirectories(std::filesystem::path(outPath).parent_path().string());

                std::ofstream outFile(outPath, std::ios::binary);
                if (!outFile) {
                    zip_fclose(zf);
                    zip_close(z);
                    throw std::runtime_error("Failed to create file on disk: " + outPath);
                }

                std::vector<char> buffer(st.size);
                zip_fread(zf, buffer.data(), st.size);
                outFile.write(buffer.data(), buffer.size());

                outFile.close();
                zip_fclose(zf);
            }
        }
    }
    zip_close(z);
}

const fmi3ValueReference* convertRepeatedFieldToCArray(const google::protobuf::RepeatedField<int>& repeatedField) {
    size_t size = repeatedField.size();
    fmi3ValueReference* cArray = new fmi3ValueReference[size];
    for (size_t i = 0; i < size; ++i) {
        cArray[i] = static_cast<fmi3ValueReference>(repeatedField.Get(i));
    }
    return cArray;
}

void printQuery(const zenoh::Query& query) {
    std::cout << "Query: " << query.get_keyexpr().as_string_view() << std::endl;
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
    fmi3InstantiateCoSimulationTYPE* fmi3InstantiateCoSimulation;
    fmi3EnterInitializationModeTYPE* fmi3EnterInitializationMode;
    fmi3ExitInitializationModeTYPE* fmi3ExitInitializationMode;
    fmi3FreeInstanceTYPE* fmi3FreeInstance;
    fmi3DoStepTYPE* fmi3DoStep;
    fmi3GetFloat64TYPE* fmi3GetFloat64;
    fmi3TerminateTYPE* fmi3Terminate;
} 

bool is_valid_utf8(const std::string& string) {
    std::string::const_iterator it = string.begin();
    while (it != string.end()) {
        if ((*it & 0x80) == 0x00) { // ASCII
            ++it;
        } else if ((*it & 0xE0) == 0xC0) { // 2-byte UTF-8
            if (it + 1 == string.end() || (it[1] & 0xC0) != 0x80) return false;
            it += 2;
        } else if ((*it & 0xF0) == 0xE0) { // 3-byte UTF-8
            if (it + 2 >= string.end() || (it[1] & 0xC0) != 0x80 || (it[2] & 0xC0) != 0x80) return false;
            it += 3;
        } else if ((*it & 0xF8) == 0xF0) { // 4-byte UTF-8
            if (it + 3 >= string.end() || (it[1] & 0xC0) != 0x80 || (it[2] & 0xC0) != 0x80 || (it[3] & 0xC0) != 0x80) return false;
            it += 4;
        } else {
            return false;
        }
    }
    return true;
}



namespace callbacks {

    void fmi3InstantiateCoSimulation(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3InstantiateCoSimulationMessage input;
        PARSE_QUERY(query, input)

        // TODO: Resource path should determined according to the place
        // where the FMU is unpacked.

        fmi3Instance instance = fmu::fmi3InstantiateCoSimulation(
            input.instance_name().c_str(),
            input.instantiation_token().c_str(),
            nullptr,
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
        
        output.set_instance(nextIndex);
        SERIALIZE_REPLY(query, output)
    }


    void fmi3EnterInitializationMode(const zenoh::Query& query) {
        printQuery(query);
        
        proto::fmi3EnterInitializationModeMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3EnterInitializationMode(
            getInstance(input.instance()),
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

        fmi3Status status = fmu::fmi3ExitInitializationMode(getInstance(input.instance()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

    void fmi3FreeInstance(const zenoh::Query& query) {
        printQuery(query);
        
        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        try {
            fmu::fmi3FreeInstance(getInstance(input.instance()));
        } catch (std::runtime_error& error) {
            std::cerr << "Failed to free FMU instance." << std::endl;
        }
        try {
            auto it = instances.find(input.instance());
            instances.erase(it); 
        } catch (std::runtime_error& error) {
            std::cerr << "Failed to erase instance from instances." << std::endl;
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
            getInstance(input.instance()),
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

    void fmi3GetFloat64(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3GetFloat64InputMessage input;
        PARSE_QUERY(query, input)

        fmi3ValueReference value_references[input.n_value_references()];
        for (int i = 0; i < input.n_value_references(); i++) {
            value_references[i] = input.value_references()[i];
        }
        fmi3Float64 values[input.n_value_references()];
        size_t nValues = input.n_value_references();

        fmi3Status status = fmu::fmi3GetFloat64(
            getInstance(input.instance()),
            value_references,
            input.n_value_references(),
            values,
            nValues
        );

        proto::fmi3GetFloat64OutputMessage output;
        for (int i = 0; i < input.n_value_references(); i++) {
            output.add_values(values[i]);
        }
        output.set_n_values(nValues);
        output.set_status(transformToProtoStatus(status));

        SERIALIZE_REPLY(query, output)
    }

    void fmi3Terminate(const zenoh::Query& query) {
        printQuery(query);

        proto::fmi3InstanceMessage input;
        PARSE_QUERY(query, input)

        fmi3Status status = fmu::fmi3Terminate(getInstance(input.instance()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

}

int main(int argc, char* argv[]) {

    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <path_to_fmu>" << std::endl;
        return 1;
    }

    // Load the FMU shared library
    std::string fmuPath = argv[1];
     std::filesystem::path fmuFilePath(fmuPath);
    std::string fmuName = fmuFilePath.stem().string();
    unzipFmu(fmuPath, "./tmp");
    std::string libPath = "./tmp/binaries/x86_64-linux/" + fmuName + ".so";
    void* fmuLibrary = dlopen(libPath.c_str(), RTLD_LAZY);
    if (!fmuLibrary) {
        throw std::runtime_error("Failed to load FMU library: " + std::string(dlerror()));
    }

    // Bind FMU library functions
    BIND_FMU_LIBRARY_FUNCTION(fmi3InstantiateCoSimulation)
    BIND_FMU_LIBRARY_FUNCTION(fmi3EnterInitializationMode)
    BIND_FMU_LIBRARY_FUNCTION(fmi3ExitInitializationMode)
    BIND_FMU_LIBRARY_FUNCTION(fmi3FreeInstance)
    BIND_FMU_LIBRARY_FUNCTION(fmi3DoStep)
    BIND_FMU_LIBRARY_FUNCTION(fmi3GetFloat64)
    BIND_FMU_LIBRARY_FUNCTION(fmi3Terminate)

    // TODO declare responder_id elsewhere
    std::string responder_id = "demo";

    // Start Zenoh Session
    zenoh::Config config;
    printf("Opening session...\n");
    auto z_server = zenoh::expect<zenoh::Session>(zenoh::open(std::move(config)));

    // Queryable declarations
    DECLARE_QUERYABLE(fmi3InstantiateCoSimulation)
    DECLARE_QUERYABLE(fmi3EnterInitializationMode)
    DECLARE_QUERYABLE(fmi3ExitInitializationMode)
    DECLARE_QUERYABLE(fmi3FreeInstance)
    DECLARE_QUERYABLE(fmi3DoStep)
    DECLARE_QUERYABLE(fmi3GetFloat64)
    DECLARE_QUERYABLE(fmi3Terminate)

    printf("Portal Server is listening!\n");
    printf("Enter 'q' to quit...\n");
    int c = 0;
    while (c != 'q') {
        c = getchar();
        if (c == -1) {
            usleep(1);
        }
    }

    return 0;

}