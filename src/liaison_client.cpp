#include <iostream>
#include <stdexcept>
#include <optional>
#include <variant>
#include <future>
#include "zenoh.hxx"
#include "zenohc.hxx"
#include "fmi3.pb.h"


std::unique_ptr<zenoh::Session> z_client;


std::vector<zenohc::Sample> extractSamples(zenoh::ClosureReplyChannelRecv& recv) {
    std::vector<zenohc::Sample> samples; // Vector to store received samples
    zenoh::Reply reply(nullptr); // Initialize a zenoh::Reply object

    while (recv(reply)) { // Use while loop to process received replies
        if (reply.check()) { // Check if the reply is valid
            auto sample = zenohc::expect<zenoh::Sample>(reply.get()); // Extract sample from reply
            samples.push_back(sample); // Add the valid sample to the vector
        } else {
            break; // Exit loop if no more valid replies are expected
        }
    }

    return samples; // Return the vector of samples
}


// Function with 'value'
std::vector<zenohc::Sample> queryZenoh(
    const std::string& keystr,
    const char* params,  
    const zenohc::Value& value,
    int n_replies = 10
) {
    auto [send, recv] = zenohc::reply_fifo_new(n_replies);
    zenoh::GetOptions opts;
    opts.set_value(value); // Set the value into options

    z_client->get(keystr, params, std::move(send), opts); 

    return extractSamples(recv);
}

std::vector<zenohc::Sample> queryZenoh(
    const std::string& keystr,
    const char* params = "",  
    int n_replies = 10
) {
    auto [send, recv] = zenohc::reply_fifo_new(n_replies);
    zenoh::GetOptions opts; // Default options are used

    z_client->get(keystr, params, std::move(send), opts); 

    return extractSamples(recv);
}


// Helper function to extract samples from replies


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


void printSamples(std::vector<zenoh::Sample> samples) {
    
   
    std::cout << " samples size " << samples.size()  << std::endl;
    for (const auto& sample : samples) {
        std::cout << "start "  << std::endl;
        proto::Instance instance;
        instance.ParseFromArray(sample.payload.start, sample.payload.len);
        std::cout << "Instance number: " << instance.key() << std::endl; 
    }
}

int main(int argc, char **argv) {

    startZenoh();
    auto samples = queryZenoh("rpc/demo/query");
    printSamples(samples);
    std::string payload = "Hello Zenoh!";
    zenohc::Value value(payload); 
    // Key string for the Zenoh query
    std::string key = "rpc/demo/query";
    auto samples2 = queryZenoh(key,"foobar",value);
    printSamples(samples2);
   
    z_client.reset();

}