#include <dlfcn.h> 
#include <unistd.h>
#include <sys/stat.h>
#include <unordered_map>
#include <zip.h>
#include <fstream>
#include <dlfcn.h>
#include "zenoh.hxx"
#include "fmi3.pb.h"
#include "fmi3Functions.h"


// MACROS

#define DECLARE_QUERYABLE(FMI3FUNCTION) \
    zenoh::KeyExprView keyexpr("rpc/" + responder_id + "/" + std::string(#FMI3FUNCTION)); \
    auto queryable_##FMI3FUNCTION = zenoh::expect<zenoh::Queryable>(z_server.declare_queryable(keyexpr, FMI3FUNCTION));

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

void unzipFmu(const std::string& fmuPath, const std::string& outputDir) {
    int err = 0;
    zip *z = zip_open(fmuPath.c_str(), 0, &err);
    if (z == nullptr) {
        throw std::runtime_error("Failed to open FMU zip file.");
    }

    // Create output directory if it doesn't exist
    mkdir(outputDir.c_str(), 0755);

    struct zip_stat st;
    zip_stat_init(&st);
    zip_file *zf = nullptr;

    for (int i = 0; i < zip_get_num_entries(z, 0); ++i) {
        if (zip_stat_index(z, i, 0, &st) == 0) {
            std::string outPath = outputDir + "/" + st.name;

            // If it's a directory, create it
            if (outPath.back() == '/') {
                mkdir(outPath.c_str(), 0755);
            } else {
                zf = zip_fopen_index(z, i, 0);
                if (!zf) {
                    zip_close(z);
                    throw std::runtime_error("Failed to open file in zip archive.");
                }

                std::ofstream outFile(outPath, std::ios::binary);
                if (!outFile) {
                    zip_fclose(zf);
                    zip_close(z);
                    throw std::runtime_error("Failed to create file on disk.");
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

proto::fmi3StatusMessage makeFmi3StatusMessage(fmi3Status status) {
    proto::fmi3StatusMessage status_message;
    switch (status) {
        case fmi3OK: status_message.set_status(proto::OK);
        case fmi3Warning: status_message.set_status(proto::WARNING);
        case fmi3Discard: status_message.set_status(proto::DISCARD);
        case fmi3Error: status_message.set_status(proto::ERROR);
        case fmi3Fatal: status_message.set_status(proto::FATAL);
        default: throw std::invalid_argument("Invalid FMI3Status value");
    }
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

void fmi3InstantiateCoSimulationCallback(const zenoh::Query& query) {
    printQuery(query);

    proto::fmi3InstantiateCoSimulationMessage input;
    PARSE_QUERY(query, input)

    fmi3Instance instance = fmu::fmi3InstantiateCoSimulation(
        input.instance_name().c_str(),
        input.instantiation_token().c_str(),
        input.resource_path().c_str(),
        input.visible(),
        input.logging_on(),
        input.event_mode_used(),
        input.early_return_allowed(),
        convertRepeatedFieldToCArray(input.required_intermediate_variables()),
        input.n_required_intermediate_variables(),
        nullptr,
        nullptr,
        nullptr
    );

    proto::fmi3InstanceMessage output;
    instances[nextIndex] = instance;
    
    output.set_instance(nextIndex);
    SERIALIZE_REPLY(query, output)
}


void fmi3EnterInitializationModeCallback(const zenoh::Query& query) {
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

void fmi3ExitInitializationModeCallback(const zenoh::Query& query) {
    printQuery(query);
    
    proto::fmi3InstanceMessage input;
    PARSE_QUERY(query, input)

    fmi3Status status = fmu::fmi3ExitInitializationMode(getInstance(input.instance()));

    proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
    SERIALIZE_REPLY(query, output)
}

void fmi3FreeInstanceCallback(const zenoh::Query& query) {
    printQuery(query);
    
    proto::fmi3InstanceMessage input;
    PARSE_QUERY(query, input)

    fmu::fmi3FreeInstance(getInstance(input.instance()));

    proto::voidMessage output;
    SERIALIZE_REPLY(query, output)

}

void fmi3DoStepCallback(const zenoh::Query& query) {
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

void fmi3GetFloat64Callback(const zenoh::Query& query) {
    printQuery(query);

    proto::fmi3GetFloat64Message input;
    PARSE_QUERY(query, input)

    fmi3ValueReference value_references[input.n_value_references()];
    for (int i = 0; i < input.n_value_references(); i++) {
        value_references[i] = input.value_references()[i];
    }
    fmi3Float64 values[input.n_values()];
    for (int i = 0; i < input.n_values(); i++) {
        values[i] = input.values()[i];
    }
    fmi3Status status = fmu::fmi3GetFloat64(
        getInstance(input.instance()),
        value_references,
        input.n_value_references(),
        values,
        input.n_values()
    );

    proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
    SERIALIZE_REPLY(query, output)
}

void fmi3TerminateCallback(const zenoh::Query& query) {
    printQuery(query);

    proto::fmi3InstanceMessage input;
    PARSE_QUERY(query, input)

    fmi3Status status = fmu::fmi3Terminate(getInstance(input.instance()));

    proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
    SERIALIZE_REPLY(query, output)
}

int main() {

    // Load the FMU shared library
    std::string fmuPath = "../BouncingBall.fmu"; 
    unzipFmu(fmuPath, "./tmp");
    std::string libPath = "./tmp/binaries/x86_64-linux/BouncingBall.so";
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
    DECLARE_QUERYABLE(fmi3InstantiateCoSimulationCallback)


    printf("fmi3Server is listening!\n");
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