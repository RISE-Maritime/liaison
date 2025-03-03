#ifndef FMI3LOGGING_HPP
#define FMI3LOGGING_HPP


#include <fstream>


#include "fmi3Functions.h"


void createDirectories(const std::string& path);

std::string createTempDirectory();

std::string unzipFmu(const std::string& fmuPath);

void addFileToFmu(zip_t* zipArchive, const std::string& filePath, const std::string& archiveName);

#endif // FMI3LOGGING_HPP
