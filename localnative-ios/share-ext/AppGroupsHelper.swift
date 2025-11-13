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
//  AppGroupsHelper.swift
//  share-ext
//
//  Helper for App Groups communication (replaces MMWormhole)
//

import Foundation

class AppGroupsHelper {
    static let shared = AppGroupsHelper()

    private let appGroupIdentifier = "group.app.localnative.ios"
    private let messageKey = "shared_message"
    private let timestampKey = "message_timestamp"

    private var sharedDefaults: UserDefaults? {
        return UserDefaults(suiteName: appGroupIdentifier)
    }

    /// Send a message from share extension to main app
    func sendMessage(_ message: String) {
        guard let defaults = sharedDefaults else {
            print("Failed to access shared UserDefaults")
            return
        }

        defaults.set(message, forKey: messageKey)
        defaults.set(Date().timeIntervalSince1970, forKey: timestampKey)
        defaults.synchronize()

        // Also post a Darwin notification to wake up the main app if needed
        CFNotificationCenterPostNotification(
            CFNotificationCenterGetDarwinNotifyCenter(),
            CFNotificationName("app.localnative.ios.message" as CFString),
            nil,
            nil,
            true
        )
    }

    /// Read the latest message (for main app)
    func readMessage() -> String? {
        guard let defaults = sharedDefaults else {
            return nil
        }

        return defaults.string(forKey: messageKey)
    }

    /// Clear the message after reading
    func clearMessage() {
        guard let defaults = sharedDefaults else {
            return
        }

        defaults.removeObject(forKey: messageKey)
        defaults.removeObject(forKey: timestampKey)
        defaults.synchronize()
    }

    /// Start listening for messages (for main app)
    func startListening(callback: @escaping (String) -> Void) {
        // Listen for Darwin notifications
        let notificationName = CFNotificationName("app.localnative.ios.message" as CFString)
        let observer = UnsafeRawPointer(Unmanaged.passUnretained(self).toOpaque())

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDarwinNotifyCenter(),
            observer,
            { (center, observer, name, object, userInfo) in
                guard let helper = observer.map({ Unmanaged<AppGroupsHelper>.fromOpaque($0).takeUnretainedValue() }) else {
                    return
                }

                if let message = helper.readMessage() {
                    // Execute callback on main thread
                    DispatchQueue.main.async {
                        callback(message)
                    }
                }
            },
            notificationName.rawValue,
            nil,
            .deliverImmediately
        )
    }
}
