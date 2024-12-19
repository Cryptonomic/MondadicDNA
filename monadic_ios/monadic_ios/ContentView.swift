//
//  ContentView.swift
//  monadic_ios
//
//  Created by Aisha Opaluwa on 26/11/2024.
//

import SwiftUI

struct ContentView: View {
    @State private var statusText: String = ""
    @State private var clientKeyText: String = ""
    
    // Define the number of lines
    @State private var numLines = 20
    
    @State private var presentImporter = false
    

    var body: some View {
        VStack {
            
            Text(statusText)
                .padding()

            Button("Run Zama Code") {
                if let result = zama_run() {
                    let resultString = String(cString: result)
                    statusText = resultString
                    print("Rust function output: \(resultString)")
                }
                storeClientKey()
            }

            Button("Upload Genome Data") {
                presentImporter = true
            }.fileImporter(isPresented: $presentImporter, allowedContentTypes: [.text]) { result in
                uploadAndProcessGenomeData(result: result)
            }
            .padding()
            .background(Color.blue)
            .foregroundColor(.white)
            .cornerRadius(8)

            Button("Retrieve Server Key") {
                guard let serverKey = retrieveServerKey() else {
                    clientKeyText = "No server key found in keychain"
                    return
                }
                let serverKeyLength = strlen(serverKey)
                print("Length of server key: \(serverKeyLength)")
            }


            Button("Retrieve Client Key") {
                guard let clientKey = retrieveClientKey() else {
                    clientKeyText = "No client key found in keychain"
                    return
                }
                let clientKeyLength = strlen(clientKey)
                print("Length of client key: \(clientKeyLength)")

                //                clientKeyText = clientKey

                clientKeyText = "Length of client key: \(clientKeyLength)"

            }
        }
    }
    
    func uploadAndProcessGenomeData(result: Result<URL, Error>) {
        guard let url = try? result.get() else {
            print("File upload failed:", result)
            return
        }
        
        guard url.startAccessingSecurityScopedResource() else {
            print("Failed to access file resource.")
            return
        }
        defer { url.stopAccessingSecurityScopedResource() }
        
        guard let filename = url.path.removingPercentEncoding else {
            print("Filename is invalid.")
            return
        }
        
        processGenomeData(filename: filename, numLines: numLines)
    }
}


#Preview {
    ContentView()
}
