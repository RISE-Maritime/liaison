#include <stdio.h>
#include <string.h>
#include <unordered_map>
#include <unistd.h>
#include "zenoh.hxx"
#include "fmi3.pb.h"

#define sleep(x) usleep(x)

void handleQueryA(const zenoh::Query& query) {
    auto keystr = query.get_keyexpr();
    auto pred = query.get_parameters();
    auto query_value = query.get_value();
    std::cout << ">> Received Query '" << keystr.as_string_view() << "?" << pred.as_string_view()
                << "' value = '" << query_value.as_string_view() << "'\n";
    zenoh::QueryReplyOptions options;
    options.set_encoding(zenoh::Encoding(Z_ENCODING_PREFIX_TEXT_PLAIN));
    query.reply(keystr, "Queryable from C++ zenoh-c!", options);
}

void handleQueryB(const zenoh::Query& query) {
    std::cout << "Query B" << std::endl;
    proto::Instance instance;
    instance.set_key(133);

    size_t instance_size = instance.ByteSizeLong();
    std::vector<uint8_t> buffer(instance_size);
    instance.SerializeToArray(buffer.data(), instance_size);


    zenoh::QueryReplyOptions options;
    options.set_encoding(zenoh::Encoding(Z_ENCODING_PREFIX_APP_CUSTOM));
    query.reply(query.get_keyexpr(),buffer);

}


int main() {

    zenoh::Config config;
    printf("Opening session...\n");
    auto z_server = zenoh::expect<zenoh::Session>(zenoh::open(std::move(config)));

    // Queryable declarations
    zenoh::KeyExprView keyexpr("rpc/demo/query");
    auto queryable_1 = zenoh::expect<zenoh::Queryable>(z_server.declare_queryable(keyexpr, handleQueryA));

    zenoh::KeyExprView keyexpr3("rpc/demo/query2");
    auto queryable_2 = zenoh::expect<zenoh::Queryable>(z_server.declare_queryable(keyexpr3, handleQueryB));
    
    printf("Enter 'q' to quit...\n");
    int c = 0;
    while (c != 'q') {
        c = getchar();
        if (c == -1) {
            sleep(1);
        }
    }

    return 0;
}

