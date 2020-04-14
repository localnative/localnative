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
//  AppState.swift
//  localnative-ios
//
//  Created by Yi Wang on 3/10/19.
//

import Foundation

class AppState {
    static let LIMIT :Int64 = 10
    static var offset :Int64 = 0
    static var count :Int64 = 0
    static var query = ""
    static func makePaginationText()-> String {
        let start = count > 0 ? offset + 1 : 0
        let end : Int64 = offset + LIMIT > count ? count : offset + LIMIT
        
//        let p: Int64 = Int64(ceil((0.0 + Double(end)) / Double(LIMIT)))
//        let z: Int64 = Int64(ceil((Double(count) + 0.0) / Double(LIMIT)))
        env.paginationText = "\(start)-\(end)/\(count)"
        return "\(start)-\(end)/\(count)"
    }
    static func getQuery() -> String{
        return query
    }
    static func setQuery(query: String) {
        self.query = query
    }
    static func setCount(count: Int64) {
        self.count = count
    }

    static func incOffset()-> Int64 {
        if(offset + LIMIT < count) {
            offset += LIMIT
        }
        return offset
    }
    static func decOffset()-> Int64 {
        if(offset - LIMIT >= 0) {
            offset -= LIMIT
        }
        return offset
    }
    static func clearOffset() {
        offset = 0
    }
    
    static func getOffset() -> Int64 {
        return offset
    }
    
    static let env = Env()
    static func getEnv()->Env{
        return env
    }
    
    static let ln = RustLocalNative()
    static func search(input: String, offset: Int64) {
        AppState.setQuery(query: input)
        let txt = ln.run(json_input:"""
            {"action":"search","query":"\(input)","limit":10,"offset":\(offset)}
            """
        )
        let data = txt.data(using: .utf8)!
        let decoder = JSONDecoder()
        do {
            let resp = try decoder.decode(Response.self, from: data)
            AppState.setCount(count: resp.count)
            AppState.env.notes = resp.notes
        } catch {
            print(error.localizedDescription)
        }
        makePaginationText()
    }
}

struct Note: Codable, Identifiable {
    var id: Int64
    var uuid4: String
    var title: String
    var url: String
    var tags: String
    var created_at: String
    private enum CodingKeys: String, CodingKey {
        case id = "rowid"
        case uuid4
        case title
        case url
        case tags
        case created_at
    }
}

struct Response: Decodable {
    let count: Int64
    let notes: [Note]
}

class Env: ObservableObject {
    @Published var notes:[Note] = []
    @Published var paginationText:String = "/"
}
