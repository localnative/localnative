//
//  NoteRowView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/12/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI

struct NoteRowView: View {
    var note: Note
    var body: some View {
        Text("\(note.id) \(note.title) \(note.url) \(note.tags) \(note.created_at)")
    }
}

struct NoteRowView_Previews: PreviewProvider {
    static var previews: some View {
        NoteRowView(note: Note(
            id: 0,
            uuid4: "uuid4",
            title: "title",
            url: "url",
            tags: "tag1,tag2",
            created_at: "2020-04-12"
        ))
    }
}
