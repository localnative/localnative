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
    var body: some View {
        VStack(alignment: .leading){
          
            HStack{
                Button(action:{
                    self.foundResult = false
                    self.scanResult = ""
                }){
                    Text("Rescan QR Code")
                }.disabled(!foundResult)
                Spacer()
                Button(action:{
                    self.syncStatus = AppState.ln.run(json_input: """
                        {"action":"client-sync",
                        "addr":"\(self.scanResult)"}
                        """)
                }){
                     Text("Start Sync")
                }
            }.padding()
                
            Text("Scan or input server address:port below:")
            TextField("xxx.xxx.xxx.xxx:YYYYY ", text: $scanResult).foregroundColor(.blue)
        
            Text("Sync status:")
            Text(syncStatus)
            
            CodeScannerView(codeTypes: [.qr], simulatedData: "0.0.0.0:2345") { result in
                switch result {
                case .success(let code):
                    print("Found code: \(code)")
                    self.scanResult = code
                    self.foundResult = true
                case .failure(let error):
                    self.foundResult = false
                    print(error.localizedDescription)
                }
            }
        }.padding()
    }
}

struct SyncView_Previews: PreviewProvider {
    static var previews: some View {
        SyncView()
    }
}
