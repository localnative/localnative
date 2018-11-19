//
//  ViewController.swift
//  ln-ios
//
//  Created by Yi Wang on 9/16/18.
//  Copyright © 2018 Yi Wang. All rights reserved.
//

import UIKit
let ln = RustLocalNative()

class ViewController: UIViewController, UITableViewDataSource, UITableViewDelegate, UISearchBarDelegate {
    @IBOutlet weak var searchBar: UISearchBar!
    @IBOutlet weak var tableView: UITableView!
    var notes : NSArray = []
    
    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        return notes.count
    }
    
    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "NoteTableViewCell") as! NoteTableViewCell
        if(indexPath.row < notes.count){
            let note = notes[indexPath.row] as! [String:Any]
            cell.contentText.text = (note["tags"] as! String)
                + "\n" + (note["created_at"] as! String) + " rowid " + (note["rowid"] as! NSNumber).stringValue
                + "\n" + (note["title"] as! String)
                + "\n" + (note["description"] as! String)
                + "\n" + (note["annotations"] as! String)
            cell.urlText.text = (note["url"] as! String)
        }
        return cell;
    }
    
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        let txt = ln.run(json_input:"""
            {"action":"search","query":"\(searchText)","limit":10,"offset":0}
            """
        )
        let data = txt.data(using: .utf8)!
        if let jsonNotes = try? JSONSerialization.jsonObject(with: data) as? [String: NSArray] {
            notes =  jsonNotes!["notes"]!
        }
        self.tableView.reloadData()
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()
        // Do any additional setup after loading the view, typically from a nib.
        /* Setup delegates */
        tableView.delegate = self
        tableView.dataSource = self
        searchBar.delegate = self
    }

    override func didReceiveMemoryWarning() {
        super.didReceiveMemoryWarning()
        // Dispose of any resources that can be recreated.
    }


}

