#include "fmi3Logging.hpp"

void fmi3LogMessage(fmi3InstanceEnvironment instanceEnvironment,
                    fmi3Status status,
                    fmi3String category,
                    fmi3String message
                    ) {

    // Print the status
    switch (status) {
        case fmi3OK:
            std::cout << "OK: ";
            break;
        case fmi3Warning:
            std::cout << "Warning: ";
            break;
        case fmi3Discard:
            std::cout << "Discard: ";
            break;
        case fmi3Error:
            std::cout << "Error: ";
            break;
        case fmi3Fatal:
            std::cout << "Fatal: ";
            break;
        default:
            std::cout << "Unknown status: ";
            break;
    }

    // Print the category
    std::cout << "[" << category << "] ";

    // Print the formatted message
     std::cout << message << std::endl;
}
