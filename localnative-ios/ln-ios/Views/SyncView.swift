//
//  SyncView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/18/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI

struct SyncView: View {
    var body: some View {
        CodeScannerView(codeTypes: [.qr], simulatedData: "Local Native") { result in
            switch result {
            case .success(let code):
                print("Found code: \(code)")
            case .failure(let error):
                print(error.localizedDescription)
            }
        }
    }
}

struct SyncView_Previews: PreviewProvider {
    static var previews: some View {
        SyncView()
    }
}
