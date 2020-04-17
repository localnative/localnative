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
                Text("X").onTapGesture {
                        print("X")
                }.foregroundColor(.red)
                ForEach(note.tags.split(separator: ","), id:\.self){
                    tag in
                    Text(tag).onTapGesture {
                        AppState.clearOffset()
                        AppState.search(input: String(tag), offset: 0)
                    }.foregroundColor(.blue)
                }
            }
            Text(makeText(note: note))
            Text(note.url).onTapGesture {
                UIApplication.shared.open(URL(string: self.note.url)!)
            }.foregroundColor(.blue)
        }
    }
    func makeText(note: Note) -> String{
        let t = "\(String(note.created_at.prefix(19))) \(String(note.uuid4.prefix(5))).. \(String(note.id))"
        return t + newLineOrEmptyString(str: note.title)
            + newLineOrEmptyString(str: note.description)
            + newLineOrEmptyString(str: note.annotations)
    }
    func newLineOrEmptyString(str: String) -> String{
        if(str.trimmingCharacters(in: .whitespacesAndNewlines) == ""){
            return ""
        }else{
            return "\n" + str
        }
    }
}

struct NoteRowView_Previews: PreviewProvider {
    @Binding var query: String
    static var previews: some View {
        NoteRowView(note: Note(
            id: 0,
            uuid4: "uuid4",
            title: "title",
            url: "url",
            tags: "tag1,tag2",
            description: "description",
            annotations: "annotations",
            created_at: "2020-04-12"
        ))
    }
}
