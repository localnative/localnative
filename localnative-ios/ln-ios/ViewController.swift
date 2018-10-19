//
//  ViewController.swift
//  ln-ios
//
//  Created by Yi Wang on 9/16/18.
//  Copyright © 2018 Yi Wang. All rights reserved.
//

import UIKit
let ln = RustLocalNative()
class ViewController: UIViewController {

    @IBOutlet var searchButton: UIButton!
    @IBOutlet var searchText: UITextField!
    @IBAction func onClick(_ sender: UIButton) {

        let txt = ln.run(json_input:"""
{"action":"select","limit":10,"offset":0}
"""
        )

        searchText.text = txt
        let data = txt.data(using: .utf8)!
        if let notes = try? JSONSerialization.jsonObject(with: data) as? [String: NSArray] {
            for note in notes!["notes"]! {
                print(note)
            }
        }
    }
    override func viewDidLoad() {
        super.viewDidLoad()
        // Do any additional setup after loading the view, typically from a nib.

    }

    override func didReceiveMemoryWarning() {
        super.didReceiveMemoryWarning()
        // Dispose of any resources that can be recreated.
    }


}

