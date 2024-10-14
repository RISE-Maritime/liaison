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

set(CMAKE_FIND_LIBRARY_SUFFIXES .a)

# Include Protobuf
find_package(Protobuf REQUIRED STATIC)
include_directories(${PROTOBUF_INCLUDE_DIR})

# Incluce libzip and zlib
find_package(PkgConfig REQUIRED)
pkg_check_modules(libzip REQUIRED libzip)
pkg_check_modules(zlib REQUIRED zlib)
include_directories(${libzip_INCLUDE_DIRS})
include_directories(${zlib_INCLUDE_DIRS})

# Include directories with header files
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}                   # For generated protobuf files
    ${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3   # For FMI headers
)

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison executable
add_executable(liaison src/liaison.cpp src/utils.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison PRIVATE
    zenohcxx::zenohc
    ${Protobuf_LIBRARIES}
    ${libzip_LIBRARIES}
    ${zlib_LIBRARIES}
    -static-libgcc
    -static-libstdc++
)

# Liaison shared library
set(LIAISON_OUTPUT_DIR ${CMAKE_BINARY_DIR}/binaries/x86_64-linux)
file(MAKE_DIRECTORY ${LIAISON_OUTPUT_DIR})
add_library(liaisonfmu SHARED src/fmi3Functions.cpp ${PROTO_SRCS} ${PROTO_HDRS})
set_target_properties(liaisonfmu PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    LIBRARY_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
    ARCHIVE_OUTPUT_DIRECTORY ${LIAISON_OUTPUT_DIR}
)
target_link_libraries(liaisonfmu PRIVATE
    zenohcxx::zenohc
    ${Protobuf_LIBRARIES}
)

# message(STATUS "libzip include dirs: ${libzip_INCLUDE_DIRS}")
# message(STATUS "libzip libraries: ${libzip_LIBRARIES}")


# # Include Protobuf
# find_package(Protobuf REQUIRED)
# set(Protobuf_INCLUDE_DIR "/usr/local/include")
# set(Protobuf_LIBRARIES "/usr/local/lib/libprotobuf.a")

# # Include libzip
# set(libz_INCLUDE_DIR "/usr/include")
# set(libz_LIBRARIES "/lib/x86_64-linux-gnu/libzip.a")

# # Include zlib
# set(zlib_INCLUDE_DIR "/usr/include")
# set(zlib_LIBRARIES "/lib/x86_64-linux-gnu/libz.a")

# # Use linker options to enforce static linking for specific libraries
# target_link_options(liaison PRIVATE
#     "-Wl,-Bstatic"
#     # List the static libraries
#     ${zenohcxx_LIBRARIES}
#     ${zenohc_LIBRARIES}
#     ${Protobuf_LIBRARIES}
#     # libzip is already a target; no need to specify here
#     "-Wl,-Bdynamic"
#     # List dynamic libraries
#     -lz
#     -lpthread
#     -ldl
# )

# # Liaison shared library
# add_library(liaisonfmu SHARED src/fmi3Functions.cpp ${PROTO_SRCS} ${PROTO_HDRS})
# target_link_libraries(liaisonfmu PRIVATE
#     # Enforce static linking for these libraries
#     ${zenohcxx_LIBRARIES}
#     ${zenohc_LIBRARIES}
#     ${Protobuf_LIBRARIES}
#     zip
#     # Dynamically link system libraries
#     z
#     pthread
#     dl
# )
# target_link_options(liaisonfmu PRIVATE
#     "-Wl,-Bstatic"
#     ${zenohcxx_LIBRARIES}
#     ${zenohc_LIBRARIES}
#     ${Protobuf_LIBRARIES}
#     # libzip is already a target; no need to specify here
#     "-Wl,-Bdynamic"
#     -lz
#     -lpthread
#     -ldl
# )

# # Test executable
# add_executable(test_liaison_fmu src/test_liaison_fmu.cpp)
# target_link_libraries(test_liaison_fmu PRIVATE liaisonfmu)
# add_dependencies(test_liaison_fmu liaisonfmu)