//
//  NoteTableViewCell.swift
//  localnative-ios
//
//  Created by Yi Wang on 10/18/18.
//  Copyright © 2018 Yi Wang. All rights reserved.
//

import UIKit

class NoteTableViewCell: UITableViewCell {

    @IBOutlet weak var contentText: UITextView!
    @IBOutlet weak var urlText: UITextView!
    override func awakeFromNib() {
        super.awakeFromNib()
        // Initialization code
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        // Configure the view for the selected state
    }

}
