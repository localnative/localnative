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
package app.localnative.android

import android.content.Context
import android.util.Log

object RustBridge {
    private var initialized = false

    /** Load the Rust core and point it at this app's own database file.
     *  Must run before any [run]; safe to call more than once. */
    @Synchronized
    fun init(context: Context) {
        if (initialized) return
        System.loadLibrary("localnative_core")
        val db = context.filesDir.resolve("localnative.sqlite3")
        localnativeSetDbPath(db.absolutePath)
        initialized = true
        Log.i("RustBridge", "database at ${db.absolutePath}")
    }

    private external fun localnativeRun(pattern: String): String
    private external fun localnativeSetDbPath(path: String)

    fun run(input: String): String {
        check(initialized) { "RustBridge.init(context) must run before run()" }
        return localnativeRun(input)
    }
}
