cmake_minimum_required(VERSION 3.22 FATAL_ERROR)
project(Liaison)

set(CMAKE_CXX_STANDARD 17)

# Include Zenoh-Cpp
include(FetchContent)
FetchContent_declare(zenohpico_backend GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-pico" GIT_TAG release-0.10.1-rc)
FetchContent_MakeAvailable(zenohpico_backend)
FetchContent_declare(cpp_wrapper GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-cpp" GIT_TAG release-0.10.1-rc)
FetchContent_MakeAvailable(cpp_wrapper)

# Include Protobuf
find_package(Protobuf REQUIRED)
include_directories(${PROTOBUF_INCLUDE_DIR})
include_directories(${CMAKE_CURRENT_BINARY_DIR})

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison Client
add_executable(liaison_client src/liaison_client.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison_client PRIVATE zenohcxx::zenohpico ${Protobuf_LIBRARIES})

# Liaison Server
add_executable(liaison_server src/liaison_server.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison_server PRIVATE zenohcxx::zenohpico ${Protobuf_LIBRARIES})