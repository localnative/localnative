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
                SearchBar(text: $searchText, placeholder: "type to search")
                List (env.notes){
                    note in
                    NoteRowView(note: note)

                }.navigationBarTitle(Text("Local Native"))
                Button(action:{
                    let ln = RustLocalNative()
                    let txt = "txt"
                    let tags = "tags"
                    let resp = ln.run(json_input:"""
                        {"action":"insert",
                        "title":"\(txt)",
                        "url": "https://localnative.app",
                        "tags":"\(tags)",

                        "description": "desc",
                        "comments": "",
                        "annotations": "",
                        
                        "limit": 10,
                        "offset": 0,
                        "is_public": false
                        }
                        """
                    )
                    print(resp)
                }){
                    Text("New Note")
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
