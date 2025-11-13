//
//  ContentView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/12/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI

struct ContentView: View {
    @State private var searchText: String = ""
    @EnvironmentObject var env: Env
    @State private var showingSync = false

    var body: some View {
        NavigationView {
            VStack {
                HStack {
                    Button(action: {
                        let offset = AppState.decOffset()
                        AppState.search(input: AppState.getQuery(), offset: offset)
                    }) {
                        Text("Prev")
                    }
                    .padding(.leading)

                    Spacer()
                    Text(env.paginationText)
                    Spacer()

                    Button(action: {
                        let offset = AppState.incOffset()
                        AppState.search(input: AppState.getQuery(), offset: offset)
                    }) {
                        Text("Next")
                    }
                    .padding(.trailing)
                }

                List(env.notes) { note in
                    NoteRowView(note: note, query: $searchText)
                }
                .listStyle(.plain)

                HStack {
                    Text("Write your own geolocation notes with")
                    Button(action: {
                        guard let url = URL(string: "https://hexagon.place/") else { return }
                        UIApplication.shared.open(url)
                    }) {
                        Text("Hexagon Place App")
                    }
                }
                .font(.caption)
                .padding()
            }
            .navigationTitle("LN")
            .navigationBarTitleDisplayMode(.inline)
            .searchable(text: $searchText, prompt: "Search notes")
            .onChange(of: searchText) { newValue in
                AppState.clearOffset()
                AppState.search(input: newValue, offset: 0)
            }
            .autocorrectionDisabled()
            .textInputAutocapitalization(.never)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        showingSync.toggle()
                    }) {
                        Image(systemName: "arrow.triangle.2.circlepath")
                    }
                }
            }
            .sheet(isPresented: $showingSync) {
                SyncView()
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
