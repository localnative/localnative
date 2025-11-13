//
//  SyncView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/18/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI

struct SyncView: View {
    @State private var scanResult: String = ""
    @State private var syncStatus: String = ""
    @State private var foundResult: Bool = false
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    Button(action: {
                        foundResult = false
                        scanResult = ""
                    }) {
                        Label("Rescan QR Code", systemImage: "qrcode.viewfinder")
                    }
                    .disabled(!foundResult)
                    .buttonStyle(.bordered)

                    Spacer()

                    Button(action: {
                        syncStatus = AppState.ln.run(json_input: """
                            {"action":"client-sync",
                            "addr":"\(scanResult)"}
                            """)
                    }) {
                        Label("Start Sync", systemImage: "arrow.triangle.2.circlepath")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(scanResult.isEmpty)
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text("Server Address")
                        .font(.headline)
                    Text("Scan QR code or enter address:port manually")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextField("xxx.xxx.xxx.xxx:YYYYY", text: $scanResult)
                        .textFieldStyle(.roundedBorder)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .keyboardType(.URL)
                }

                if !syncStatus.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Sync Status")
                            .font(.headline)
                        ScrollView {
                            Text(syncStatus)
                                .font(.system(.body, design: .monospaced))
                                .padding()
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(Color(.systemGray6))
                                .cornerRadius(8)
                        }
                        .frame(maxHeight: 150)
                    }
                }

                if !foundResult {
                    VStack {
                        Text("Scan QR Code")
                            .font(.headline)
                        CodeScannerView(codeTypes: [.qr], simulatedData: "0.0.0.0:2345") { result in
                            switch result {
                            case .success(let code):
                                print("Found code: \(code)")
                                scanResult = code
                                foundResult = true
                            case .failure(let error):
                                foundResult = false
                                print(error.localizedDescription)
                            }
                        }
                        .frame(height: 300)
                        .cornerRadius(12)
                    }
                }

                Spacer()
            }
            .padding()
            .navigationTitle("Sync")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct SyncView_Previews: PreviewProvider {
    static var previews: some View {
        SyncView()
    }
}
