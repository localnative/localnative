//
//  RustLocalNative.swift
//  localnative-ios
//
//  Created by e on 10/11/18.
//  Copyright © 2018 Yi Wang. All rights reserved.
//

class RustLocalNative {
    func run(json_input: String) -> String {
        let result = localnative_run(json_input)
        let swift_result = String(cString: result!)
        localnative_free(UnsafeMutablePointer(mutating: result))
        return swift_result
    }
}
