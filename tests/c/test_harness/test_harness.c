#include "test_harness.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <dlfcn.h>
#include <signal.h>
#include <sys/wait.h>
#include <errno.h>
#include <fcntl.h>

/* ============================================================================
 * Internal State for Call Log
 * ============================================================================ */

static CallRecord g_harness_call_log[MAX_CALL_RECORDS];
static size_t g_harness_call_count = 0;

/* ============================================================================
 * Harness Lifecycle Functions
 * ============================================================================ */

int harness_init(TestHarness* harness) {
    if (!harness) return -1;

    memset(harness, 0, sizeof(TestHarness));

    /* Get paths from environment variables */
    const char* liaison_binary = getenv("LIAISON_BINARY");
    const char* liaison_fmu_lib = getenv("LIAISON_FMU_LIB");
    const char* dummy_fmu_lib = getenv("DUMMY_FMU_LIB");
    const char* dummy_fmu_path = getenv("DUMMY_FMU_PATH");

    if (!liaison_binary || !liaison_fmu_lib || !dummy_fmu_lib || !dummy_fmu_path) {
        fprintf(stderr, "Missing required environment variables:\n");
        fprintf(stderr, "  LIAISON_BINARY=%s\n", liaison_binary ? liaison_binary : "(not set)");
        fprintf(stderr, "  LIAISON_FMU_LIB=%s\n", liaison_fmu_lib ? liaison_fmu_lib : "(not set)");
        fprintf(stderr, "  DUMMY_FMU_LIB=%s\n", dummy_fmu_lib ? dummy_fmu_lib : "(not set)");
        fprintf(stderr, "  DUMMY_FMU_PATH=%s\n", dummy_fmu_path ? dummy_fmu_path : "(not set)");
        return -1;
    }

    strncpy(harness->liaison_binary_path, liaison_binary, HARNESS_MAX_PATH - 1);
    strncpy(harness->liaison_fmu_lib_path, liaison_fmu_lib, HARNESS_MAX_PATH - 1);
    strncpy(harness->dummy_fmu_lib_path, dummy_fmu_lib, HARNESS_MAX_PATH - 1);
    strncpy(harness->dummy_fmu_dir, dummy_fmu_path, HARNESS_MAX_PATH - 1);  /* FMU path */

    /* Set call log path */
    snprintf(harness->call_log_path, HARNESS_MAX_PATH,
             "/tmp/dummy_fmu_calls_%d.json", getpid());

    /*
     * LiaisonFMU looks for config.json at: libraryPath.parent_path().parent_path()
     * For library at /workspace/build/binaries/x86_64-linux/libliaisonfmu.so
     * it expects config.json at /workspace/build/binaries/config.json
     *
     * Extract the base directory from the library path.
     */
    char lib_path_copy[HARNESS_MAX_PATH];
    strncpy(lib_path_copy, liaison_fmu_lib, HARNESS_MAX_PATH - 1);
    lib_path_copy[HARNESS_MAX_PATH - 1] = '\0';

    /* Find the parent of the parent directory */
    char* last_slash = strrchr(lib_path_copy, '/');
    if (last_slash) {
        *last_slash = '\0';  /* Remove filename: /workspace/build/binaries/x86_64-linux */
        last_slash = strrchr(lib_path_copy, '/');
        if (last_slash) {
            *last_slash = '\0';  /* Remove x86_64-linux: /workspace/build/binaries */
        }
    }

    /* Store the base directory for config.json */
    strncpy(harness->config_json_path, lib_path_copy, HARNESS_MAX_PATH - 1);

    /* Write config.json at the location LiaisonFMU expects */
    char config_path[HARNESS_MAX_PATH];
    snprintf(config_path, sizeof(config_path), "%s/config.json", harness->config_json_path);
    FILE* config_file = fopen(config_path, "w");
    if (config_file) {
        fprintf(config_file, "{\n");
        fprintf(config_file, "  \"responderId\": \"%s\"\n", HARNESS_RESPONDER_ID);
        fprintf(config_file, "}\n");
        fclose(config_file);
    } else {
        fprintf(stderr, "Failed to create config.json at %s\n", config_path);
        return -1;
    }

    return 0;
}

int harness_start_server(TestHarness* harness) {
    if (!harness) return -1;

    /* Check if liaison binary exists */
    if (access(harness->liaison_binary_path, X_OK) != 0) {
        fprintf(stderr, "Liaison binary not found or not executable: %s\n",
                harness->liaison_binary_path);
        return -1;
    }

    /* Check if dummy FMU file exists */
    if (access(harness->dummy_fmu_dir, R_OK) != 0) {
        fprintf(stderr, "Dummy FMU file not found: %s\n",
                harness->dummy_fmu_dir);
        return -1;
    }

    /* Set environment variable for call log path */
    setenv("DUMMY_FMU_CALL_LOG", harness->call_log_path, 1);

    /* Fork and exec the liaison server */
    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        return -1;
    }

    if (pid == 0) {
        /* Child process: exec liaison server */

        /* Keep stderr for debugging, redirect stdout to /dev/null */
        int devnull = open("/dev/null", O_WRONLY);
        if (devnull >= 0) {
            dup2(devnull, STDOUT_FILENO);
            /* Keep stderr for debugging */
            close(devnull);
        }

        execlp(harness->liaison_binary_path,
               "liaison",
               "--serve",
               harness->dummy_fmu_dir,
               HARNESS_RESPONDER_ID,
               NULL);

        /* If exec fails */
        perror("execlp");
        _exit(1);
    }

    /* Parent process */
    harness->server_pid = pid;
    harness->server_running = true;

    /* Wait for server to start up (Zenoh discovery) */
    usleep(HARNESS_SERVER_STARTUP_WAIT_MS * 1000);

    /* Verify server is still running */
    int status;
    pid_t result = waitpid(pid, &status, WNOHANG);
    if (result != 0) {
        fprintf(stderr, "Server process exited unexpectedly\n");
        harness->server_running = false;
        return -1;
    }

    return 0;
}

