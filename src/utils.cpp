#ifdef _WIN32
#include <windows.h>
#endif
#include <vector> 
#include <filesystem>
#include <zip.h>
#include <cstdarg>
#include <cstdio>
#include <iostream>
#include "utils.hpp"

void fmi3LogMessage(fmi3InstanceEnvironment instanceEnvironment,
                    fmi3Status status,
                    fmi3String category,
                    fmi3String message
                    ) {

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

    std::cout << "[" << category << "] ";

    std::cout << message << std::endl;
}


void createDirectories(const std::string& path) {
    std::filesystem::create_directories(path);
}


#ifdef _WIN32
std::string createTempDirectory() {
    char tempPath[MAX_PATH];
    GetTempPath(MAX_PATH, tempPath);
    char tempDir[MAX_PATH];
    if (GetTempFileName(tempPath, "liaison", 0, tempDir) == 0) {
        throw std::runtime_error("Failed to create temporary directory.");
    }
    // Delete the file and create a directory instead
    DeleteFile(tempDir);
    if (!CreateDirectory(tempDir, NULL)) {
        throw std::runtime_error("Failed to create temporary directory.");
    }
    return std::string(tempDir);
}
#else
std::string createTempDirectory() {
    char tempDirTemplate[] = "/tmp/liaison.XXXXXX";
    char* tempDirPath = mkdtemp(tempDirTemplate);
    if (tempDirPath == nullptr) {
        throw std::runtime_error("Failed to create temporary directory.");
    }
    return std::string(tempDirPath);
}
#endif


std::string unzipFmu(const std::string& fmuPath) {
    int err = 0;
    zip *z = zip_open(fmuPath.c_str(), 0, &err);
    if (z == nullptr) {
        throw std::runtime_error("Failed to open FMU file.");
    }

    std::string outputDir = createTempDirectory();

    struct zip_stat st;
    zip_stat_init(&st);
    zip_file *zf = nullptr;

    for (int i = 0; i < zip_get_num_entries(z, 0); ++i) {
        if (zip_stat_index(z, i, 0, &st) == 0) {
            std::filesystem::path outPath = std::filesystem::path(outputDir) / st.name;

            // If it's a directory, create it
            if (outPath.string().back() == '/') {
                createDirectories(outPath);
            } else {
                zf = zip_fopen_index(z, i, 0);
                if (!zf) {
                    zip_close(z);
                    throw std::runtime_error("Failed to open file in zip archive.");
                }

                // Ensure the directory for the file exists
                createDirectories(outPath.parent_path().string());

                std::ofstream outFile(outPath.string(), std::ios::binary);
                if (!outFile) {
                    zip_fclose(zf);
                    zip_close(z);
                    throw std::runtime_error("Failed to create file on disk: " + outPath.string());
                }

                std::vector<char> buffer(st.size);
                zip_fread(zf, buffer.data(), st.size);
                outFile.write(buffer.data(), buffer.size());

                outFile.close();
                zip_fclose(zf);
            }
        }
    }
    zip_close(z);

    return outputDir;
}


bool addFileToZip(zip_t* zipArchive, const std::string& filePath, const std::string& archiveName) {
    if (!std::filesystem::exists(filePath)) {
        std::cerr << "File does not exist: " << filePath << std::endl;
        return false;
    }
    
    zip_source_t* source = zip_source_file(zipArchive, filePath.c_str(), 0, 0);
    if (!source || zip_file_add(zipArchive, archiveName.c_str(), source, ZIP_FL_OVERWRITE) < 0) {
        zip_source_free(source);
        std::cerr << "Failed to add " << archiveName << " to zip archive" << std::endl;
        return false;
    }
    return true;
}