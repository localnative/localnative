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
    
    @objc func buttonClicked(sender : UIButton){
        let alert = UIAlertController(title: "Clicked", message: "You have clicked on the button", preferredStyle: .alert)
        
        //self.present(alert, animated: true, completion: nil)
    }
    
    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "NoteTableViewCell") as! NoteTableViewCell
        if(indexPath.row < notes.count){
            let note = notes[indexPath.row] as! [String:Any]
            let rowid = note["rowid"] as! NSNumber
            for view in cell.tagsContainer.subviews {
                view.removeFromSuperview()
            }
            // delete button
            let deleteButton = UIButton(type: .system)
            deleteButton.tag = -1
            deleteButton.frame = CGRect(x: 0, y: 0, width: 40, height: 35)
            deleteButton.setTitle("X", for: .normal)
            deleteButton.tintColor =  .red
            deleteButton.addTarget(self, action: #selector(buttonClicked), for: .touchUpInside)
            cell.tagsContainer.addSubview(deleteButton)
            
            for (i,tag) in (note["tags"] as! String).components(separatedBy: ",").enumerated(){
                let tagButton = UIButton(type: .system)
                tagButton.tag = rowid.intValue
                tagButton.frame = CGRect(x: 100*i + 40, y: 0, width: 100, height: 35)
                tagButton.setTitle(tag, for: .normal)
                tagButton.addTarget(self, action: #selector(buttonClicked), for: .touchUpInside)
                cell.tagsContainer.addSubview(tagButton)
            }
            cell.tagsContainer.setNeedsLayout()
            
            cell.contentText.text = (note["created_at"] as! String) + " rowid " + rowid.stringValue
                + "\n" + (note["title"] as! String)
                + newLineOrEmptyString(str: note["description"] as! String)
                + newLineOrEmptyString(str: note["annotations"] as! String)
            cell.urlText.text = (note["url"] as! String)
        }
        return cell;
    }
    
    func newLineOrEmptyString(str: String) -> String{
        if(str.trimmingCharacters(in: .whitespacesAndNewlines) == ""){
            return ""
        }else{
            return "\n" + str
        }
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
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 600
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

