#ifndef RUST_SWIFT_BRIDGE_H
#define RUST_SWIFT_BRIDGE_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stddef.h>

char* zama_run();
char* zama_get_client_key();
char* zama_get_server_key();
char* zama_get_keys();

int32_t zama_process_genome_data(
    const char* filename,
    size_t num_lines,
    const char* client_key_json,
    const char* server_key_json
);


#endif