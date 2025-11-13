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
//  ShareViewController.swift
//  share-ext
//
//  Created by Yi Wang on 9/16/18.
//
//

import MobileCoreServices
import UIKit
import SwiftUI
import UniformTypeIdentifiers

class ShareViewController: UIViewController {

    private var sharedTitle: String = ""
    private var sharedURL: String = ""

    override func viewDidLoad() {
        super.viewDidLoad()

        // Extract shared content from extension context
        extractSharedContent { [weak self] title, url in
            guard let self = self else { return }
            self.sharedTitle = title
            self.sharedURL = url
            self.setupSwiftUIView()
        }
    }

    private func setupSwiftUIView() {
        let sharedData = SharedData(title: sharedTitle, url: sharedURL)

        let shareView = ShareView(
            sharedData: sharedData,
            onSave: { [weak self] title, url, tags, description in
                self?.saveNote(title: title, url: url, tags: tags, description: description)
            },
            onCancel: { [weak self] in
                self?.cancelShare()
            }
        )

        let hostingController = UIHostingController(rootView: shareView)
        addChild(hostingController)
        view.addSubview(hostingController.view)
        hostingController.view.frame = view.bounds
        hostingController.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        hostingController.didMove(toParent: self)
    }

    private func extractSharedContent(completion: @escaping (String, String) -> Void) {
        guard let extensionItem = extensionContext?.inputItems.first as? NSExtensionItem,
              let itemProvider = extensionItem.attachments?.first else {
            completion("", "")
            return
        }

        // Try to get URL and title from different content types
        let propertyList = String(kUTTypePropertyList)

        if itemProvider.hasItemConformingToTypeIdentifier(propertyList) {
            itemProvider.loadItem(forTypeIdentifier: propertyList, options: nil) { item, error in
                guard let dictionary = item as? NSDictionary,
                      let results = dictionary[NSExtensionJavaScriptPreprocessingResultsKey] as? NSDictionary else {
                    DispatchQueue.main.async {
                        completion("", "")
                    }
                    return
                }

                let title = results["title"] as? String ?? ""
                let url = results["url"] as? String ?? ""

                DispatchQueue.main.async {
                    completion(title, url)
                }
            }
        } else if itemProvider.hasItemConformingToTypeIdentifier(UTType.url.identifier) {
            itemProvider.loadItem(forTypeIdentifier: UTType.url.identifier, options: nil) { item, error in
                guard let url = item as? URL else {
                    DispatchQueue.main.async {
                        completion("", "")
                    }
                    return
                }

                DispatchQueue.main.async {
                    completion(url.absoluteString, url.absoluteString)
                }
            }
        } else if itemProvider.hasItemConformingToTypeIdentifier(UTType.plainText.identifier) {
            itemProvider.loadItem(forTypeIdentifier: UTType.plainText.identifier, options: nil) { item, error in
                guard let text = item as? String else {
                    DispatchQueue.main.async {
                        completion("", "")
                    }
                    return
                }

                DispatchQueue.main.async {
                    completion(text, text)
                }
            }
        } else {
            completion("", "")
        }
    }

    private func saveNote(title: String, url: String, tags: String, description: String) {
        let message: [String: Any] = [
            "action": "insert",
            "title": title,
            "url": url,
            "tags": tags,
            "description": description,
            "comments": "",
            "annotations": "",
            "limit": 10,
            "offset": 0,
            "is_public": false
        ]

        if let jsonData = try? JSONSerialization.data(withJSONObject: message),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            // Send message to main app using App Groups
            AppGroupsHelper.shared.sendMessage(jsonString)

            // Try to open the main app
            if let appURL = URL(string: "localnative://insert") {
                var responder: UIResponder? = self
                while responder != nil {
                    if let application = responder as? UIApplication {
                        application.perform(#selector(openURL(_:)), with: appURL)
                        break
                    }
                    responder = responder?.next
                }
            }
        }

        // Complete the extension request
        extensionContext?.completeRequest(returningItems: [], completionHandler: nil)
    }

    private func cancelShare() {
        extensionContext?.completeRequest(returningItems: [], completionHandler: nil)
    }

    @objc func openURL(_ url: URL) -> Bool {
        var responder: UIResponder? = self
        while responder != nil {
            if let application = responder as? UIApplication {
                return application.perform(#selector(openURL(_:)), with: url) != nil
            }
            responder = responder?.next
        }
        return false
    }
}
