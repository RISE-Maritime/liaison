#include <iostream>
#include "fmi3Functions.h"

int main() {
    fmi3Instance instance = fmi3InstantiateCoSimulation(
        "instanceName", "{1AE5E10D-9521-4DE3-80B9-D0EAAA7D5AF1}", "resourcePath",
        fmi3False, fmi3False, fmi3False, fmi3False, nullptr, 0, nullptr, nullptr, nullptr
    );

    if (instance == nullptr) {
        std::cerr << "Failed to instantiate FMU" << std::endl;
        return 1;
    }

    std::cout << "FMU instantiated successfully" << std::endl;
    fmi3Status status = fmi3EnterInitializationMode(instance, fmi3False, 0.0, 0.0, fmi3False, 0.0);
    if (status != fmi3OK) {
        std::cerr << "Failed to enter initialization mode" << std::endl;
        return 1;
    }


    fmi3FreeInstance(instance);
    return 0;
}