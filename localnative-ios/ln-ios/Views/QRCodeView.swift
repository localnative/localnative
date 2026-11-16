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
    var showQRCode: Bool = true
    @State private var fullNote: Note?
    @Environment(\.dismiss) private var dismiss

    private var displayNote: Note {
        fullNote ?? note
    }

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                HStack {
                    Text(displayNote.created_at.prefix(19))
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                    Text("\(String(displayNote.uuid4.prefix(5))).. \(String(note.id))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Text(makeText(note: displayNote))
                    .font(.body)

                if let url = URL(string: displayNote.url), !displayNote.url.isEmpty {
                    Link(destination: url) {
                        Text(displayNote.url)
                            .foregroundColor(.blue)
                            .lineLimit(3)
                    }
                    .font(.caption)
                }

                if showQRCode {
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
                }

                Spacer()
            }
            .padding()
        }
        .onAppear {
            if note.has_archive == true && fullNote == nil {
                AppState.getNote(uuid4: note.uuid4) { loaded in
                    fullNote = loaded
                }
            }
        }
        .navigationTitle(showQRCode ? "Note QR Code" : "Archived Text")
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
