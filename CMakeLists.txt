cmake_minimum_required(VERSION 3.22 FATAL_ERROR)

# Set vcpkg target triplet
if (WIN32)
    set(VCPKG_TARGET_TRIPLET x64-windows-static)
endif()

project(Liaison LANGUAGES C CXX)

option(ENABLE_SANITIZERS "Enable sanitizers in debug builds" OFF)

if(ENABLE_SANITIZERS)
    if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
        # Add sanitizer flags for gcc/clang
        set(SANITIZER_FLAGS "-fsanitize=address -fsanitize=undefined -fno-omit-frame-pointer")
        set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} ${SANITIZER_FLAGS}")
        set(CMAKE_LINKER_FLAGS_DEBUG "${CMAKE_LINKER_FLAGS_DEBUG} ${SANITIZER_FLAGS}")
        message(STATUS "Sanitizers enabled in debug build")
    else()
        message(WARNING "Sanitizers requested but not supported with current compiler")
    endif()
endif()

if(CMAKE_BUILD_TYPE STREQUAL "")
    set(CMAKE_BUILD_TYPE Release)
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_POSITION_INDEPENDENT_CODE ON) 

# Ensure static libraries are used
set(BUILD_SHARED_LIBS OFF)

# Set static runtime library for MSVC
if (MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
endif()

# Include Zenoh-c and Zenoh-cpp (find static libraries)
find_package(zenohc REQUIRED)
find_package(zenohcxx REQUIRED)

# Include Protobuf
set(Protobuf_USE_STATIC_LIBS ON)
find_package(Protobuf REQUIRED)

# Include libzip and zlib
if (WIN32)
    find_package(libzip CONFIG REQUIRED)
    find_package(ZLIB REQUIRED)
else()
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(libzip REQUIRED libzip)
    pkg_check_modules(zlib REQUIRED zlib)
    include_directories(${libzip_INCLUDE_DIRS})
    include_directories(${zlib_INCLUDE_DIRS})
endif()

# Include spdlog
set(SPDLOG_STATIC_LIB ON)
find_package(spdlog REQUIRED)

# Include nlohmann/json¨
include(FetchContent)
FetchContent_Declare(json URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz)
FetchContent_MakeAvailable(json)


# Include directories with header files
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}                   # For generated protobuf files
    ${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3   # For FMI headers
)

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison executable
add_executable(liaison src/liaison.cpp src/utils.cpp ${PROTO_SRCS} ${PROTO_HDRS})
if (WIN32)
    target_link_libraries(liaison PRIVATE
        zenohcxx::zenohc
        protobuf::libprotobuf
        spdlog::spdlog
        libzip::zip
        ZLIB::ZLIB
        nlohmann_json::nlohmann_json
    )
else()
    target_link_libraries(liaison PRIVATE
        zenohcxx::zenohc
        protobuf::libprotobuf
        spdlog::spdlog
        nlohmann_json::nlohmann_json
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
# List of configurations
set(CONFIGURATIONS Debug Release RelWithDebInfo MinSizeRel)

# Set output directories for each configuration
foreach(CONFIGURATION ${CONFIGURATIONS})
    set(CONFIGURATION_UPPER ${CONFIGURATION})
    string(TOUPPER ${CONFIGURATION_UPPER} CONFIGURATION_UPPER)
    set_target_properties(liaisonfmu PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY_${CONFIGURATION_UPPER} "${LIAISON_OUTPUT_DIR}"
        LIBRARY_OUTPUT_DIRECTORY_${CONFIGURATION_UPPER} "${LIAISON_OUTPUT_DIR}"
        ARCHIVE_OUTPUT_DIRECTORY_${CONFIGURATION_UPPER} "${LIAISON_OUTPUT_DIR}"
    )
endforeach()

target_link_libraries(liaisonfmu PRIVATE
    zenohcxx::zenohc
    protobuf::libprotobuf
    nlohmann_json::nlohmann_json
)

add_executable(simple_fmu_runner src/simple_fmu_runner.cpp src/utils.cpp)

if (WIN32)
    target_link_libraries(simple_fmu_runner PRIVATE
        libzip::zip
        ZLIB::ZLIB
    )
else()
    target_link_libraries(simple_fmu_runner PRIVATE
        ${libzip_LIBRARIES}
        ${zlib_LIBRARIES}
        -static-libgcc
        -static-libstdc++
    )
endif()
