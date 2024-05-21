cmake_minimum_required(VERSION 3.22 FATAL_ERROR)
project(Liaison)

set(CMAKE_CXX_STANDARD 17)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Statically link dependencies
set(Zenohc_STATIC_LIB "/usr/local/lib/libzenohc.a")
set(Protobuf_STATIC_LIB "/usr/local/lib/libprotobuf.a")

# Include Zenoh-Cpp
include(FetchContent)
find_package(zenohc) # Zenoh-c, as well as rust, is installed in the devcontainer 
FetchContent_declare(zenohcpp GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-cpp" GIT_TAG release-0.10.1-rc)
FetchContent_MakeAvailable(zenohcpp)

# Include Protobuf
find_package(Protobuf REQUIRED)
message(STATUS "Using protobuf ${Protobuf_VERSION}")


# Incluce libzip
find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBZIP REQUIRED libzip)
include_directories(${LIBZIP_INCLUDE_DIRS})
link_directories(${LIBZIP_LIBRARY_DIRS})

# Generate protobuf files
include_directories(${PROTOBUF_INCLUDE_DIR})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3")


# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison Client
add_executable(liaison_client src/liaison_client.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison_client PRIVATE zenohcxx::zenohc::lib ${Zenohc_STATIC_LIB} ${Protobuf_STATIC_LIB})

# Liaison Server
add_executable(liaison_server src/liaison_server.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison_server PRIVATE zenohcxx::zenohc::lib ${Zenohc_STATIC_LIB} ${Protobuf_STATIC_LIB})

# Liaison FMU
add_library(liaison SHARED src/fmi3Functions.cpp)
target_link_libraries(liaison PRIVATE zenohcxx::zenohc::lib ${Zenohc_STATIC_LIB} ${Protobuf_STATIC_LIB} )


# fmi3Server 
add_executable(fmi3Server src/fmi3Server.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(fmi3Server PRIVATE
    zenohcxx::zenohc::lib 
    ${Zenohc_STATIC_LIB} 
    ${Protobuf_STATIC_LIB}
    ${LIBZIP_LIBRARIES}
)