int harness_stop_server(TestHarness* harness) {
    if (!harness || !harness->server_running) return 0;

    /* Send SIGTERM to gracefully stop the server */
    if (kill(harness->server_pid, SIGTERM) != 0) {
        if (errno != ESRCH) {
            perror("kill SIGTERM");
        }
    }

    /* Wait for server to exit */
    int status;
    int wait_count = 0;
    while (wait_count < 10) {
        pid_t result = waitpid(harness->server_pid, &status, WNOHANG);
        if (result > 0) {
            break;
        }
        usleep(100000);  /* 100ms */
        wait_count++;
    }

    /* If still running, force kill */
    if (wait_count >= 10) {
        kill(harness->server_pid, SIGKILL);
        waitpid(harness->server_pid, &status, 0);
    }

    harness->server_running = false;
    harness->server_pid = 0;

    return 0;
}

int harness_load_liaison_fmu(TestHarness* harness) {
    if (!harness) return -1;

    /* Check if library exists */
    if (access(harness->liaison_fmu_lib_path, R_OK) != 0) {
        fprintf(stderr, "LiaisonFMU library not found: %s\n",
                harness->liaison_fmu_lib_path);
        return -1;
    }

    /* Load the library */
    harness->liaison_fmu_lib = dlopen(harness->liaison_fmu_lib_path, RTLD_NOW);
    if (!harness->liaison_fmu_lib) {
        fprintf(stderr, "Failed to load LiaisonFMU library: %s\n", dlerror());
        return -1;
    }

    /* Bind function pointers */
    #define BIND_FUNC(name) \
        harness->name = (name##TYPE*)dlsym(harness->liaison_fmu_lib, #name); \
        if (!harness->name) { \
            fprintf(stderr, "Failed to bind " #name ": %s\n", dlerror()); \
            dlclose(harness->liaison_fmu_lib); \
            harness->liaison_fmu_lib = NULL; \
            return -1; \
        }

    BIND_FUNC(fmi3GetVersion);
    BIND_FUNC(fmi3SetDebugLogging);
    BIND_FUNC(fmi3InstantiateModelExchange);
    BIND_FUNC(fmi3InstantiateCoSimulation);
    BIND_FUNC(fmi3InstantiateScheduledExecution);
    BIND_FUNC(fmi3FreeInstance);
    BIND_FUNC(fmi3EnterInitializationMode);
    BIND_FUNC(fmi3ExitInitializationMode);
    BIND_FUNC(fmi3EnterEventMode);
    BIND_FUNC(fmi3Terminate);
    BIND_FUNC(fmi3Reset);
    BIND_FUNC(fmi3GetFloat32);
    BIND_FUNC(fmi3GetFloat64);
    BIND_FUNC(fmi3GetInt8);
    BIND_FUNC(fmi3GetUInt8);
    BIND_FUNC(fmi3GetInt16);
    BIND_FUNC(fmi3GetUInt16);
    BIND_FUNC(fmi3GetInt32);
    BIND_FUNC(fmi3GetUInt32);
    BIND_FUNC(fmi3GetInt64);
    BIND_FUNC(fmi3GetUInt64);
    BIND_FUNC(fmi3GetBoolean);
    BIND_FUNC(fmi3GetString);
    BIND_FUNC(fmi3GetBinary);
    BIND_FUNC(fmi3GetClock);
    BIND_FUNC(fmi3SetFloat32);
    BIND_FUNC(fmi3SetFloat64);
    BIND_FUNC(fmi3SetInt8);
    BIND_FUNC(fmi3SetUInt8);
    BIND_FUNC(fmi3SetInt16);
    BIND_FUNC(fmi3SetUInt16);
    BIND_FUNC(fmi3SetInt32);
    BIND_FUNC(fmi3SetUInt32);
    BIND_FUNC(fmi3SetInt64);
    BIND_FUNC(fmi3SetUInt64);
    BIND_FUNC(fmi3SetBoolean);
    BIND_FUNC(fmi3SetString);
    BIND_FUNC(fmi3SetBinary);
    BIND_FUNC(fmi3SetClock);
    BIND_FUNC(fmi3DoStep);

    #undef BIND_FUNC

    return 0;
}

void harness_cleanup(TestHarness* harness) {
    if (!harness) return;

    /* Free current instance */
    harness_free_instance(harness);

    /* Stop server */
    harness_stop_server(harness);

    /* Unload library */
    if (harness->liaison_fmu_lib) {
        dlclose(harness->liaison_fmu_lib);
        harness->liaison_fmu_lib = NULL;
    }

    /* Remove call log file */
    if (harness->call_log_path[0]) {
        unlink(harness->call_log_path);
    }

    /* Remove config.json file (but not the directory) */
    if (harness->config_json_path[0]) {
        char config_path[HARNESS_MAX_PATH];
        snprintf(config_path, sizeof(config_path), "%s/config.json", harness->config_json_path);
        unlink(config_path);
    }
}

/* ============================================================================
 * Call Log Access Functions
 * ============================================================================ */

int harness_clear_call_log(TestHarness* harness) {
    if (!harness) return -1;

    /* Write empty call log */
    FILE* f = fopen(harness->call_log_path, "w");
    if (!f) return -1;
    fprintf(f, "{\"call_count\": 0, \"calls\": []}\n");
    fclose(f);

    g_harness_call_count = 0;
    memset(g_harness_call_log, 0, sizeof(g_harness_call_log));

    return 0;
}

/* Simple JSON parser for call log (minimal implementation) */
static int parse_call_log_json(const char* json, CallRecord* records, size_t* count) {
    *count = 0;

    /* Find call_count */
    const char* p = strstr(json, "\"call_count\":");
    if (!p) return -1;
    p += strlen("\"call_count\":");
    while (*p == ' ') p++;
    *count = (size_t)atoi(p);

    if (*count == 0) return 0;
    if (*count > MAX_CALL_RECORDS) *count = MAX_CALL_RECORDS;

    /* Find calls array */
    p = strstr(json, "\"calls\":");
    if (!p) return -1;
    p = strchr(p, '[');
    if (!p) return -1;

    /* Parse each call record */
    for (size_t i = 0; i < *count; i++) {
        CallRecord* rec = &records[i];
        memset(rec, 0, sizeof(CallRecord));

        /* Find next object start */
        p = strchr(p, '{');
        if (!p) break;

        /* Find end of this object - need to handle nested braces and strings */
        const char* obj_end = p + 1;
        int brace_count = 1;
        int in_string = 0;
        while (*obj_end && brace_count > 0) {
            if (*obj_end == '"' && (obj_end == p + 1 || *(obj_end-1) != '\\')) {
                in_string = !in_string;
            } else if (!in_string) {
                if (*obj_end == '{') brace_count++;
                else if (*obj_end == '}') brace_count--;
            }
            obj_end++;
        }
        if (brace_count != 0) break;

        /* Parse type */
        const char* type_start = strstr(p, "\"type\":");
        if (type_start && type_start < obj_end) {
            /* Skip past "type": to find the value */
            type_start += strlen("\"type\":");
            /* Skip whitespace and find opening quote of value */
            while (*type_start == ' ' || *type_start == '\t' || *type_start == '\n') type_start++;
            if (*type_start == '"') {
                type_start++;  /* Skip opening quote */
                const char* type_end = strchr(type_start, '"');
                if (type_end && type_end < obj_end) {
                    char type_str[64];
                    size_t len = type_end - type_start;
                    if (len >= sizeof(type_str)) len = sizeof(type_str) - 1;
                    memcpy(type_str, type_start, len);
                    type_str[len] = '\0';

                    /* Map string to enum */
                    if (strcmp(type_str, "INSTANTIATE_CO_SIMULATION") == 0) rec->type = CALL_FMI3_INSTANTIATE_CO_SIMULATION;
                    else if (strcmp(type_str, "INSTANTIATE_MODEL_EXCHANGE") == 0) rec->type = CALL_FMI3_INSTANTIATE_MODEL_EXCHANGE;
                    else if (strcmp(type_str, "INSTANTIATE_SCHEDULED_EXECUTION") == 0) rec->type = CALL_FMI3_INSTANTIATE_SCHEDULED_EXECUTION;
                    else if (strcmp(type_str, "FREE_INSTANCE") == 0) rec->type = CALL_FMI3_FREE_INSTANCE;
                    else if (strcmp(type_str, "ENTER_INITIALIZATION_MODE") == 0) rec->type = CALL_FMI3_ENTER_INITIALIZATION_MODE;
                    else if (strcmp(type_str, "EXIT_INITIALIZATION_MODE") == 0) rec->type = CALL_FMI3_EXIT_INITIALIZATION_MODE;
                    else if (strcmp(type_str, "ENTER_EVENT_MODE") == 0) rec->type = CALL_FMI3_ENTER_EVENT_MODE;
                    else if (strcmp(type_str, "TERMINATE") == 0) rec->type = CALL_FMI3_TERMINATE;
                    else if (strcmp(type_str, "RESET") == 0) rec->type = CALL_FMI3_RESET;
                    else if (strcmp(type_str, "GET_FLOAT64") == 0) rec->type = CALL_FMI3_GET_FLOAT64;
                    else if (strcmp(type_str, "SET_FLOAT64") == 0) rec->type = CALL_FMI3_SET_FLOAT64;
                    else if (strcmp(type_str, "GET_FLOAT32") == 0) rec->type = CALL_FMI3_GET_FLOAT32;
                    else if (strcmp(type_str, "SET_FLOAT32") == 0) rec->type = CALL_FMI3_SET_FLOAT32;
                    else if (strcmp(type_str, "GET_INT8") == 0) rec->type = CALL_FMI3_GET_INT8;
                    else if (strcmp(type_str, "SET_INT8") == 0) rec->type = CALL_FMI3_SET_INT8;
                    else if (strcmp(type_str, "GET_UINT8") == 0) rec->type = CALL_FMI3_GET_UINT8;
                    else if (strcmp(type_str, "SET_UINT8") == 0) rec->type = CALL_FMI3_SET_UINT8;
                    else if (strcmp(type_str, "GET_INT16") == 0) rec->type = CALL_FMI3_GET_INT16;
                    else if (strcmp(type_str, "SET_INT16") == 0) rec->type = CALL_FMI3_SET_INT16;
                    else if (strcmp(type_str, "GET_UINT16") == 0) rec->type = CALL_FMI3_GET_UINT16;
                    else if (strcmp(type_str, "SET_UINT16") == 0) rec->type = CALL_FMI3_SET_UINT16;
                    else if (strcmp(type_str, "GET_INT32") == 0) rec->type = CALL_FMI3_GET_INT32;
                    else if (strcmp(type_str, "SET_INT32") == 0) rec->type = CALL_FMI3_SET_INT32;
                    else if (strcmp(type_str, "GET_UINT32") == 0) rec->type = CALL_FMI3_GET_UINT32;
                    else if (strcmp(type_str, "SET_UINT32") == 0) rec->type = CALL_FMI3_SET_UINT32;
                    else if (strcmp(type_str, "GET_INT64") == 0) rec->type = CALL_FMI3_GET_INT64;
                    else if (strcmp(type_str, "SET_INT64") == 0) rec->type = CALL_FMI3_SET_INT64;
                    else if (strcmp(type_str, "GET_UINT64") == 0) rec->type = CALL_FMI3_GET_UINT64;
                    else if (strcmp(type_str, "SET_UINT64") == 0) rec->type = CALL_FMI3_SET_UINT64;
                    else if (strcmp(type_str, "GET_BOOLEAN") == 0) rec->type = CALL_FMI3_GET_BOOLEAN;
                    else if (strcmp(type_str, "SET_BOOLEAN") == 0) rec->type = CALL_FMI3_SET_BOOLEAN;
                    else if (strcmp(type_str, "GET_STRING") == 0) rec->type = CALL_FMI3_GET_STRING;
                    else if (strcmp(type_str, "SET_STRING") == 0) rec->type = CALL_FMI3_SET_STRING;
                    else if (strcmp(type_str, "GET_BINARY") == 0) rec->type = CALL_FMI3_GET_BINARY;
                    else if (strcmp(type_str, "SET_BINARY") == 0) rec->type = CALL_FMI3_SET_BINARY;
                    else if (strcmp(type_str, "GET_CLOCK") == 0) rec->type = CALL_FMI3_GET_CLOCK;
                    else if (strcmp(type_str, "SET_CLOCK") == 0) rec->type = CALL_FMI3_SET_CLOCK;
                    else if (strcmp(type_str, "DO_STEP") == 0) rec->type = CALL_FMI3_DO_STEP;
                    else if (strcmp(type_str, "SET_DEBUG_LOGGING") == 0) rec->type = CALL_FMI3_SET_DEBUG_LOGGING;
                    else if (strcmp(type_str, "GET_VERSION") == 0) rec->type = CALL_FMI3_GET_VERSION;
                }
            }
        }

        /* Parse instance_index */
        const char* idx_start = strstr(p, "\"instance_index\":");
        if (idx_start && idx_start < obj_end) {
            idx_start += strlen("\"instance_index\":");
            while (*idx_start == ' ') idx_start++;
            rec->instance_index = atoi(idx_start);
        }

        /* Parse instance_name */
        const char* name_start = strstr(p, "\"instance_name\":");
        if (name_start && name_start < obj_end) {
            name_start = strchr(name_start + strlen("\"instance_name\":"), '"');
            if (name_start) {
                name_start++;
                const char* name_end = strchr(name_start, '"');
                if (name_end && name_end < obj_end) {
                    size_t len = name_end - name_start;
                    if (len >= MAX_STRING_LENGTH) len = MAX_STRING_LENGTH - 1;
                    memcpy(rec->instance_name, name_start, len);
                    rec->instance_name[len] = '\0';
                }
            }
        }

        /* Parse instantiation_token */
        const char* token_start = strstr(p, "\"instantiation_token\":");
        if (token_start && token_start < obj_end) {
            token_start = strchr(token_start + strlen("\"instantiation_token\":"), '"');
            if (token_start) {
                token_start++;
                const char* token_end = strchr(token_start, '"');
                if (token_end && token_end < obj_end) {
                    size_t len = token_end - token_start;
                    if (len >= MAX_STRING_LENGTH) len = MAX_STRING_LENGTH - 1;
                    memcpy(rec->instantiation_token, token_start, len);
                    rec->instantiation_token[len] = '\0';
                }
            }
        }

        /* Parse visible */
        const char* visible_start = strstr(p, "\"visible\":");
        if (visible_start && visible_start < obj_end) {
            visible_start += strlen("\"visible\":");
            while (*visible_start == ' ' || *visible_start == '\t') visible_start++;
            rec->visible = (strncmp(visible_start, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        /* Parse logging_on */
        const char* logging_start = strstr(p, "\"logging_on\":");
        if (logging_start && logging_start < obj_end) {
            logging_start += strlen("\"logging_on\":");
            while (*logging_start == ' ' || *logging_start == '\t') logging_start++;
            rec->logging_on = (strncmp(logging_start, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        /* Parse event_mode_used */
        const char* event_mode_start = strstr(p, "\"event_mode_used\":");
        if (event_mode_start && event_mode_start < obj_end) {
            event_mode_start += strlen("\"event_mode_used\":");
            while (*event_mode_start == ' ' || *event_mode_start == '\t') event_mode_start++;
            rec->event_mode_used = (strncmp(event_mode_start, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        /* Parse early_return_allowed */
        const char* early_return_start = strstr(p, "\"early_return_allowed\":");
        if (early_return_start && early_return_start < obj_end) {
            early_return_start += strlen("\"early_return_allowed\":");
            while (*early_return_start == ' ' || *early_return_start == '\t') early_return_start++;
            rec->early_return_allowed = (strncmp(early_return_start, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        /* Parse n_value_references */
        const char* nvr_start = strstr(p, "\"n_value_references\":");
        if (nvr_start && nvr_start < obj_end) {
            nvr_start += strlen("\"n_value_references\":");
            while (*nvr_start == ' ') nvr_start++;
            rec->n_value_references = (size_t)atoi(nvr_start);
        }

        /* Parse n_values */
        const char* nv_start = strstr(p, "\"n_values\":");
        if (nv_start && nv_start < obj_end) {
            nv_start += strlen("\"n_values\":");
            while (*nv_start == ' ') nv_start++;
            rec->n_values = (size_t)atoi(nv_start);
        }

        /* Parse value_references array */
        const char* vr_start = strstr(p, "\"value_references\":");
        if (vr_start && vr_start < obj_end) {
            vr_start = strchr(vr_start, '[');
            if (vr_start && vr_start < obj_end) {
                vr_start++;
                for (size_t j = 0; j < rec->n_value_references && j < MAX_VALUE_REFERENCES; j++) {
                    while (*vr_start == ' ' || *vr_start == ',') vr_start++;
                    if (*vr_start == ']') break;
                    rec->value_references[j] = (fmi3ValueReference)atoi(vr_start);
                    while (*vr_start >= '0' && *vr_start <= '9') vr_start++;
                }
            }
        }

        /* Parse float64_values array */
        const char* f64_start = strstr(p, "\"float64_values\":");
        if (f64_start && f64_start < obj_end) {
            f64_start = strchr(f64_start, '[');
            if (f64_start && f64_start < obj_end) {
                f64_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Float64); j++) {
                    while (*f64_start == ' ' || *f64_start == ',') f64_start++;
                    if (*f64_start == ']') break;
                    rec->values.float64_values[j] = atof(f64_start);
                    /* Skip the number, handling inf, -inf, nan, and regular numbers */
                    while ((*f64_start >= '0' && *f64_start <= '9') || *f64_start == '.' ||
                           *f64_start == '-' || *f64_start == 'e' || *f64_start == 'E' || *f64_start == '+' ||
                           *f64_start == 'i' || *f64_start == 'n' || *f64_start == 'f' ||
                           *f64_start == 'a' /* for nan */) {
                        f64_start++;
                    }
                }
            }
        }

        /* Parse int32_values array */
        const char* i32_start = strstr(p, "\"int32_values\":");
        if (i32_start && i32_start < obj_end) {
            i32_start = strchr(i32_start, '[');
            if (i32_start && i32_start < obj_end) {
                i32_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Int32); j++) {
                    while (*i32_start == ' ' || *i32_start == ',') i32_start++;
                    if (*i32_start == ']') break;
                    rec->values.int32_values[j] = (fmi3Int32)atoi(i32_start);
                    while ((*i32_start >= '0' && *i32_start <= '9') || *i32_start == '-') {
                        i32_start++;
                    }
                }
            }
        }

        /* Parse boolean_values array */
        const char* bool_start = strstr(p, "\"boolean_values\":");
        if (bool_start && bool_start < obj_end) {
            bool_start = strchr(bool_start, '[');
            if (bool_start && bool_start < obj_end) {
                bool_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Boolean); j++) {
                    while (*bool_start == ' ' || *bool_start == ',') bool_start++;
                    if (*bool_start == ']') break;
                    rec->values.boolean_values[j] = (strncmp(bool_start, "true", 4) == 0) ? fmi3True : fmi3False;
                    while (*bool_start != ',' && *bool_start != ']') bool_start++;
                }
            }
        }

        /* Parse initialization mode parameters */
        const char* tol_def = strstr(p, "\"tolerance_defined\":");
        if (tol_def && tol_def < obj_end) {
            tol_def += strlen("\"tolerance_defined\":");
            while (*tol_def == ' ') tol_def++;
            rec->tolerance_defined = (strncmp(tol_def, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        const char* tol = strstr(p, "\"tolerance\":");
        if (tol && tol < obj_end) {
            tol += strlen("\"tolerance\":");
            while (*tol == ' ') tol++;
            rec->tolerance = atof(tol);
        }

        const char* st = strstr(p, "\"start_time\":");
        if (st && st < obj_end) {
            st += strlen("\"start_time\":");
            while (*st == ' ') st++;
            rec->start_time = atof(st);
        }

        const char* stop_def = strstr(p, "\"stop_time_defined\":");
        if (stop_def && stop_def < obj_end) {
            stop_def += strlen("\"stop_time_defined\":");
            while (*stop_def == ' ') stop_def++;
            rec->stop_time_defined = (strncmp(stop_def, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        const char* stop = strstr(p, "\"stop_time\":");
        if (stop && stop < obj_end) {
            stop += strlen("\"stop_time\":");
            while (*stop == ' ') stop++;
            rec->stop_time = atof(stop);
        }

        /* Parse DoStep parameters */
        const char* ccp = strstr(p, "\"current_communication_point\":");
        if (ccp && ccp < obj_end) {
            ccp += strlen("\"current_communication_point\":");
            while (*ccp == ' ') ccp++;
            rec->current_communication_point = atof(ccp);
        }

        const char* css = strstr(p, "\"communication_step_size\":");
        if (css && css < obj_end) {
            css += strlen("\"communication_step_size\":");
            while (*css == ' ') css++;
            rec->communication_step_size = atof(css);
        }

        const char* no_set = strstr(p, "\"no_set_fmu_state_prior_to_current_point\":");
        if (no_set && no_set < obj_end) {
            no_set += strlen("\"no_set_fmu_state_prior_to_current_point\":");
            while (*no_set == ' ') no_set++;
            rec->no_set_fmu_state_prior_to_current_point = (strncmp(no_set, "true", 4) == 0) ? fmi3True : fmi3False;
        }

        /* Parse n_categories */
        const char* n_cat = strstr(p, "\"n_categories\":");
        if (n_cat && n_cat < obj_end) {
            n_cat += strlen("\"n_categories\":");
            while (*n_cat == ' ') n_cat++;
            rec->n_categories = (size_t)atoi(n_cat);
        }

        /* Parse categories array */
        const char* cat_start = strstr(p, "\"categories\":");
        if (cat_start && cat_start < obj_end) {
            cat_start = strchr(cat_start, '[');
            if (cat_start && cat_start < obj_end) {
                cat_start++;
                for (size_t j = 0; j < rec->n_categories && j < MAX_CATEGORIES; j++) {
                    while (*cat_start == ' ' || *cat_start == ',' || *cat_start == '\n') cat_start++;
                    if (*cat_start == ']') break;
                    if (*cat_start == '"') {
                        cat_start++;  /* Skip opening quote */
                        const char* cat_end = strchr(cat_start, '"');
                        if (cat_end && cat_end < obj_end) {
                            size_t len = cat_end - cat_start;
                            if (len >= MAX_STRING_LENGTH) len = MAX_STRING_LENGTH - 1;
                            memcpy(rec->categories[j], cat_start, len);
                            rec->categories[j][len] = '\0';
                            cat_start = cat_end + 1;
                        }
                    }
                }
            }
        }

        /* Parse n_strings - must be before string_values parsing */
        const char* n_str = strstr(p, "\"n_strings\":");
        if (n_str && n_str < obj_end) {
            n_str += strlen("\"n_strings\":");
            while (*n_str == ' ') n_str++;
            rec->n_strings = (size_t)atoi(n_str);
        }

        /* Parse string_values array */
        const char* str_val_start = strstr(p, "\"string_values\":");
        if (str_val_start && str_val_start < obj_end) {
            str_val_start = strchr(str_val_start, '[');
            if (str_val_start && str_val_start < obj_end) {
                str_val_start++;
                for (size_t j = 0; j < rec->n_strings && j < MAX_STRINGS; j++) {
                    while (*str_val_start == ' ' || *str_val_start == ',' || *str_val_start == '\n') str_val_start++;
                    if (*str_val_start == ']') break;
                    if (*str_val_start == '"') {
                        str_val_start++;  /* Skip opening quote */
                        const char* str_end = strchr(str_val_start, '"');
                        if (str_end && str_end < obj_end) {
                            size_t len = str_end - str_val_start;
                            if (len >= MAX_STRING_LENGTH) len = MAX_STRING_LENGTH - 1;
                            memcpy(rec->string_values[j], str_val_start, len);
                            rec->string_values[j][len] = '\0';
                            str_val_start = str_end + 1;
                        }
                    }
                }
            }
        }

        /* Parse binary data */
        const char* total_bin_start = strstr(p, "\"total_binary_size\":");
        if (total_bin_start && total_bin_start < obj_end) {
            total_bin_start += strlen("\"total_binary_size\":");
            while (*total_bin_start == ' ') total_bin_start++;
            rec->total_binary_size = (size_t)atol(total_bin_start);
        }

        const char* bin_sizes_start = strstr(p, "\"binary_sizes\":");
        if (bin_sizes_start && bin_sizes_start < obj_end) {
            bin_sizes_start = strchr(bin_sizes_start, '[');
            if (bin_sizes_start && bin_sizes_start < obj_end) {
                bin_sizes_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUE_REFERENCES; j++) {
                    while (*bin_sizes_start == ' ' || *bin_sizes_start == ',') bin_sizes_start++;
                    if (*bin_sizes_start == ']') break;
                    rec->binary_sizes[j] = (size_t)atol(bin_sizes_start);
                    while ((*bin_sizes_start >= '0' && *bin_sizes_start <= '9')) {
                        bin_sizes_start++;
                    }
                }
            }
        }

        const char* bin_hex_start = strstr(p, "\"binary_data_hex\":");
        if (bin_hex_start && bin_hex_start < obj_end) {
            /* Skip past the key and colon to find the value */
            bin_hex_start += strlen("\"binary_data_hex\":");
            while (*bin_hex_start == ' ') bin_hex_start++;  /* Skip whitespace */
            if (*bin_hex_start == '"') {
                bin_hex_start++;  /* Skip opening quote of value */
                size_t hex_idx = 0;
                size_t bin_idx = 0;
                while (bin_hex_start[hex_idx] && bin_hex_start[hex_idx] != '"' && bin_idx < MAX_BINARY_SIZE) {
                    char hex_byte[3] = {bin_hex_start[hex_idx], bin_hex_start[hex_idx + 1], '\0'};
                    rec->binary_data[bin_idx] = (fmi3Byte)strtol(hex_byte, NULL, 16);
                    hex_idx += 2;
                    bin_idx++;
                }
            }
        }

        /* Parse clock_values array */
        const char* clock_start = strstr(p, "\"clock_values\":");
        if (clock_start && clock_start < obj_end) {
            clock_start = strchr(clock_start, '[');
            if (clock_start && clock_start < obj_end) {
                clock_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Clock); j++) {
                    while (*clock_start == ' ' || *clock_start == ',') clock_start++;
                    if (*clock_start == ']') break;
                    rec->values.clock_values[j] = (strncmp(clock_start, "true", 4) == 0) ? fmi3ClockActive : fmi3ClockInactive;
                    while (*clock_start != ',' && *clock_start != ']') clock_start++;
                }
            }
        }

        /* Parse uint32_values array */
        const char* u32_start = strstr(p, "\"uint32_values\":");
        if (u32_start && u32_start < obj_end) {
            u32_start = strchr(u32_start, '[');
            if (u32_start && u32_start < obj_end) {
                u32_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3UInt32); j++) {
                    while (*u32_start == ' ' || *u32_start == ',') u32_start++;
                    if (*u32_start == ']') break;
                    rec->values.uint32_values[j] = (fmi3UInt32)strtoul(u32_start, NULL, 10);
                    while ((*u32_start >= '0' && *u32_start <= '9')) {
                        u32_start++;
                    }
                }
            }
        }

        /* Parse int8_values array */
        const char* i8_start = strstr(p, "\"int8_values\":");
        if (i8_start && i8_start < obj_end) {
            i8_start = strchr(i8_start, '[');
            if (i8_start && i8_start < obj_end) {
                i8_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Int8); j++) {
                    while (*i8_start == ' ' || *i8_start == ',') i8_start++;
                    if (*i8_start == ']') break;
                    rec->values.int8_values[j] = (fmi3Int8)atoi(i8_start);
                    while ((*i8_start >= '0' && *i8_start <= '9') || *i8_start == '-') {
                        i8_start++;
                    }
                }
            }
        }

        /* Parse uint8_values array */
        const char* u8_start = strstr(p, "\"uint8_values\":");
        if (u8_start && u8_start < obj_end) {
            u8_start = strchr(u8_start, '[');
            if (u8_start && u8_start < obj_end) {
                u8_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3UInt8); j++) {
                    while (*u8_start == ' ' || *u8_start == ',') u8_start++;
                    if (*u8_start == ']') break;
                    rec->values.uint8_values[j] = (fmi3UInt8)strtoul(u8_start, NULL, 10);
                    while (*u8_start >= '0' && *u8_start <= '9') {
                        u8_start++;
                    }
                }
            }
        }

        /* Parse int16_values array */
        const char* i16_start = strstr(p, "\"int16_values\":");
        if (i16_start && i16_start < obj_end) {
            i16_start = strchr(i16_start, '[');
            if (i16_start && i16_start < obj_end) {
                i16_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Int16); j++) {
                    while (*i16_start == ' ' || *i16_start == ',') i16_start++;
                    if (*i16_start == ']') break;
                    rec->values.int16_values[j] = (fmi3Int16)atoi(i16_start);
                    while ((*i16_start >= '0' && *i16_start <= '9') || *i16_start == '-') {
                        i16_start++;
                    }
                }
            }
        }

        /* Parse uint16_values array */
        const char* u16_start = strstr(p, "\"uint16_values\":");
        if (u16_start && u16_start < obj_end) {
            u16_start = strchr(u16_start, '[');
            if (u16_start && u16_start < obj_end) {
                u16_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3UInt16); j++) {
                    while (*u16_start == ' ' || *u16_start == ',') u16_start++;
                    if (*u16_start == ']') break;
                    rec->values.uint16_values[j] = (fmi3UInt16)strtoul(u16_start, NULL, 10);
                    while (*u16_start >= '0' && *u16_start <= '9') {
                        u16_start++;
                    }
                }
            }
        }

        /* Parse int64_values array */
        const char* i64_start = strstr(p, "\"int64_values\":");
        if (i64_start && i64_start < obj_end) {
            i64_start = strchr(i64_start, '[');
            if (i64_start && i64_start < obj_end) {
                i64_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3Int64); j++) {
                    while (*i64_start == ' ' || *i64_start == ',') i64_start++;
                    if (*i64_start == ']') break;
                    rec->values.int64_values[j] = (fmi3Int64)strtoll(i64_start, NULL, 10);
                    while ((*i64_start >= '0' && *i64_start <= '9') || *i64_start == '-') {
                        i64_start++;
                    }
                }
            }
        }

        /* Parse uint64_values array */
        const char* u64_start = strstr(p, "\"uint64_values\":");
        if (u64_start && u64_start < obj_end) {
            u64_start = strchr(u64_start, '[');
            if (u64_start && u64_start < obj_end) {
                u64_start++;
                for (size_t j = 0; j < rec->n_values && j < MAX_VALUES_SIZE / sizeof(fmi3UInt64); j++) {
                    while (*u64_start == ' ' || *u64_start == ',') u64_start++;
                    if (*u64_start == ']') break;
                    rec->values.uint64_values[j] = (fmi3UInt64)strtoull(u64_start, NULL, 10);
                    while (*u64_start >= '0' && *u64_start <= '9') {
                        u64_start++;
                    }
                }
            }
        }

        p = obj_end + 1;
    }

    return 0;
}

