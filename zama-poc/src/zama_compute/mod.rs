use std::time::Instant;
use std::collections::HashMap;
use tfhe::{ClientKey, CompressedFheUint8, ConfigBuilder, FheBool, FheUint8, generate_keys, ServerKey, set_server_key};
use std::result;
use std::io::Error;
use tfhe::prelude::{FheDecrypt, FheEq, FheTryEncrypt};
use log::info;
use crate::genome_processing;
use bincode;
use std::io::Cursor;
use crate::genome_processing::{encode_genotype, get_genotype_encoding_map};
use tfhe::prelude::*;

// Iterate through encrypted_genotypes and get the frequency of each genotype
pub fn get_genotype_frequencies(
    encrypted_genotypes: &HashMap<u64, CompressedFheUint8>,
    max_genotype_value: u8
) -> Vec<FheUint8> {
    info!("Caching encrypted numbers");
    // Create a cache for encrypted values
    let encrypted_values: Vec<FheUint8> = (0..=max_genotype_value)
        .map(|i| FheUint8::try_encrypt_trivial(i).unwrap())
        .collect();

    let encrypted_zero = FheUint8::try_encrypt_trivial(0u8).unwrap();
    let encrypted_one = FheUint8::try_encrypt_trivial(1u8).unwrap();

    info!("Calculating genotype frequencies");
    let mut frequencies = vec![encrypted_zero.clone(); max_genotype_value as usize + 1];

    for encrypted_genotype in encrypted_genotypes.values() {
        let decompressed = encrypted_genotype.decompress();
        for (i, encrypted_value) in encrypted_values.iter().enumerate() {
            let is_match = decompressed.eq(encrypted_value);
            frequencies[i] += is_match.if_then_else(&encrypted_one, &encrypted_zero);
        }
    }

    frequencies
}

pub fn encrypt_genotypes_for_zama(processed_data: &HashMap<u64, u8>, client_key: ClientKey) -> result::Result<HashMap<u64, CompressedFheUint8>, Error> {
    let mut enc_data = HashMap::new();

    for (encoded_rsid, encoded_genotype) in processed_data {
        let genotype_encrypted = CompressedFheUint8::try_encrypt(*encoded_genotype, &client_key).unwrap();
        enc_data.insert(*encoded_rsid, genotype_encrypted);
    }

    Ok(enc_data)
}

pub fn serialize_encrypted_genotypes(server_key: &ServerKey, encrypted_genotypes: &HashMap<u64, CompressedFheUint8>, mut serialized_data: &mut Vec<u8>) {
    bincode::serialize_into(&mut serialized_data, &server_key.clone()).expect("Could not serialize server key");
    bincode::serialize_into(&mut serialized_data, &encrypted_genotypes).expect("Could not serialize encrypted genotypes");
}

pub fn deserialize_encrypted_genotypes(mut serialized_data: Vec<u8>) -> (HashMap<u64, CompressedFheUint8>, ServerKey) {
    let mut deserialized_data = Cursor::new(serialized_data);
    let deserialized_server_key: ServerKey = bincode::deserialize_from(&mut deserialized_data).unwrap();
    let deserialized_encrypted_genome: HashMap<u64, CompressedFheUint8> = bincode::deserialize_from(&mut deserialized_data).unwrap();
    (deserialized_encrypted_genome, deserialized_server_key)
}

pub fn run_iteration(filename: &str, num_lines: usize) -> result::Result<(), Error>{
    info!("Setting up Zama env");
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    let cloned_server_key = server_key.clone();
    set_server_key(server_key);

    info!("Number of lines to process: {:?}", num_lines);

    let processed_data = genome_processing::process_file(filename, num_lines)?;
    info!("Lines of processed data: {:?}", processed_data.len());
    //println!("{:?}", processed_data.);

    let encrypted_genotypes = encrypt_genotypes_for_zama(&processed_data, client_key.clone())?;
    info!("Lines of encrypted data: {:?}", encrypted_genotypes.len());

    let target_genotype = "CC";
    let target_rsid = "rs75333668";
    let encoded_result = check_genotype( &encrypted_genotypes, target_rsid, target_genotype)?;
    let decoded_result = encoded_result.decrypt(&client_key);
    info!("Lookup result: {:?}", decoded_result);

    let target_genotype = "AA";
    let target_rsid = "rs75333668";
    let encoded_result = check_genotype( &encrypted_genotypes, target_rsid, target_genotype)?;
    let decoded_result = encoded_result.decrypt(&client_key);
    info!("Lookup result: {:?}", decoded_result);

    let genotype_frequencies = get_genotype_frequencies(&encrypted_genotypes, 10);
    info!("Genotype frequencies:");
    //Iterate through genotype_frequencies and print the frequency of each genotype
    for (i, frequency) in genotype_frequencies.iter().enumerate() {
        let decrypted_frequency:u8 = frequency.decrypt(&client_key);
        info!("{:?}: {:?}", i, decrypted_frequency);
    }

    let mut serialized_data = Vec::new();
    serialize_encrypted_genotypes(&cloned_server_key, &encrypted_genotypes, &mut serialized_data);
    info!("Serialized data: {:?}", serialized_data.len());

    let (deserialized_encrypted_genome, _) = deserialize_encrypted_genotypes(serialized_data);
    info!("Deserialized data: {:?}", deserialized_encrypted_genome.len());

    return Ok(());
}

