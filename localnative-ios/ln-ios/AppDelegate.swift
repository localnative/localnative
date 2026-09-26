//
//  AppDelegate.swift
//  ln-ios
//
//  Created by Yi Wang on 9/16/18.
//

import UIKit

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Shares from the share extension arrive as JSON commands queued in
        // the App Group; apply them and refresh the list. The queue is also
        // drained on launch, so shares made while the app was closed land too.
        AppGroupsHelper.shared.startListening { messages in
            for message in messages {
                _ = AppState.ln.run(json_input: message)
            }
            AppState.search(input: "", offset: 0)
        }
        return true
    }

    // MARK: UISceneSession Lifecycle

    func application(_ application: UIApplication, configurationForConnecting connectingSceneSession: UISceneSession, options: UIScene.ConnectionOptions) -> UISceneConfiguration {
        // Called when a new scene session is being created.
        // Use this method to select a configuration to create the new scene with.
        return UISceneConfiguration(name: "Default Configuration", sessionRole: connectingSceneSession.role)
    }

    func application(_ application: UIApplication, didDiscardSceneSessions sceneSessions: Set<UISceneSession>) {
        // Called when the user discards a scene session.
        // If any sessions are discarded while the application is not running, this will be called shortly after application:didFinishLaunchingWithOptions.
        // Use this method to release any resources that were specific to the discarded scenes, as they will not return.
    }

}
