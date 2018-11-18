//
//  ShareViewController.swift
//  share-ext
//
//  Created by Yi Wang on 9/16/18.
//  Copyright © 2018 Yi Wang. All rights reserved.
//
//  some inital code from https://hackernoon.com/how-to-build-an-ios-share-extension-in-swift-4a2019935b2e
//

import MobileCoreServices
import UIKit
import Social
let ln = RustLocalNative()
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
    
    @IBAction func saveButtonTouchDown(_ sender: Any) {
        self.extensionContext!.completeRequest(returningItems: [], completionHandler: nil)
    }
    
    override func viewDidLoad() {
        print("viewDidLoad")
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
