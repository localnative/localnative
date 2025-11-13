//
//  NoteRowView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/12/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI
import CoreImage.CIFilterBuiltins

struct NoteRowView: View {
    var note: Note
    @Binding var query: String
    @State private var showingAlert = false
    @State private var showingQRCode = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Button(action: {
                    showingAlert = true
                }) {
                    Image(systemName: "trash")
                        .foregroundColor(.red)
                }
                .buttonStyle(BorderlessButtonStyle())
                .alert(isPresented: $showingAlert) {
                    Alert(
                        title: Text("Delete Note?"),
                        message: Text("Delete \(String(note.uuid4.prefix(5))).. (\(String(note.id)))? This cannot be undone."),
                        primaryButton: .destructive(Text("Delete")) {
                            AppState.ln.run(json_input: """
                                {"action":"delete","rowid":\(note.id),"query":"\(AppState.getQuery())","limit":10,"offset":\(AppState.getOffset())}
                                """)
                            AppState.search(input: AppState.getQuery(), offset: AppState.getOffset())
                        },
                        secondaryButton: .cancel()
                    )
                }

                Spacer()

                HStack(spacing: 6) {
                    ForEach(note.tags.split(separator: ","), id: \.self) { tag in
                        Text(tag)
                            .font(.caption)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(Color.blue.opacity(0.1))
                            .foregroundColor(.blue)
                            .cornerRadius(8)
                            .onTapGesture {
                                query = String(tag)
                                AppState.clearOffset()
                                AppState.search(input: String(tag), offset: 0)
                            }
                    }

                    Button(action: {
                        showingQRCode.toggle()
                    }) {
                        Image(systemName: "qrcode")
                            .foregroundColor(.white)
                            .padding(6)
                            .background(Color.gray)
                            .cornerRadius(6)
                    }
                    .buttonStyle(BorderlessButtonStyle())
                    .sheet(isPresented: $showingQRCode) {
                        QRCodeView(note: note, image: generateQRCode(from: note.url))
                    }
                }
            }

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
                        .font(.caption)
                        .foregroundColor(.blue)
                        .lineLimit(2)
                }
            }
        }
        .padding(.vertical, 4)
    }
    let context = CIContext()
    let filter = CIFilter.qrCodeGenerator()
    func generateQRCode(from string: String) -> UIImage {
        let data = Data(string.utf8)
        filter.setValue(data, forKey: "inputMessage")

        if let outputImage = filter.outputImage {
            if let cgimg = context.createCGImage(outputImage, from: outputImage.extent) {
                return UIImage(cgImage: cgimg)
            }
        }

        return UIImage(systemName: "xmark.circle") ?? UIImage()
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

struct NoteRowView_Previews: PreviewProvider {
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
        ), query: .constant("query"))
    }
}
