#include <iostream>
#include <stdexcept>
#include <optional>
#include <variant>
#include <future>
#include "zenoh.hxx"
#include "zenohc.hxx"
#include "fmi3.pb.h"


std::unique_ptr<zenoh::Session> z_client;
std::string responder_id = "demo";


// This macro assumes that:
//  1.  the 'input' and 'output' of protobuf messages
// are declared
#define ZENOH_FMI3_QUERY(FMI3_FUNCTION) \
    size_t input_size = input.ByteSizeLong(); \
    std::vector<uint8_t> buffer(input_size); \
    input.SerializeToArray(buffer.data(), input_size); \
    std::string expr = "rpc/" + responder_id + "/" + FMI3_FUNCTION; \
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
        std::cerr << "Failed zenoh query for keyexpr: "<< expr << std::endl; \
    } \
    




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

int main(int argc, char **argv) {

    startZenoh();
    
    proto::Empty input;
    input.set_value(333);
    proto::fmi3InstanceMessage output;
    ZENOH_FMI3_QUERY("fmi3InstantiateCoSimulation")

    std::cout << "Instance number: " << output.instance() << std::endl; 
   
    z_client.reset();

}