pub fn check_genotype(encrypted_genotypes: &HashMap<u64, CompressedFheUint8>, target_rsid: &str, target_genotype: &str) -> Result<FheBool, Error> {
    let encoded_target_genotype = encode_genotype(target_genotype);
    let encrypted_target_genotype = FheUint8::try_encrypt_trivial(encoded_target_genotype).unwrap();
    let encoded_rsid = genome_processing::encode_rsid(target_rsid);
    let encrypted_genotype = encrypted_genotypes.get(&encoded_rsid).unwrap();
    let decompressed_encrypted_genotype = encrypted_genotype.decompress();
    Ok(decompressed_encrypted_genotype.eq(encrypted_target_genotype))
}


pub fn get_encrypted_genotype_encoding_map(client_key: &ClientKey) -> HashMap<&'static str, FheUint8> {
    let genotype_encoding_map = get_genotype_encoding_map();

    genotype_encoding_map
        .into_iter()
        .map(|(genotype, encoding)| {
            let encrypted_encoding = FheUint8::try_encrypt(encoding, client_key).unwrap();
            (genotype, encrypted_encoding)
        })
        .collect()
}

pub fn process_and_encrypt_genome_data(
    filename: &str,
    num_lines: usize,
    client_key: &ClientKey,
    server_key: &ServerKey,
  ) -> result::Result<(), Error> {
    println!("Cloning server key...");

    let cloned_server_key = server_key.clone();
    set_server_key(server_key.clone());

    println!("Number of lines to process: {:?}", num_lines);

    let start_processed_data = Instant::now();

    let processed_data = genome_processing::process_file(filename, num_lines)?;
    println!("Lines of processed data: {:?}", processed_data.len());

    let processed_data_duration = start_processed_data.elapsed();
    println!("genome_processing took: {:?}", processed_data_duration);


    let start_encrypted_genotypes = Instant::now();
    let encrypted_genotypes = encrypt_genotypes_for_zama(&processed_data, client_key.clone())?;
    println!("Lines of encrypted data: {:?}", encrypted_genotypes.len());

    let encrypted_genotypes_duration = start_encrypted_genotypes.elapsed();
    println!("encrypt_genotypes_for_zama took: {:?}", encrypted_genotypes_duration);


    let target_genotype = "CC";
    let target_rsid = "rs75333668";

    let start_check_genotype = Instant::now();
    let encoded_result = check_genotype(&encrypted_genotypes, target_rsid, target_genotype)?;
    let decoded_result = encoded_result.decrypt(&client_key);
    println!("Lookup result: {:?}", decoded_result);

    let check_genotype_duration = start_check_genotype.elapsed();
    println!("check_genotype took: {:?}", check_genotype_duration);


    let target_genotype = "AA";
    let target_rsid = "rs75333668";
    let encoded_result = check_genotype(&encrypted_genotypes, target_rsid, target_genotype)?;
    let decoded_result = encoded_result.decrypt(&client_key);
    println!("Lookup result: {:?}", decoded_result);

    let genotype_frequencies = get_genotype_frequencies(&encrypted_genotypes, 10);
    println!("Genotype frequencies:");

    for (i, frequency) in genotype_frequencies.iter().enumerate() {
        let decrypted_frequency: u8 = frequency.decrypt(&client_key);
        println!("{:?}: {:?}", i, decrypted_frequency);
    }

    let start_serialized_data = Instant::now();

    let mut serialized_data = Vec::new();
    serialize_encrypted_genotypes(&cloned_server_key, &encrypted_genotypes, &mut serialized_data);
    println!("Serialized data: {:?}", serialized_data.len());

    let serialized_data_duration = start_serialized_data.elapsed();
    println!("serialize_encrypted_genotypes took: {:?}", serialized_data_duration);


    Ok(())
}