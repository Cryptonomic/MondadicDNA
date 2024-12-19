use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use tfhe::prelude::*;
use tfhe::{ConfigBuilder, FheUint8, FheUint32, generate_keys, set_server_key};
use bincode;

// use crate::genome_processing::{encode_genotype, get_genotype_encoding_map};

#[unsafe(no_mangle)]
pub extern "C" fn zama_run() -> *mut c_char {
    // Basic configuration to use homomorphic integers
    let config = ConfigBuilder::default().build();

    // Key generation
    let (client_key, server_keys) = generate_keys(config);

    let clear_a = 1344u32;
    let clear_b = 5u32;
    let clear_c = 7u8;

    // Encrypting the input data using the (private) client_key
    // FheUint32: Encrypted equivalent to u32
    let mut encrypted_a = FheUint32::try_encrypt(clear_a, &client_key).unwrap();
    let encrypted_b = FheUint32::try_encrypt(clear_b, &client_key).unwrap();

    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &server_keys).unwrap();

    println!(
        "Server Keys Json Len = {}",
        serde_json::to_string(&server_keys).unwrap().len()
    );

    println!("Server Keys Bin Len  = {}", serialized_data.len());

    // FheUint8: Encrypted equivalent to u8
    let encrypted_c = FheUint8::try_encrypt(clear_c, &client_key).unwrap();

    // On the server side:
    set_server_key(server_keys);

    // Clear equivalent computations: 1344 * 5 = 6720
    let encrypted_res_mul = &encrypted_a * &encrypted_b;

    // Clear equivalent computations: 6720 >> 5 = 210
    encrypted_a = &encrypted_res_mul >> &encrypted_b;

    // Clear equivalent computations: let casted_a = a as u8;
    let casted_a: FheUint8 = encrypted_a.cast_into();

    // Clear equivalent computations: min(210, 7) = 7
    let encrypted_res_min = &casted_a.min(&encrypted_c);

    // Operation between clear and encrypted data:
    // Clear equivalent computations: 7 & 1 = 1
    let encrypted_res = encrypted_res_min & 1_u8;

    // Decrypting on the client side:
    let clear_res: u8 = encrypted_res.decrypt(&client_key);
    assert_eq!(clear_res, 1_u8);

    CString::new("Hello, If you see me, zama ran Ok".to_owned())
        .unwrap()
        .into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn zama_get_client_key() -> *mut c_char {
    // Basic configuration to use homomorphic integers
    let config = ConfigBuilder::default().build();

    // Key generation
    let (client_key, server_keys) = generate_keys(config);

    CString::new(serde_json::to_string(&client_key).unwrap())
        .unwrap()
        .into_raw()
}


#[unsafe(no_mangle)]
pub extern "C" fn zama_get_server_key() -> *mut c_char {
    // Basic configuration to use homomorphic integers
    let config = ConfigBuilder::default().build();

    // Key generation
    let (client_key, server_keys) = generate_keys(config);

    CString::new(serde_json::to_string(&server_keys).unwrap())
        .unwrap()
        .into_raw()
}

// TODO: use a single function in getting the keys
#[unsafe(no_mangle)]
pub extern "C" fn zama_get_keys() -> *mut c_char {
    let config = ConfigBuilder::default().build();

    let (client_key, server_keys) = generate_keys(config);

    // Serialize both keys into a JSON object
    let keys = serde_json::json!({
        "client_key": client_key,
        "server_keys": server_keys,
    });

    // Serialize the entire JSON object to a string
    let json_str = serde_json::to_string(&keys).unwrap();

    // Convert to a CString and return a raw pointer
    CString::new(json_str).unwrap().into_raw()
}