#ifndef FMI3LOGGING_HPP
#define FMI3LOGGING_HPP

#include <cstdarg>
#include <cstdio>
#include <iostream>
#include "fmi3Functions.h"

void fmi3LogMessage(fmi3InstanceEnvironment instanceEnvironment,
                    fmi3Status status,
                    fmi3String category,
                    fmi3String message
                    );

#endif // FMI3LOGGING_HPP
