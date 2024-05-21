#include <dlfcn.h> 
#include <unistd.h>
#include <sys/stat.h>
#include <zip.h>
#include <fstream>
#include "zenoh.hxx"
#include "fmi3.pb.h"



// MACROS


#define DECLARE_QUERYABLE(FMI3FUNCTION) \
    zenoh::KeyExprView keyexpr("rpc/" + responder_id + "/query"); \
    auto queryable_##FMI3FUNCTION = zenoh::expect<zenoh::Queryable>(z_server.declare_queryable(keyexpr, FMI3FUNCTION));

#define PARSE_INPUT() \
    auto query_value = query.get_value(); \
    input.ParseFromArray(query_value.payload.start, query_value.payload.len); \

#define SERIALIZE_OUTPUT_AND_REPLY() \
    size_t output_size = output.ByteSizeLong(); \
    std::vector<uint8_t> buffer(output_size); \
    output.SerializeToArray(buffer.data(), output_size); \
    zenoh::QueryReplyOptions options; \
    options.set_encoding(zenoh::Encoding(Z_ENCODING_PREFIX_APP_CUSTOM)); \
    query.reply(query.get_keyexpr(), buffer); \


// end of MACROS

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

void fmi3InstantiateCoSimulation(const zenoh::Query& query) {
    std::cout << "Query: " << query.get_keyexpr().as_string_view() << std::endl;

    proto::fmi3InstantiateCoSimulationMessage input;
    proto::fmi3InstanceMessage output;

    PARSE_INPUT()

    output.set_instance(1);
    SERIALIZE_OUTPUT_AND_REPLY()
}


int main() {

    // Load the FMU shared library
    std::string fmuPath = "../BouncingBall.fmu"; 
    unzipFmu(fmuPath, "./tmp");
    std::string libPath = "./tmp/binaries/x86_64-linux/BouncingBall.so";
    void* fmu = dlopen(libPath.c_str(), RTLD_LAZY);
    if (!fmu) {
        throw std::runtime_error("Failed to load FMU library: " + std::string(dlerror()));
    }

    // // Load FMI functions
    // fmi3InstantiateCoSimulation = (fmi3InstantiateCoSimulationTYPE*)dlsym(fmuLibrary, "fmi3InstantiateCoSimulation");
    // fmi3SetupExperiment = (fmi3SetupExperimentTYPE*)dlsym(fmuLibrary, "fmi3SetupExperiment");
    // fmi3EnterInitializationMode = (fmi3EnterInitializationModeTYPE*)dlsym(fmuLibrary, "fmi3EnterInitializationMode");
    // fmi3ExitInitializationMode = (fmi3ExitInitializationModeTYPE*)dlsym(fmuLibrary, "fmi3ExitInitializationMode");
    // fmi3DoStep = (fmi3DoStepTYPE*)dlsym(fmuLibrary, "fmi3DoStep");
    // fmi3Terminate = (fmi3TerminateTYPE*)dlsym(fmuLibrary, "fmi3Terminate");
    // fmi3FreeInstance = (fmi3FreeInstanceTYPE*)dlsym(fmuLibrary, "fmi3FreeInstance");

    // // Check that all function pointers were loaded correctly
    // if (!fmi3InstantiateCoSimulation || !fmi3SetupExperiment || !fmi3EnterInitializationMode || !fmi3ExitInitializationMode || !fmi3DoStep || !fmi3Terminate || !fmi3FreeInstance) {
    //     throw std::runtime_error("Failed to load FMI functions.");
    // }

    // TODO declare responder_id elsewhere
    std::string responder_id = "demo";

    // Start Zenoh Session
    zenoh::Config config;
    printf("Opening session...\n");
    auto z_server = zenoh::expect<zenoh::Session>(zenoh::open(std::move(config)));

    // Queryable declarations
    DECLARE_QUERYABLE(fmi3InstantiateCoSimulation)


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