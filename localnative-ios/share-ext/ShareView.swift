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
//  ShareView.swift
//  share-ext
//
//  SwiftUI-based share extension interface
//

import SwiftUI

struct ShareView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var title: String = ""
    @State private var url: String = ""
    @State private var tags: String = ""
    @State private var description: String = ""

    var sharedData: SharedData
    var onSave: (String, String, String, String) -> Void
    var onCancel: () -> Void

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Note Information")) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Title")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        TextField("Enter title", text: $title)
                            .textFieldStyle(.roundedBorder)
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        Text("URL")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        TextField("Enter URL", text: $url)
                            .textFieldStyle(.roundedBorder)
                            .autocorrectionDisabled()
                            .textInputAutocapitalization(.never)
                            .keyboardType(.URL)
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Tags")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        TextField("Comma or space separated", text: $tags)
                            .textFieldStyle(.roundedBorder)
                            .autocorrectionDisabled()
                            .textInputAutocapitalization(.never)
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Description")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        TextEditor(text: $description)
                            .frame(minHeight: 100)
                            .overlay(
                                RoundedRectangle(cornerRadius: 8)
                                    .stroke(Color(.systemGray4), lineWidth: 1)
                            )
                    }
                }
            }
            .navigationTitle("Save to Local Native")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        onCancel()
                    }
                }

                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Save") {
                        onSave(title, url, tags, description)
                    }
                    .fontWeight(.semibold)
                }
            }
        }
        .onAppear {
            // Pre-populate with shared data
            title = sharedData.title
            url = sharedData.url
        }
    }
}

struct SharedData {
    var title: String
    var url: String
}

struct ShareView_Previews: PreviewProvider {
    static var previews: some View {
        ShareView(
            sharedData: SharedData(title: "Example Title", url: "https://example.com"),
            onSave: { _, _, _, _ in },
            onCancel: { }
        )
    }
}
