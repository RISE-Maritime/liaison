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
include_directories(${PROTOBUF_INCLUDE_DIR})

# Incluce libzip
find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBZIP REQUIRED libzip)
include_directories(${LIBZIP_INCLUDE_DIRS})
link_directories(${LIBZIP_LIBRARY_DIRS})

# Include directories with header files
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3")

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison Lib
set(LIAISON_OUTPUT_DIR ${CMAKE_BINARY_DIR}/binaries/x86_64-linux)
file(MAKE_DIRECTORY ${LIAISON_OUTPUT_DIR})
add_library(liaisonfmu SHARED src/fmi3Functions.cpp ${PROTO_SRCS} ${PROTO_HDRS})
set_target_properties(liaisonfmu PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    LIBRARY_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    ARCHIVE_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
)
target_link_libraries(liaisonfmu PRIVATE zenohcxx::zenohc::lib ${Zenohc_STATIC_LIB} ${Protobuf_STATIC_LIB})

# Liaison 
add_executable(liaison src/liaison.cpp src/utils.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison PRIVATE
    zenohcxx::zenohc::lib 
    ${Zenohc_STATIC_LIB} 
    ${Protobuf_STATIC_LIB}
    ${LIBZIP_LIBRARIES}
)