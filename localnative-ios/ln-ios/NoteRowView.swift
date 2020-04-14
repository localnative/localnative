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
        VStack(alignment: .leading){
            HStack{
                Button(action:{
                    print("X")
                }){
                    Text("X")
                }
                Text("\(note.tags)")
            }
            Text("\(note.created_at) rowid \(note.id)")
            Text(note.title)
            Text(note.url)
        }
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
