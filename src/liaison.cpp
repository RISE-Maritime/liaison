#include <dlfcn.h> 
#include <unistd.h>
#include <sys/stat.h>
#include <unordered_map>
#include <filesystem>
#include <zip.h>
#include <iostream>

#include "zenoh.hxx"
#include "fmi3.pb.h"
#include "fmi3Functions.h"
#include "utils.hpp"


// MACROS

#define DECLARE_QUERYABLE(FMI3FUNCTION, RESPONDER_ID) \
    std::string expr_##FMI3FUNCTION = "rpc/" + RESPONDER_ID + "/" + std::string(#FMI3FUNCTION); \
    zenoh::KeyExpr keyexpr_##FMI3FUNCTION(expr_##FMI3FUNCTION); \
    auto on_drop_queryable_##FMI3FUNCTION = []() { std::cout << "Destroying queryable for " << #FMI3FUNCTION << "\n"; }; \
    auto queryable_##FMI3FUNCTION = session.declare_queryable(keyexpr_##FMI3FUNCTION, std::function<void(const zenoh::Query&)>(callbacks::FMI3FUNCTION), on_drop_queryable_##FMI3FUNCTION); \

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


#define BIND_FMU_LIBRARY_FUNCTION(FMI3FUNCTION) \
    fmu::FMI3FUNCTION = (FMI3FUNCTION##TYPE*)dlsym(fmuLibrary, #FMI3FUNCTION);

// end of MACROS

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
        
        output.set_instance_index(nextIndex);
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
            std::cerr << "Failed to free FMU instance." << std::endl;
        }
        try {
            auto it = instances.find(input.instance_index());
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
            getInstance(input.instance_index()),
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

        fmi3Status status = fmu::fmi3Terminate(getInstance(input.instance_index()));

        proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
        SERIALIZE_REPLY(query, output)
    }

}


int startServer(const std::string& fmuPath, const std::string& responderId) {
    std::cout << "Starting server using:" << std::endl;
    std::cout << "  FMU: " << fmuPath << std::endl;
    std::cout << "  responderId: " << responderId << std::endl;

    // Load the FMU library
    std::filesystem::path fmuFilePath(fmuPath);
    std::string modelName = fmuFilePath.stem().string();
    std::string tempPath = unzipFmu(fmuPath);
    std::string libPath = tempPath + "/binaries/x86_64-linux/" + modelName + ".so";
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

    // Start Zenoh Session
    zenoh::Config config = zenoh::Config::create_default();
    auto session = zenoh::Session::open(std::move(config));

    // Queryable declarations
    DECLARE_QUERYABLE(fmi3InstantiateCoSimulation, responderId)
    DECLARE_QUERYABLE(fmi3EnterInitializationMode, responderId)
    DECLARE_QUERYABLE(fmi3ExitInitializationMode, responderId)
    DECLARE_QUERYABLE(fmi3FreeInstance, responderId)
    DECLARE_QUERYABLE(fmi3DoStep, responderId)
    DECLARE_QUERYABLE(fmi3GetFloat64, responderId)
    DECLARE_QUERYABLE(fmi3Terminate, responderId)

    printf("Now is listening!\n");
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


std::string createResponderIdFile(const std::string& directory, const std::string& responderId) {
    std::filesystem::path filePath = std::filesystem::path(directory) / "responderId";

    std::ofstream file(filePath);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to create responderId file at: " + filePath.string());
    }
    file << "responderId='" << responderId << "'";
    file.close();

    // Verify 
    if (!std::filesystem::exists(filePath)) {
        throw std::runtime_error("responderId file creation failed at: " + filePath.string());
    }
    return filePath;
}

void generateFmu(const std::string& fmuPath, const std::string& responderId) {
    std::cout << "Generating Liaison FMU using:" << std::endl;
    std::cout << "  FMU: '" << fmuPath << "'" << std::endl;
    std::cout << "  responderId: '" << responderId << "'" << std::endl;;

    

    std::filesystem::path fmuFilePath(fmuPath);
    std::string modelName = fmuFilePath.stem().string();
    std::string tempPath = unzipFmu(fmuPath);

    if (tempPath.empty()) {
        std::cerr << "Failed to unzip FMU." << std::endl;
        return;
    }
    
    std::string outputFmuPath = "./" + modelName + "Liaison.fmu";

    // Create the ZIP/FMU archive
    int error = 0;
    zip_t* fmu = zip_open(outputFmuPath.c_str(), ZIP_CREATE | ZIP_TRUNCATE, &error);
    if (!fmu) {
        std::cerr << "Failed to create FMU at: " << outputFmuPath << std::endl;
        return;
    }

    // Add the renamed DLL file to the FMU inside the binaries/platform directory
    std::string originalDllPath ="./binaries/x86_64-linux/libliaisonfmu.so";
    std::string renamedDllPath = "binaries/x86_64-linux/" + modelName + ".so";    
    if (!addFileToZip(fmu, originalDllPath, renamedDllPath)) {
        std::cerr << "Error adding Liaison Dynamic Library file to FMU." << std::endl;
        zip_discard(fmu);
        return;
    }

    // Add the modelDescription.xml file to the FMU at the base directory
    std::string modelDescriptionPath = tempPath + "/modelDescription.xml";
    if (!addFileToZip(fmu, modelDescriptionPath, "modelDescription.xml")) {
        std::cerr << "Error adding modelDescription.xml file to FMU." << std::endl;
        zip_discard(fmu);
        return;
    }

    // Add the responderId.txt fiel to the FMU at the base directory
    std::string responderIdPath = createResponderIdFile(tempPath, responderId);
    if (!addFileToZip(fmu, responderIdPath, "binaries/responderId")) {
        std::cerr << "Error copyng the responderId file to FMU." << std::endl;
        zip_discard(fmu);
        return;
    }

    // Close the zip archive
    if (zip_close(fmu) < 0) {
        std::cerr << "Failed to finalize FMU zip archive" << std::endl;
        return;
    }

    std::cout << "FMU successfully created! "  << std::endl;
}


void printUsage() {
    std::cout << "Usage:\n";
    std::cout << "  liaison --server <Path to FMU> <Responder Id>\n";
    std::cout << "  liaison --make-fmu <Path to FMU> <Responder Id>\n";
}

int main(int argc, char* argv[]) {

    if (argc != 4) {
        printUsage();
        return 1;
    }

    std::string option = argv[1];
    std::string fmuPath = argv[2];
    std::string responderId = argv[3];

    try {
        if (option == "--server") {
            startServer(fmuPath, responderId);
        } else if (option == "--make-fmu") {
            generateFmu(fmuPath, responderId);
        } else {
            printUsage();
            return 1;
        }
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;

}