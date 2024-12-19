//
//  zama.swift
//  monadic_ios
//
//  Created by Aisha Opaluwa on 03/12/2024.
//

import SwiftUI
import KeychainSwift

let keychain = KeychainSwift()

struct ClientKey {
    var key: [UInt8]
}

struct ServerKey {
    var key: [UInt8]
}

func storeClientKey() {
    guard let clientKey = zama_get_client_key() else {
        print("Failed to retrieve client key.")
        return
    }
    let clientKeyString = String(cString: clientKey)
    
    guard let serverKey = zama_get_server_key() else {
        print("Failed to retrieve server key.")
        return
    }
    let serverKeyString = String(cString: serverKey)
    
    
    keychain.set(clientKeyString, forKey: "zamaClientKey")
    keychain.set(serverKeyString, forKey: "zamaServerKey")
    
    print("Successfully stored client key.")
}

func retrieveClientKey() -> String? {
    return keychain.get("zamaClientKey")
}
func retrieveServerKey() -> String? {
    return keychain.get("zamaServerKey")
}


func processGenomeData(filename: String, numLines: Int) {
    // Retrieve keys from Keychain
    guard let clientKey = retrieveClientKey(), let serverKey = retrieveServerKey() else {
        print("Failed to retrieve keys from Keychain.")
        return
    }

    // Convert keys to C strings
    let clientKeyCString = (clientKey as NSString).utf8String
    let serverKeyCString = (serverKey as NSString).utf8String


    // Call zama_process_genome_data
    let result = zama_process_genome_data(
        filename,
        numLines,
        clientKeyCString,
        serverKeyCString
    )

    if result == 0 {
        print("Iteration completed successfully.")
    } else {
        print("Iteration failed.")
    }
}

