//
//  ContentView.swift
//  localnative-ios
//
//  Created by Yi Wang on 4/12/20.
//  Copyright © 2020 Yi Wang. All rights reserved.
//

import SwiftUI

struct ContentView: View {
    @State private var searchText : String = ""
    @EnvironmentObject var env : Env
    
    var body: some View {
        NavigationView {
            VStack {
                Text("Own your bookmarks on your device.")
                SearchBar(text: $searchText, placeholder: "type to search")
                HStack{
                    Button(action:{
                        let offset = AppState.decOffset()
                        AppState.search(input: AppState.getQuery(), offset: offset)
                    }){
                        Text("Prev")
                    }
                    Text(env.paginationText)
                    Button(action:{
                        let offset = AppState.incOffset()
                        AppState.search(input: AppState.getQuery(), offset: offset)
                    }){
                        Text("Next")
                    }
                }
                List (env.notes){
                    note in
                    NoteRowView(note: note)

                }.navigationBarTitle(Text("Local Native"))
                Button(action:{
                    print("sync")
                }){
                    Text("Sync")
                }
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}

struct SearchBar: UIViewRepresentable {
    @Binding var text: String
    var placeholder: String
    
    class Coordinator: NSObject, UISearchBarDelegate {
        
        @Binding var text: String

        init(text: Binding<String>) {
            _text = text
        }

        func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
            AppState.clearOffset()
            AppState.search(input: searchText, offset: 0)
        }
    }

    func makeCoordinator() -> SearchBar.Coordinator {
        return Coordinator(text: $text)
    }

    func makeUIView(context: UIViewRepresentableContext<SearchBar>) -> UISearchBar {
        let searchBar = UISearchBar(frame: .zero)
        searchBar.delegate = context.coordinator
        searchBar.placeholder = placeholder
        searchBar.searchBarStyle = .minimal
        searchBar.autocapitalizationType = .none
        return searchBar
    }

    func updateUIView(_ uiView: UISearchBar, context: UIViewRepresentableContext<SearchBar>) {
        uiView.text = text
    }
}
