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
//  ViewController.swift
//  ln-ios
//
//  Created by Yi Wang on 9/16/18.
//

import UIKit
let ln = RustLocalNative()

class ViewController: UIViewController, UITableViewDataSource, UITableViewDelegate, UISearchBarDelegate, UIToolbarDelegate {
    @IBOutlet weak var searchInput: UISearchBar!
    @IBOutlet weak var tableView: UITableView!
    @IBOutlet weak var prevButton: UIBarButtonItem!
    @IBOutlet weak var nextButton: UIBarButtonItem!
    @IBOutlet weak var paginationButton: UIBarButtonItem!
    var notes : NSArray = []
    
    @IBAction func prevButtonTouchDown(_ sender: Any){
        let offset = AppState.decOffset()
        search(input: AppState.getQuery(), offset: offset)
    }
    @IBAction func nextButtonTouchDown(_ sender: Any){
        let offset = AppState.incOffset()
        search(input: AppState.getQuery(), offset: offset)
    }

    
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
    
    func searchBar(_ searchInput: UISearchBar, textDidChange searchText: String) {
        AppState.clearOffset()
        search(input: searchText, offset: 0)
    }
    
    func search(input: String, offset: Int64){
        AppState.setQuery(query: input)
        let txt = ln.run(json_input:"""
            {"action":"search","query":"\(input)","limit":10,"offset":\(offset)}
            """
        )
        let data = txt.data(using: .utf8)!
        if let jsonObject = try? JSONSerialization.jsonObject(with: data) as? [String: NSObject] {
            notes =  jsonObject!["notes"] as! NSArray
            let count = jsonObject!["count"] as! Int64
            AppState.setCount(count: count)
            paginationButton.title = AppState.makePaginationText()
        }
        self.tableView.reloadData()
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()
        // Do any additional setup after loading the view, typically from a nib.
        /* Setup delegates */
        tableView.delegate = self
        tableView.dataSource = self
        searchInput.delegate = self
        // search with empty string first to show content
        search(input: "", offset: 0)
    }

    override func didReceiveMemoryWarning() {
        super.didReceiveMemoryWarning()
        // Dispose of any resources that can be recreated.
    }


}

