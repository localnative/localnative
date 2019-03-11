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
import Social
import UITextView_Placeholder
import MMWormhole
let wormhole = MMWormhole(applicationGroupIdentifier: "group.app.localnative.ios", optionalDirectory: "wormhole")

class ShareViewController: UIViewController {
    @IBOutlet weak var saveButton: UIButton!
    @IBOutlet weak var cancelButton: UIButton!
    @IBOutlet weak var titleText: UITextView!
    @IBOutlet weak var urlText: UITextView!
    @IBOutlet weak var tagsText: UITextView!
    @IBOutlet weak var descriptionText: UITextView!
    @IBAction func cancelButtonTouchDown(_ sender: Any) {
        self.extensionContext!.completeRequest(returningItems: [], completionHandler: nil)
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
    
    @IBAction func saveButtonTouchDown(_ sender: Any) {
        let message : [String: Any] = [
            "action": "insert",
            
            "title": titleText.text,
            "url": urlText.text,
            "tags": tagsText.text,
            "description": descriptionText.text,
            "comments": "",
            "annotations": "",
            
            "limit": 10,
            "offset": 0,
            "is_public": false
        ]

        let valid = JSONSerialization.isValidJSONObject(message)
        
        let url = URL(string: "localnative://insert") as! URL
        openURL(url)
        // send json after container app open
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
           if valid {
                let jsonText = try? JSONSerialization.data(withJSONObject: message)
                wormhole.passMessageObject( String(data: jsonText!, encoding: .utf8)! as NSCoding, identifier: "message")
            }
            self.extensionContext!.completeRequest(returningItems: [], completionHandler: nil)
        }

    }
    
    override func viewDidLoad() {
        titleText.placeholder = "title"
        urlText.placeholder = "url"
        tagsText.placeholder = "type to add tags, enter to save, comma or space as tag seperator"
        descriptionText.placeholder = "description"
        tagsText.becomeFirstResponder()
        // https://hackernoon.com/how-to-build-an-ios-share-extension-in-swift-4a2019935b2e
        let extensionItem = extensionContext?.inputItems.first as! NSExtensionItem
        let itemProvider = extensionItem.attachments?.first as! NSItemProvider
        let propertyList = String(kUTTypePropertyList)
        if itemProvider.hasItemConformingToTypeIdentifier(propertyList) {
            itemProvider.loadItem(forTypeIdentifier: propertyList, options: nil, completionHandler: { (item, error) -> Void in
                guard let dictionary = item as? NSDictionary else { return }
                OperationQueue.main.addOperation {
                    if let results = dictionary[NSExtensionJavaScriptPreprocessingResultsKey] as? NSDictionary
                    {
                        print("results")
                        print(results)
                        self.urlText.text =  results["url"] as? String
                        self.titleText.text = results["title"] as? String
                        
                    }
                }
            })
        } else {
            print("error")
        }
    }
    func isContentValid() -> Bool {
        // Do validation of contentText and/or NSExtensionContext attachments here
        return true
    }
    
    func didSelectPost() {
        // This is called after the user selects Post. Do the upload of contentText and/or NSExtensionContext attachments.
    
        // Inform the host that we're done, so it un-blocks its UI. Note: Alternatively you could call super's -didSelectPost, which will similarly complete the extension context.
        self.extensionContext!.completeRequest(returningItems: [], completionHandler: nil)
    }

    func configurationItems() -> [Any]! {
        // To add configuration options via table cells at the bottom of the sheet, return an array of SLComposeSheetConfigurationItem here.
        return []
    }

}
