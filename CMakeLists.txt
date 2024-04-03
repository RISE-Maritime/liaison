cmake_minimum_required(VERSION 3.14 FATAL_ERROR)
project(Liaison)

set(CMAKE_CXX_STANDARD 17)

include(FetchContent)

FetchContent_declare(zenohpico_backend GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-pico" GIT_TAG release-0.10.1-rc)
FetchContent_MakeAvailable(zenohpico_backend)
FetchContent_declare(cpp_wrapper GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-cpp" GIT_TAG release-0.10.1-rc)
FetchContent_MakeAvailable(cpp_wrapper)

add_executable(liaison src/zenoh.cpp)
target_link_libraries(liaison PRIVATE zenohcxx::zenohpico)
