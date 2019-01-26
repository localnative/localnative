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
//  RustLocalNative.swift
//  localnative-ios
//
//  Created by Yi Wang on 10/11/18.
//

class RustLocalNative {
    func run(json_input: String) -> String {
        let result = localnative_run(json_input)
        let swift_result = String(cString: result!)
        localnative_free(UnsafeMutablePointer(mutating: result))
        return swift_result
    }
}
