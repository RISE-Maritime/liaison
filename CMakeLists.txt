
cmake_minimum_required(VERSION 3.11)

project(Liaison VERSION 0.1.0)

if(MSVC)
  add_definitions(-D_WIN32_WINNT=0x600)
endif()

# Dependencies

include(FetchContent)

FetchContent_Declare(
  gRPC
  GIT_REPOSITORY https://github.com/grpc/grpc
  GIT_TAG        v1.28.0
  )
set(FETCHCONTENT_QUIET OFF)
FetchContent_MakeAvailable(gRPC)

set(_PROTOBUF_LIBPROTOBUF libprotobuf)
set(_REFLECTION grpc++_reflection)
set(_PROTOBUF_PROTOC $<TARGET_FILE:protoc>)
set(_GRPC_GRPCPP grpc++)
if(CMAKE_CROSSCOMPILING)
  find_program(_GRPC_CPP_PLUGIN_EXECUTABLE grpc_cpp_plugin)
else()
  set(_GRPC_CPP_PLUGIN_EXECUTABLE $<TARGET_FILE:grpc_cpp_plugin>)
endif()


# Proto file
get_filename_component(fmi3_proto "./fmi3.proto" ABSOLUTE)
get_filename_component(fmi3_proto_path "${fmi3_proto}" PATH)

# Generated sources
set(fmi3_proto_srcs "${CMAKE_CURRENT_BINARY_DIR}/fmi3.pb.cc")
set(fmi3_proto_hdrs "${CMAKE_CURRENT_BINARY_DIR}/fmi3.pb.h")
set(fmi3_grpc_srcs "${CMAKE_CURRENT_BINARY_DIR}/fmi3.grpc.pb.cc")
set(fmi3_grpc_hdrs "${CMAKE_CURRENT_BINARY_DIR}/fmi3.grpc.pb.h")
add_custom_command(
      OUTPUT "${fmi3_proto_srcs}" "${fmi3_proto_hdrs}" "${fmi3_grpc_srcs}" "${fmi3_grpc_hdrs}"
      COMMAND ${_PROTOBUF_PROTOC}
      ARGS --grpc_out "${CMAKE_CURRENT_BINARY_DIR}"
        --cpp_out "${CMAKE_CURRENT_BINARY_DIR}"
        -I "${fmi3_proto_path}"
        --plugin=protoc-gen-grpc="${_GRPC_CPP_PLUGIN_EXECUTABLE}"
        "${fmi3_proto}"
      DEPENDS "${fmi3_proto}")

include_directories("${CMAKE_CURRENT_BINARY_DIR}")

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3")

## Targets

# fmi3_grpc_proto
add_library(fmi3_grpc_proto
  ${fmi3_grpc_srcs}
  ${fmi3_grpc_hdrs}
  ${fmi3_proto_srcs}
  ${fmi3_proto_hdrs})
target_link_libraries(fmi3_grpc_proto
  ${_REFLECTION}
  ${_GRPC_GRPCPP}
  ${_PROTOBUF_LIBPROTOBUF})

# liaison
add_library(liaison SHARED src/fmi3Functions.cpp)
target_link_libraries(liaison
    fmi3_grpc_proto
    absl::flags
    absl::flags_parse
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    ${_PROTOBUF_LIBPROTOBUF})

