#include <iostream>
#include <stdexcept>
#include <variant>
#include <future>
#include "zenoh.hxx"
//#include "zenohc.hxx"
#include "fmi3.pb.h"


std::unique_ptr<zenoh::Session> z_client;

zenoh::Sample queryZenoh(std::string keystr) {

    std::promise<zenoh::Sample> promise;
    std::future<zenoh::Sample> future = promise.get_future();

    zenoh::GetOptions opts;
    opts.set_value("foobar");

    auto on_reply = [&promise](zenoh::Reply reply) {
        auto result = reply.get();
        if (auto sample = std::get_if<zenoh::Sample>(&result)) {
            promise.set_value(*sample);
            std::cout << "Instance number: " << sample->get_payload().as_string_view() << std::endl;
        } else if (auto error = std::get_if<zenoh::ErrorMessage>(&result)) {
            throw std::runtime_error(std::string(error->as_string_view()));
        }
    };
    auto on_done = []() {
        std::cout << "done!" << std::endl;
    };
    z_client->get(keystr,"foo=89&bar=12", {on_reply, on_done}, opts);
    std::cout << "sending!" << std::endl;
    zenoh::Sample sam = future.get();
    std::cout << "Instance number 333: " << sam.get_payload().as_string_view() << std::endl;
    return sam;
}


// zenoh::Sample queryZenoh(std::string keystr) {

//     zenoh::GetOptions opts;
//     opts.set_value("foobar");

//     auto [send, recv] = zenohc::reply_fifo_new(16);
//     z_client->get(keystr,"foo=89&bar=12",std::move(send), opts);
//     zenoh::Reply reply(nullptr);
//     for (recv(reply); reply.check(); recv(reply)) {
//         auto sample = expect<zenoh::Sample>(reply.get());
//     }

// }





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
    
    auto sample = queryZenoh("rpc/demo/query");
    std::cout << "Query A: " << sample.get_payload().as_string_view() << std::endl;

}


void queryB() {
    std::string keystr("rpc/demo/query2");
    auto sample = queryZenoh(keystr);
    proto::Instance instance;
    instance.ParseFromArray(sample.payload.start, sample.payload.len);
    std::cout << "Instance number: " << instance.key() << std::endl;

}

int main(int argc, char **argv) {

    startZenoh();

    // std::string keystr = "rpc/demo/query2";
    // auto sample = queryZenoh(keystr);
    
    // std::cout << sample.get_payload().as_string_view() << " foo" << std::endl;
    queryA();
    //queryB();

}