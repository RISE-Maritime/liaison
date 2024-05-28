cmake_minimum_required(VERSION 3.22 FATAL_ERROR)
project(PortalFMI)

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

include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/external/fmi3")

# Generate protobuf files
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS src/fmi3.proto)

# Liaison FMU
add_library(liaison SHARED src/fmi3Functions.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(liaison PRIVATE zenohcxx::zenohc::lib ${Zenohc_STATIC_LIB} ${Protobuf_STATIC_LIB} )

# PortalServer 
add_executable(portal-server src/portalServer.cpp src/fmi3Logging.cpp ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(portal-server PRIVATE
    zenohcxx::zenohc::lib 
    ${Zenohc_STATIC_LIB} 
    ${Protobuf_STATIC_LIB}
    ${LIBZIP_LIBRARIES}
)