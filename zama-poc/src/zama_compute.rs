use std::collections::HashMap;
use tfhe::{ClientKey, CompressedFheUint8, ConfigBuilder, FheBool, FheUint8, generate_keys, ServerKey, set_server_key};
use std::result;
use std::io::Error;
use tfhe::prelude::{FheDecrypt, FheEq, FheTryEncrypt};
use log::info;
use crate::genome_processing;
use bincode;
use std::io::Cursor;
use crate::genome_processing::encode_genotype;

// Iterate through encrypted_genotypes and get the frequency of each genotype
pub fn get_genotype_frequencies(encrypted_genotypes: &HashMap<&u64, CompressedFheUint8>,
    client_key: ClientKey
    )
    -> HashMap<u64, u64> {
    let mut genotype_frequencies = HashMap::new();

    for (_encoded_rsid, encrypted_genotype) in encrypted_genotypes {
        let decompressed_encrypted_genotype = encrypted_genotype.decompress();
        let decrypted_genotype = decompressed_encrypted_genotype.decrypt(&client_key);

        let count = genotype_frequencies.entry(decrypted_genotype).or_insert(0);
        *count += 1;
    }

    genotype_frequencies
}

pub fn check_genotype_server(encrypted_genotypes: &HashMap<&u64, CompressedFheUint8>,
                             rsid: &str,
                             encrypted_target: FheUint8
    ) -> result::Result<FheBool, Error> {
    let encoded_rsid = genome_processing::encode_rsid(rsid);
    let encrypted_genotype = encrypted_genotypes.get(&encoded_rsid).unwrap();
    let decompressed_encrypted_genotype = encrypted_genotype.decompress();
    Ok(decompressed_encrypted_genotype.eq(encrypted_target))
}

pub fn encrypt_genotypes_for_zama(processed_data: &HashMap<u64, u8>, client_key: ClientKey) -> result::Result<HashMap<&u64, CompressedFheUint8>, Error> {
    let mut enc_data = HashMap::new();

    for (encoded_rsid, encoded_genotype) in processed_data {
        let genotype_encrypted = CompressedFheUint8::try_encrypt(*encoded_genotype, &client_key).unwrap();
        enc_data.insert(encoded_rsid, genotype_encrypted);
    }

    Ok(enc_data)
}

pub fn serialize_encrypted_genotypes(server_key: &ServerKey, encrypted_genotypes: &HashMap<&u64, CompressedFheUint8>, mut serialized_data: &mut Vec<u8>) {
    bincode::serialize_into(&mut serialized_data, &server_key.clone()).expect("Could not serialize server key");
    bincode::serialize_into(&mut serialized_data, &encrypted_genotypes).expect("Could not serialize encrypted genotypes");
}

pub fn deserialize_encrypted_genotypes(mut serialized_data: Vec<u8>) -> HashMap<u64, CompressedFheUint8> {
    let mut deserialized_data = Cursor::new(serialized_data);
    let deserialized_server_key: ServerKey = bincode::deserialize_from(&mut deserialized_data).unwrap();
    let deserialized_encrypted_genome: HashMap<u64, CompressedFheUint8> = bincode::deserialize_from(&mut deserialized_data).unwrap();
    deserialized_encrypted_genome
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
    let decoded_result = check_genotype_client(&client_key, &encrypted_genotypes, target_genotype)?;
    info!("Lookup result: {:?}", decoded_result);
    
    let target_genotype = "AA";
    let decoded_result = check_genotype_client(&client_key, &encrypted_genotypes, target_genotype)?;
    info!("Lookup result: {:?}", decoded_result);

    let genotype_frequencies = get_genotype_frequencies(&encrypted_genotypes, client_key.clone());
    info!("Genotype frequencies: {:?}", genotype_frequencies);

    let mut serialized_data = Vec::new();
    serialize_encrypted_genotypes(&cloned_server_key, &encrypted_genotypes, &mut serialized_data);
    info!("Serialized data: {:?}", serialized_data.len());

    let deserialized_encrypted_genome = deserialize_encrypted_genotypes(serialized_data);
    info!("Deserialized data: {:?}", deserialized_encrypted_genome.len());

    return Ok(());
}

fn check_genotype_client(client_key: &ClientKey, encrypted_genotypes: &HashMap<&u64, CompressedFheUint8>, target_genotype: &str) -> Result<bool, Error> {
    let encoded_target_genotype = encode_genotype(target_genotype);
    let encrypted_target_genotype = FheUint8::try_encrypt(encoded_target_genotype, client_key).unwrap();
    let encoded_result = check_genotype_server(&encrypted_genotypes, "rs75333668", encrypted_target_genotype)?;
    let decoded_result = encoded_result.decrypt(&client_key);
    Ok(decoded_result)
}

