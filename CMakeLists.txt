cmake_minimum_required(VERSION 3.22 FATAL_ERROR)
project(Liaison LANGUAGES C CXX)

if(CMAKE_BUILD_TYPE STREQUAL "")
    set(CMAKE_BUILD_TYPE Release)
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Prefer static libraries
set(CMAKE_FIND_LIBRARY_SUFFIXES .a)
set(BUILD_SHARED_LIBS OFF)

# Include Zenoh-c and Zenoh-cpp
find_package(zenohc REQUIRED STATIC)
find_package(zenohcxx REQUIRED STATIC)

# Include Protobuf
find_package(Protobuf REQUIRED STATIC)
include_directories(${PROTOBUF_INCLUDE_DIR})

# Incluce libzip
find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBZIP REQUIRED libzip)
include_directories(${LIBZIP_INCLUDE_DIRS})
link_directories(${LIBZIP_LIBRARY_DIRS})

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Include directories with header files
include_directories(${CMAKE_CURRENT_BINARY_DIR}) # for gen. protobuf files
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3")

# Liaison 
add_executable(liaison src/liaison.cpp src/utils.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison PRIVATE
    zenohcxx::zenohc
    ${PROTOBUF_LIBRARIES}
    ${LIBZIP_LIBRARIES}
    -static-libgcc
    -static-libstdc++
    -lpthread
    -ldl
)

# Liaison Lib
set(LIAISON_OUTPUT_DIR ${CMAKE_BINARY_DIR}/binaries/x86_64-linux)
file(MAKE_DIRECTORY ${LIAISON_OUTPUT_DIR})
add_library(liaisonfmu SHARED src/fmi3Functions.cpp ${PROTO_SRCS} ${PROTO_HDRS})
set_target_properties(liaisonfmu PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    LIBRARY_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    ARCHIVE_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
)
target_link_libraries(liaisonfmu PRIVATE zenohcxx::zenohc ${Zenohc_STATIC_LIB} ${Protobuf_STATIC_LIB})

# add_executable(z_get src/z_get.cxx)
# include_directories("${CMAKE_SOURCE_DIR}/src")
# target_link_libraries(z_get PRIVATE zenohcxx::zenohc)
# set_property(TARGET z_get PROPERTY LANGUAGE CXX)
# set_property(TARGET z_get PROPERTY CXX_STANDARD 17)

# add_executable(z_queryable src/z_queryable.cxx)
# include_directories("${CMAKE_SOURCE_DIR}/src")
# target_link_libraries(z_queryable PRIVATE zenohcxx::zenohc)
# set_property(TARGET z_queryable PROPERTY LANGUAGE CXX)
# set_property(TARGET z_queryable PROPERTY CXX_STANDARD 17)


