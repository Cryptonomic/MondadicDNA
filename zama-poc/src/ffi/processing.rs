use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use std::io::Error;
use tfhe::prelude::{FheDecrypt, FheEq, FheTryEncrypt};
use crate::genome_processing::{process_file, encode_genotype, get_genotype_encoding_map};

use tfhe::{ClientKey, ServerKey, set_server_key};

use crate::zama_compute::{process_and_encrypt_genome_data, encrypt_genotypes_for_zama, serialize_encrypted_genotypes};


#[no_mangle]
pub extern "C" fn zama_process_genome_data(
    filename: *const c_char,
    num_lines: usize,
    client_key_json: *const c_char,
    server_key_json: *const c_char,
) -> i32 {
    // Convert C-style strings to Rust strings
    let filename_str = unsafe { CStr::from_ptr(filename).to_string_lossy().into_owned() };
    println!("Filename: {}", filename_str);

    let client_key_str = unsafe { CStr::from_ptr(client_key_json).to_string_lossy().into_owned() };

    let server_key_str = unsafe { CStr::from_ptr(server_key_json).to_string_lossy().into_owned() };

    // Deserialize the JSON keys into appropriate Rust types
    let client_key: ClientKey = match serde_json::from_str(&client_key_str) {
        Ok(key) => {
            println!("Successfully deserialized client key.");
            key
        },
        Err(err) => {
            println!("Failed to deserialize client key: {}", err);
            return -1;
        },
    };

    let server_key: ServerKey = match serde_json::from_str(&server_key_str) {
        Ok(key) => {
            println!("Successfully deserialized server key.");
            key
        },
        Err(err) => {
            println!("Failed to deserialize server key: {}", err);
            return -1;
        },
    };

    // Run the iteration process
    println!("Running iteration with {} lines.", num_lines);
    match process_and_encrypt_genome_data(&filename_str, num_lines, &client_key, &server_key) {
        Ok(_) => {
            println!("Iteration completed successfully.");
            0
        },
        Err(err) => {
            println!("Iteration failed with error: {:?}", err);
            -1
        },
    }
}

