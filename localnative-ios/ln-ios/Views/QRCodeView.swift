//
//  QRCodeView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/17/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI

struct QRCodeView: View {
    var note: Note
    var image: UIImage

    var body: some View {
        VStack(alignment: .leading){
            HStack{
                Text(note.created_at.prefix(19))
                Spacer()
                Text("\(String(note.uuid4.prefix(5))).. \(String(note.id))")
            }
            Text(makeText(note: note))
            Text(note.url).onTapGesture {
                UIApplication.shared.open(URL(string: self.note.url)!)
            }.foregroundColor(.blue)
            Image(uiImage: image).interpolation(.none)
            .resizable()
            .aspectRatio(contentMode: .fit)
            Spacer()
        }.padding()
    }
    func makeText(note: Note) -> String{
        return note.title
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

struct QRCodeView_Previews: PreviewProvider {
    static var previews: some View {
        QRCodeView(note: Note(
            id: 0,
            uuid4: "uuid4",
            title: "title",
            url: "url",
            tags: "tag1,tag2",
            description: "description",
            annotations: "annotations",
            created_at: "2020-04-12"
        ), image: UIImage())
    }
}
