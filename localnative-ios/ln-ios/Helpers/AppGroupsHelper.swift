//
//  AppGroupsHelper.swift
//  ln-ios
//
//  Helper for App Groups communication between the share extension and the
//  main app.
//
//  Shares are a queue, not a slot: a second share no longer overwrites the
//  first, and shares made while the app is closed are delivered on next
//  launch (the app drains the queue on startup, on entering the foreground,
//  and on every Darwin notification).

import Foundation

class AppGroupsHelper {
    static let shared = AppGroupsHelper()

    private let appGroupIdentifier = "group.app.localnative.ios"
    private let queueKey = "shared_messages"

    private var sharedDefaults: UserDefaults? {
        return UserDefaults(suiteName: appGroupIdentifier)
    }

    /// Append a message to the shared queue (share extension side).
    func sendMessage(_ message: String) {
        guard let defaults = sharedDefaults else {
            print("Failed to access shared UserDefaults")
            return
        }

        var queue = defaults.stringArray(forKey: queueKey) ?? []
        queue.append(message)
        defaults.set(queue, forKey: queueKey)
        defaults.synchronize()

        // Wake the main app if it is running.
        CFNotificationCenterPostNotification(
            CFNotificationCenterGetDarwinNotifyCenter(),
            CFNotificationName("app.localnative.ios.message" as CFString),
            nil,
            nil,
            true
        )
    }

    /// Read and clear every queued message (main app side).
    @discardableResult
    func drainMessages() -> [String] {
        guard let defaults = sharedDefaults else {
            return []
        }
        let queue = defaults.stringArray(forKey: queueKey) ?? []
        if !queue.isEmpty {
            defaults.removeObject(forKey: queueKey)
            defaults.synchronize()
        }
        return queue
    }

    /// The handler registered through `startListening`.
    private var messageCallback: (([String]) -> Void)?

    private func deliverQueued() {
        let messages = drainMessages()
        if !messages.isEmpty {
            DispatchQueue.main.async {
                self.messageCallback?(messages)
            }
        }
    }

    /// Deliver queued messages now and on every later notification.
    func startListening(callback: @escaping ([String]) -> Void) {
        self.messageCallback = callback

        let notificationName = CFNotificationName("app.localnative.ios.message" as CFString)
        // Darwin callbacks are C function pointers and cannot capture
        // context, so the helper is passed through as the observer pointer.
        let observer = UnsafeRawPointer(Unmanaged.passUnretained(self).toOpaque())
        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDarwinNotifyCenter(),
            observer,
            { (_, observer, _, _, _) in
                guard let observer = observer else { return }
                let helper = Unmanaged<AppGroupsHelper>.fromOpaque(observer).takeUnretainedValue()
                helper.deliverQueued()
            },
            notificationName.rawValue,
            nil,
            .deliverImmediately
        )
        // Shares queued while the app was closed.
        deliverQueued()
    }
}
