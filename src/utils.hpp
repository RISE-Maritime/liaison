#ifndef FMI3LOGGING_HPP
#define FMI3LOGGING_HPP


#include <fstream>


#include "fmi3Functions.h"

void fmi3LogMessage(fmi3InstanceEnvironment instanceEnvironment,
                    fmi3Status status,
                    fmi3String category,
                    fmi3String message
                    );

void createDirectories(const std::string& path);

std::string createTempDirectory();

std::string unzipFmu(const std::string& fmuPath);

bool addFileToZip(zip_t* zipArchive, const std::string& filePath, const std::string& archiveName);

#endif // FMI3LOGGING_HPP
