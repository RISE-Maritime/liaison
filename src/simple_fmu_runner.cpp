#include <iostream>
#include <string>
#include <filesystem>
#include <memory>
#include <dlfcn.h>
#include <sstream>
#include "fmi3Functions.h"
#include "zip.h"
#include "utils.hpp"

// Declare FMI function pointers with different names to avoid conflicts
static fmi3InstantiateCoSimulationTYPE* inst_fmi3InstantiateCoSimulation;
static fmi3EnterInitializationModeTYPE* inst_fmi3EnterInitializationMode;
static fmi3ExitInitializationModeTYPE* inst_fmi3ExitInitializationMode;
static fmi3DoStepTYPE* inst_fmi3DoStep;
static fmi3TerminateTYPE* inst_fmi3Terminate;
static fmi3FreeInstanceTYPE* inst_fmi3FreeInstance;

// Simple logger function
void logMessage(fmi3InstanceEnvironment instanceEnvironment, fmi3Status status, fmi3String category, fmi3String message) {
    std::cout << "[" << category << "] " << message << std::endl;
}

// Function to construct library path
std::string constructLibraryPath(const std::string& tempPath, const std::string& modelName) {
#ifdef _WIN32
    return tempPath + "/binaries/x86_64-windows/" + modelName + ".dll";
#else
    return tempPath + "/binaries/x86_64-linux/" + modelName + ".so";
#endif
}

// Function to load FMI functions
void loadFmiFunctions(void* handle) {
#ifdef _WIN32
    #define LOAD_FUNCTION(name) \
        inst_##name = (name##TYPE*)GetProcAddress((HMODULE)handle, #name)
#else
    #define LOAD_FUNCTION(name) \
        inst_##name = (name##TYPE*)dlsym(handle, #name)
#endif

    LOAD_FUNCTION(fmi3InstantiateCoSimulation);
    LOAD_FUNCTION(fmi3EnterInitializationMode);
    LOAD_FUNCTION(fmi3ExitInitializationMode);
    LOAD_FUNCTION(fmi3DoStep);
    LOAD_FUNCTION(fmi3Terminate);
    LOAD_FUNCTION(fmi3FreeInstance);

#undef LOAD_FUNCTION
}

// Function to load and unload FMU library (platform-specific)
#ifdef _WIN32
HMODULE loadFmuLibrary(const std::string& libPath) {
    HMODULE handle = LoadLibraryA(libPath.c_str());
    if (!handle) {
        DWORD errorCode = GetLastError();
        LPVOID errorMsg;
        FormatMessage(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            errorCode,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
            (LPTSTR)&errorMsg,
            0,
            NULL
        );
        std::string errorMessage = "Failed to load library: " + std::string((char*)errorMsg);
        LocalFree(errorMsg);
        throw std::runtime_error(errorMessage);
    }
    return handle;
}

void unloadFmuLibrary(HMODULE handle) {
    FreeLibrary(handle);
}
#else
void* loadFmuLibrary(const std::string& libPath) {
    void* handle = dlopen(libPath.c_str(), RTLD_LAZY);
    if (!handle) {
        std::ostringstream oss;
        oss << "Failed to load FMU library '" << libPath << "': " << dlerror();
        throw std::runtime_error(oss.str());
    }
    return handle;
}

void unloadFmuLibrary(void* handle) {
    dlclose(handle);
}
#endif

int main(int argc, char* argv[]) {
    if (argc != 2) {
        std::cerr << "Usage: " << argv[0] << " <path_to_fmu>" << std::endl;
        return 1;
    }

    std::string fmuPath = argv[1];
    std::string instanceName = "test_instance";
    std::string instantiationToken = "{1AE5E10D-9521-4DE3-80B9-D0EAAA7D5AF1}";
    
    try {
        // Unzip the FMU
        std::string tempPath = unzipFmu(fmuPath);
        
        // Get the model name from the FMU path
        std::filesystem::path fmuFilePath(fmuPath);
        //std::string modelName = fmuFilePath.stem().string();
        std::string modelName = "BouncingBall";
        
        // Construct the library path
        std::string libPath = constructLibraryPath(tempPath, modelName);
        
        // Load the FMU library
        void* fmuLibrary = loadFmuLibrary(libPath);
        
        // Create a temporary directory for resources
        std::string resourcePath = tempPath + "/resources";
        std::filesystem::create_directories(resourcePath);

        // Load FMI functions
        loadFmiFunctions(fmuLibrary);
        
        // Instantiate the FMU
        fmi3Instance instance = inst_fmi3InstantiateCoSimulation(
            instanceName.c_str(),
            instantiationToken.c_str(),
            resourcePath.c_str(),
            fmi3False,  // visible
            fmi3True,   // loggingOn
            fmi3False,  // eventModeUsed
            fmi3False,  // earlyReturnAllowed
            nullptr,    // requiredIntermediateVariables
            0,         // nRequiredIntermediateVariables
            nullptr,   // instanceEnvironment
            logMessage,
            nullptr    // intermediateUpdate
        );

        if (!instance) {
            throw std::runtime_error("Failed to instantiate FMU");
        }

        // Enter initialization mode
        fmi3Status status = inst_fmi3EnterInitializationMode(
            instance,
            fmi3False,  // toleranceDefined
            0.0,       // tolerance
            0.0,       // startTime
            fmi3False, // stopTimeDefined
            0.0        // stopTime
        );
        if (status != fmi3OK) {
            throw std::runtime_error("Failed to enter initialization mode");
        }

        // Exit initialization mode
        status = inst_fmi3ExitInitializationMode(instance);
        if (status != fmi3OK) {
            throw std::runtime_error("Failed to exit initialization mode");
        }

        // Run a simple simulation
        fmi3Boolean eventHandlingNeeded = fmi3False;
        fmi3Boolean terminateSimulation = fmi3False;
        fmi3Boolean earlyReturn = fmi3False;
        fmi3Float64 lastSuccessfulTime = 0.0;

        // Do a few simulation steps
        for (double time = 0.0; time < 1.0; time += 0.1) {
            status = inst_fmi3DoStep(
                instance,
                time,           // currentCommunicationPoint
                0.1,           // communicationStepSize
                fmi3True,      // noSetFMUStatePriorToCurrentPoint
                &eventHandlingNeeded,
                &terminateSimulation,
                &earlyReturn,
                &lastSuccessfulTime
            );

            if (status != fmi3OK) {
                throw std::runtime_error("Simulation step failed");
            }

            if (terminateSimulation) {
                std::cout << "Simulation terminated by FMU" << std::endl;
                break;
            }
        }

        // Terminate the FMU
        status = inst_fmi3Terminate(instance);
        if (status != fmi3OK) {
            throw std::runtime_error("Failed to terminate FMU");
        }

        // Free the instance
        inst_fmi3FreeInstance(instance);

        // Unload the FMU library
        unloadFmuLibrary(fmuLibrary);

        std::cout << "Simulation completed successfully" << std::endl;
        return 0;
    }
    catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}