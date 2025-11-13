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
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    Text(note.created_at.prefix(19))
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                    Text("\(String(note.uuid4.prefix(5))).. \(String(note.id))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Text(makeText(note: note))
                    .font(.body)

                if let url = URL(string: note.url), !note.url.isEmpty {
                    Link(destination: url) {
                        Text(note.url)
                            .foregroundColor(.blue)
                            .lineLimit(3)
                    }
                    .font(.caption)
                }

                VStack {
                    Text("QR Code")
                        .font(.headline)
                    Image(uiImage: image)
                        .interpolation(.none)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxWidth: 300, maxHeight: 300)
                        .padding()
                        .background(Color.white)
                        .cornerRadius(12)
                        .shadow(radius: 4)
                }
                .frame(maxWidth: .infinity)

                Spacer()
            }
            .padding()
            .navigationTitle("Note QR Code")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
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
