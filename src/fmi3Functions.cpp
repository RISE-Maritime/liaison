#include <stdexcept>
#include "fmi3.pb.h"
#include "fmi3Functions.h"

#define TRY_CODE(FUNCTION_CALL, ERROR_MSG, ERROR_RETURN) \
    try {                                                          \
        FUNCTION_CALL                                             \
    } catch (const std::exception &e) {                            \
        std::cerr << ERROR_MSG << e.what() << std::endl;           \
        return ERROR_RETURN;                                       \
    }


// Zenoh client
std::unique_ptr<Session> session;

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


void queryZenoh(zenoh::net::Session& session, const std::string& query) {
    try {
        auto syncGet = session.get(query);
        auto responses = syncGet.wait();
        // Iterate through and process each response
        for (auto &response : responses) {
            if (response.kind() == zenoh::net::GET) {
                std::cout << "Received response: " << response.payload().value().to_string() << std::endl;
            } else {
                std::cerr << "Received non-get response" << std::endl;
            }
        }
    } catch (const std::exception& e) {
        std::cerr << "Exception during zenoh get: " << e.what() << std::endl;
    }
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
    

    Config config;
    session = except<Session>(open(std::move(config)));
    session.get("rpc/demo/query")

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

}