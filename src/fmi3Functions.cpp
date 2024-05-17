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

std::string responder_id = "demo";

// ZENOH 

std::unique_ptr<zenoh::Session> z_client;

void startZenoh(){
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
}


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
//      is declared and defined (e.g. proto::Instance input; 
//      input.set_key(1);)
//  2. A variable 'output' of a certain Protobuf messate type 
//     is declared (e.g. proto::Instance output;).
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
    
    
    startZenoh();
    proto::Empty input;
    proto::Instance output;
    ZENOH_FMI3_QUERY("fmi3InstantiateCoSimulation")
    FmuInstance* fmu_instance = new FmuInstance(output.key());
    return reinterpret_cast<fmi3Instance>(fmu_instance);
}

}