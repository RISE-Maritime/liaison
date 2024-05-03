#include <iostream>
#include <stdexcept>
#include <optional>
#include <variant>
#include <future>
#include "zenoh.hxx"
#include "zenohc.hxx"
#include "fmi3.pb.h"


std::unique_ptr<zenoh::Session> z_client;

// zenoh::Sample queryZenoh(std::string keystr) {

//     std::promise<zenoh::Sample> promise;
//     std::future<zenoh::Sample> future = promise.get_future();

//     zenoh::GetOptions opts;
//     opts.set_value("foobar");

//     auto on_reply = [&promise](zenoh::Reply reply) {
//         auto result = reply.get();
//         if (auto sample = std::get_if<zenoh::Sample>(&result)) {
//             promise.set_value(*sample);
//             std::cout << "Instance number: " << sample->get_payload().as_string_view() << std::endl;
//         } else if (auto error = std::get_if<zenoh::ErrorMessage>(&result)) {
//             throw std::runtime_error(std::string(error->as_string_view()));
//         }
//     };
//     auto on_done = []() {
//         std::cout << "done!" << std::endl;
//     };
//     z_client->get(keystr,"foo=89&bar=12", {on_reply, on_done}, opts);
//     std::cout << "sending!" << std::endl;
//     zenoh::Sample sam = future.get();
//     std::cout << "Instance number 333: " << sam.get_payload().as_string_view() << std::endl;
//     return sam;
// }


// zenoh::Sample queryZenoh(std::string keystr) {

//     zenoh::GetOptions opts;
//     opts.set_value("foobar");

//     auto [send, recv] = zenohc::reply_fifo_new(16);
//     z_client->get(keystr,"foo=89&bar=12",std::move(send), opts);
//     zenoh::Reply reply(nullptr);
//     for (recv(reply); reply.check(); recv(reply)) {
//         auto sample = zenohc::expect<zenoh::Sample>(reply.get());
//         return sample;
//     }
    

// }

// std::optional<zenoh::Sample> queryZenoh(const std::string& keystr) {
//     zenoh::GetOptions opts;
//     opts.set_value("foobar"); // Set additional options as necessary

//     auto [send, recv] = zenohc::reply_fifo_new(16); // Create a FIFO for 16 elements
//     z_client->get(keystr, "foo=89&bar=12", std::move(send), opts); // Make the get request

//     zenoh::Reply reply(nullptr); // Initialize a zenoh::Reply object
//     while (recv(reply)) { // Use while loop to process received replies
//         if (reply.check()) { // Check if the reply is valid
//             auto sample = zenohc::expect<zenoh::Sample>(reply.get()); // Extract sample from reply
//             return sample;
//         }
//     }

//     return std::nullopt; // Return std::nullopt if no valid sample is received
//}

std::vector<zenohc::BytesView> queryZenoh(const std::string& keystr) {
    zenoh::GetOptions opts;
    opts.set_value("foobar"); // Set additional options as necessary

    auto [send, recv] = zenohc::reply_fifo_new(16); // Create a FIFO for 16 elements
    z_client->get(keystr, "foo=89&bar=12", std::move(send), opts); // Make the get request

    std::vector<zenohc::BytesView> payloads; // Vector to store received samples
    zenoh::Reply reply(nullptr); // Initialize a zenoh::Reply object

    while (recv(reply)) { // Use while loop to process received replies
        if (reply.check()) { // Check if the reply is valid
            auto sample = zenohc::expect<zenoh::Sample>(reply.get()); // Extract sample from reply
            payloads.push_back(sample.get_payload()); // Add the valid sample to the vector
            
        } else {
            break; // Exit loop if no more valid replies are expected
        }
    }

    return payloads; // Return the vector of samples
}


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


void queryA() {
    
    auto payloads = queryZenoh("rpc/demo/query");
    std::cout << " samples size " << payloads.size()  << std::endl;
    for (const auto& payload : payloads) {
        std::cout << "start "  << std::endl;
        proto::Instance instance;
        instance.ParseFromArray(payload.start, payload.len);
        std::cout << "Instance number: " << instance.key() << std::endl; 
    }
}


// void queryB() {
//     std::string keystr("rpc/demo/query2");
//     auto result = queryZenoh(keystr);
//     if (result) {
//         proto::Instance instance;
//         instance.ParseFromArray(result.value().payload.start, result.value().payload.len);
//         std::cout << "Instance number: " << instance.key() << std::endl;

//     } else {
//         std::cout << "Query B: No response " << std::endl;
//     }
    

// }

int main(int argc, char **argv) {

    startZenoh();
    queryZenoh("rpc/demo/query");

    // std::string keystr = "rpc/demo/query2";
    // auto sample = queryZenoh(keystr);
    
    // std::cout << sample.get_payload().as_string_view() << " foo" << std::endl;
    queryA();
    // queryB();
    z_client.reset();

}