int harness_read_call_log(TestHarness* harness) {
    if (!harness) return -1;

    /* Give server a moment to write the file */
    usleep(50000);  /* 50ms */

    FILE* f = fopen(harness->call_log_path, "r");
    if (!f) {
        g_harness_call_count = 0;
        return 0;  /* File may not exist yet */
    }

    /* Read entire file */
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    if (size <= 0 || size > 1024 * 1024) {  /* Max 1MB */
        fclose(f);
        return -1;
    }

    char* json = (char*)malloc(size + 1);
    if (!json) {
        fclose(f);
        return -1;
    }

    size_t read_size = fread(json, 1, size, f);
    fclose(f);
    json[read_size] = '\0';

    int result = parse_call_log_json(json, g_harness_call_log, &g_harness_call_count);
    free(json);

    return result;
}

size_t harness_get_call_count(TestHarness* harness) {
    (void)harness;
    return g_harness_call_count;
}

const CallRecord* harness_get_call_record(TestHarness* harness, size_t index) {
    (void)harness;
    if (index >= g_harness_call_count) return NULL;
    return &g_harness_call_log[index];
}

/* ============================================================================
 * Convenience Functions
 * ============================================================================ */

void harness_log_callback(fmi3InstanceEnvironment instanceEnvironment,
                          fmi3Status status,
                          fmi3String category,
                          fmi3String message) {
    (void)instanceEnvironment;
    const char* status_str = "?";
    switch (status) {
        case fmi3OK: status_str = "OK"; break;
        case fmi3Warning: status_str = "Warning"; break;
        case fmi3Discard: status_str = "Discard"; break;
        case fmi3Error: status_str = "Error"; break;
        case fmi3Fatal: status_str = "Fatal"; break;
    }
    fprintf(stderr, "[%s] %s: %s\n", status_str, category, message);
}

