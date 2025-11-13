/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
//
//  LocalNativeApp.swift
//  ln-ios
//
//  SwiftUI App lifecycle
//

import SwiftUI
import MMWormhole

@main
struct LocalNativeApp: App {
    @StateObject private var env = AppState.getEnv()
    let wormhole = MMWormhole(applicationGroupIdentifier: "group.app.localnative.ios", optionalDirectory: "wormhole")

    init() {
        // Set up wormhole listener
        wormhole.listenForMessage(withIdentifier: "message", listener: { (messageObject) -> Void in
            if let message = messageObject as? String {
                AppState.ln.run(json_input: message)
                AppState.search(input: "", offset: 0)
            }
        })

        // Perform initial search
        AppState.search(input: "", offset: 0)
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(env)
        }
    }
}
