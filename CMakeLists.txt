cmake_minimum_required(VERSION 3.22 FATAL_ERROR)
project(Liaison LANGUAGES C CXX)

if(CMAKE_BUILD_TYPE STREQUAL "")
    set(CMAKE_BUILD_TYPE Release)
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_POSITION_INDEPENDENT_CODE ON) 

# Include Zenoh-c and Zenoh-cpp (find static libraries)
find_package(zenohc REQUIRED STATIC)
find_package(zenohcxx REQUIRED STATIC)

set(CMAKE_FIND_LIBRARY_SUFFIXES .lib .a)

# Set vcpkg target triplet
if (WIN32)
    set(VCPKG_TARGET_TRIPLET x64-windows-static)
endif()

# Include Protobuf
find_package(Protobuf REQUIRED STATIC)
include_directories(${PROTOBUF_INCLUDE_DIRS})

# Include libzip and zlib
if (WIN32)
    # set(CMAKE_PREFIX_PATH "${CMAKE_SOURCE_DIR}/vcpkg/installed/x64-windows-static")
    find_package(libzip CONFIG REQUIRED)
    find_package(zlib REQUIRED)
else()
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(libzip REQUIRED libzip)
    pkg_check_modules(zlib REQUIRED zlib)
endif()
include_directories(${libzip_INCLUDE_DIRS})
include_directories(${zlib_INCLUDE_DIRS})

# Include directories with header files
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}                   # For generated protobuf files
    ${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3   # For FMI headers
)

message(STATUS "libzip include dirs: ${libzip_INCLUDE_DIRS}")
message(STATUS "zlib include dirs: ${libzip_INCLUDE_DIRS}")
message(STATUS "Protobuf include dirs: ${PROTOBUF_INCLUDE_DIRS}")

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison executable
add_executable(liaison src/liaison.cpp src/utils.cpp ${PROTO_SRCS} ${PROTO_HDRS})
if (WIN32)
    target_link_libraries(liaison PRIVATE
        zenohcxx::zenohc
        ${Protobuf_LIBRARIES}
        ${libzip_LIBRARIES}
        ${zlib_LIBRARIES}
    )
else()
    target_link_libraries(liaison PRIVATE
        zenohcxx::zenohc
        ${Protobuf_LIBRARIES}
        ${libzip_LIBRARIES}
        ${zlib_LIBRARIES}
        -static-libgcc
        -static-libstdc++
    )
endif()

# Liaison shared library
if(WIN32)
    set(LIAISON_OUTPUT_DIR ${CMAKE_BINARY_DIR}/binaries/x86_64-windows)
else()
    set(LIAISON_OUTPUT_DIR ${CMAKE_BINARY_DIR}/binaries/x86_64-linux)
endif()

file(MAKE_DIRECTORY ${LIAISON_OUTPUT_DIR})
add_library(liaisonfmu SHARED src/fmi3Functions.cpp ${PROTO_SRCS} ${PROTO_HDRS})
set_target_properties(liaisonfmu PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    LIBRARY_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    ARCHIVE_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
)

if(WIN32)
    target_link_libraries(liaisonfmu PRIVATE
        zenohcxx::zenohc
        ${Protobuf_LIBRARIES}
        ws2_32
    )
else()
    target_link_libraries(liaisonfmu PRIVATE
        zenohcxx::zenohc
        ${Protobuf_LIBRARIES}
    )
endif() 