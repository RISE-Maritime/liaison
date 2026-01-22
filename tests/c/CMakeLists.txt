cmake_minimum_required(VERSION 3.22 FATAL_ERROR)

project(LiaisonTests LANGUAGES C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Fetch CMocka testing framework
include(FetchContent)
FetchContent_Declare(
    cmocka
    GIT_REPOSITORY https://gitlab.com/cmocka/cmocka.git
    GIT_TAG cmocka-1.1.7
)
set(WITH_EXAMPLES OFF CACHE BOOL "" FORCE)
set(UNIT_TESTING OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(cmocka)

# Include FMI3 headers
include_directories(${CMAKE_SOURCE_DIR}/external/fmi3)

# Include nlohmann/json for call log serialization
include_directories(${json_SOURCE_DIR}/include)

# =============================================================================
# DummyFMU - Mock FMU that records all calls
# =============================================================================
set(DUMMY_FMU_OUTPUT_DIR ${CMAKE_BINARY_DIR}/test_fmu/binaries/x86_64-linux)
file(MAKE_DIRECTORY ${DUMMY_FMU_OUTPUT_DIR})

add_library(dummyfmu SHARED
    dummy_fmu/call_record.c
    dummy_fmu/fmi3Functions.c
)
target_include_directories(dummyfmu PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dummy_fmu
)
set_target_properties(dummyfmu PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY ${DUMMY_FMU_OUTPUT_DIR}
    OUTPUT_NAME "DummyFMU"
    PREFIX ""  # Remove 'lib' prefix
)

# =============================================================================
# Test Harness - Server management and FMU loading
# =============================================================================
add_library(test_harness STATIC
    test_harness/test_harness.c
)
target_include_directories(test_harness PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/test_harness
    ${CMAKE_CURRENT_SOURCE_DIR}/dummy_fmu
)
target_link_libraries(test_harness PRIVATE
    ${CMAKE_DL_LIBS}
)

# =============================================================================
# Test Executables
# =============================================================================
enable_testing()

# Macro to create test executables
macro(add_liaison_test TEST_NAME)
    add_executable(${TEST_NAME} tests/${TEST_NAME}.c)
    target_include_directories(${TEST_NAME} PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/test_harness
        ${CMAKE_CURRENT_SOURCE_DIR}/dummy_fmu
        ${cmocka_SOURCE_DIR}/include
    )
    target_link_libraries(${TEST_NAME} PRIVATE
        test_harness
        cmocka
        ${CMAKE_DL_LIBS}
    )
    add_test(NAME ${TEST_NAME} COMMAND ${TEST_NAME})

    # Set environment variables for tests
    set_tests_properties(${TEST_NAME} PROPERTIES
        ENVIRONMENT "LIAISON_BINARY=${CMAKE_BINARY_DIR}/liaison;LIAISON_FMU_LIB=${CMAKE_BINARY_DIR}/binaries/x86_64-linux/libliaisonfmu.so;DUMMY_FMU_LIB=${DUMMY_FMU_OUTPUT_DIR}/DummyFMU.so;DUMMY_FMU_PATH=${CMAKE_BINARY_DIR}/test_fmu/DummyFMU.fmu"
    )
endmacro()

# Add test targets
add_liaison_test(test_instantiation)
add_liaison_test(test_initialization)
add_liaison_test(test_float64_values)
add_liaison_test(test_integer_values)
add_liaison_test(test_boolean_values)
add_liaison_test(test_string_values)
add_liaison_test(test_binary_values)
add_liaison_test(test_clock_values)
add_liaison_test(test_do_step)
add_liaison_test(test_state_management)

# =============================================================================
# Create DummyFMU archive
# =============================================================================
# Create a minimal modelDescription.xml for the dummy FMU
set(TEST_FMU_STAGING_DIR ${CMAKE_BINARY_DIR}/test_fmu_staging)
file(MAKE_DIRECTORY ${TEST_FMU_STAGING_DIR})
file(MAKE_DIRECTORY ${TEST_FMU_STAGING_DIR}/binaries/x86_64-linux)
file(MAKE_DIRECTORY ${TEST_FMU_STAGING_DIR}/resources)

file(WRITE ${TEST_FMU_STAGING_DIR}/modelDescription.xml
"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<fmiModelDescription
    fmiVersion=\"3.0\"
    modelName=\"DummyFMU\"
    instantiationToken=\"{dummy-fmu-token}\"
    generationTool=\"LiaisonTests\"
    generationDateAndTime=\"2024-01-01T00:00:00Z\">
    <CoSimulation modelIdentifier=\"dummyfmu\"/>
    <ModelExchange modelIdentifier=\"dummyfmu\"/>
    <ScheduledExecution modelIdentifier=\"dummyfmu\"/>
    <ModelVariables>
        <Float64 name=\"x\" valueReference=\"0\" causality=\"output\" variability=\"continuous\"/>
        <Float64 name=\"v\" valueReference=\"1\" causality=\"output\" variability=\"continuous\"/>
        <Float64 name=\"g\" valueReference=\"2\" causality=\"parameter\" variability=\"fixed\"/>
        <Int32 name=\"intVar\" valueReference=\"3\" causality=\"local\" variability=\"discrete\"/>
        <Boolean name=\"boolVar\" valueReference=\"4\" causality=\"local\" variability=\"discrete\"/>
        <String name=\"stringVar\" valueReference=\"5\" causality=\"local\" variability=\"fixed\"/>
    </ModelVariables>
    <ModelStructure/>
</fmiModelDescription>")

# Custom command to create the FMU zip after building dummyfmu
add_custom_command(
    TARGET dummyfmu POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:dummyfmu> ${TEST_FMU_STAGING_DIR}/binaries/x86_64-linux/
    COMMAND ${CMAKE_COMMAND} -E chdir ${TEST_FMU_STAGING_DIR} ${CMAKE_COMMAND} -E tar cf ${CMAKE_BINARY_DIR}/test_fmu/DummyFMU.fmu --format=zip modelDescription.xml binaries resources
    COMMENT "Creating DummyFMU.fmu test archive"
)

# Create test_fmu directory for the final FMU
file(MAKE_DIRECTORY ${CMAKE_BINARY_DIR}/test_fmu)
