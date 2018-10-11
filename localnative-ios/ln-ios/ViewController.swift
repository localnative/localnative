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
{
"action": "search"
}
"""
        )

        searchText.text = txt
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