fmi3Instance harness_create_instance(TestHarness* harness) {
    if (!harness || !harness->fmi3InstantiateCoSimulation) return NULL;

    harness->current_instance = harness->fmi3InstantiateCoSimulation(
        "TestInstance",
        "{dummy-fmu-token}",
        harness->config_json_path,  /* resource path with config.json */
        fmi3False,  /* visible */
        fmi3False,  /* loggingOn */
        fmi3False,  /* eventModeUsed */
        fmi3False,  /* earlyReturnAllowed */
        NULL,       /* requiredIntermediateVariables */
        0,          /* nRequiredIntermediateVariables */
        NULL,       /* instanceEnvironment */
        harness_log_callback,
        NULL        /* intermediateUpdate */
    );

    return harness->current_instance;
}

void harness_free_instance(TestHarness* harness) {
    if (!harness || !harness->current_instance) return;
    if (harness->fmi3FreeInstance) {
        harness->fmi3FreeInstance(harness->current_instance);
    }
    harness->current_instance = NULL;
}

/* ============================================================================
 * CMocka Integration
 * ============================================================================ */

int harness_group_setup(void** state) {
    TestHarness* harness = (TestHarness*)malloc(sizeof(TestHarness));
    if (!harness) return -1;

    if (harness_init(harness) != 0) {
        free(harness);
        return -1;
    }

    if (harness_start_server(harness) != 0) {
        harness_cleanup(harness);
        free(harness);
        return -1;
    }

    if (harness_load_liaison_fmu(harness) != 0) {
        harness_cleanup(harness);
        free(harness);
        return -1;
    }

    *state = harness;
    return 0;
}

int harness_group_teardown(void** state) {
    TestHarness* harness = (TestHarness*)*state;
    if (harness) {
        harness_cleanup(harness);
        free(harness);
    }
    *state = NULL;
    return 0;
}

int harness_test_setup(void** state) {
    TestHarness* harness = (TestHarness*)*state;
    if (!harness) return -1;

    harness_clear_call_log(harness);

    /* Create a fresh instance */
    if (!harness_create_instance(harness)) {
        return -1;
    }

    /* Wait for instance creation to be logged */
    usleep(100000);  /* 100ms */
    harness_read_call_log(harness);

    return 0;
}

int harness_test_teardown(void** state) {
    TestHarness* harness = (TestHarness*)*state;
    if (harness) {
        harness_free_instance(harness);
    }
    return 0;
}
