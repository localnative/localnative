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

import android.util.Log

import org.json.JSONObject
import java.io.Serializable

import java.util.ArrayList
import java.util.HashMap

object NoteContent {

    const val NOTE_ITEM = "note_item"

    /**
     * An array of note items.
     */
    val ITEMS: MutableList<NoteItem> = ArrayList()

    /**
     * A map of note items, by rowid.
     */
    val ITEM_MAP: MutableMap<Int, NoteItem> = HashMap()

    fun refresh(s: String): Long? {
        var count: Long? = 0L
        try {
            val j = JSONObject(s)
            count = j.getLong("count")
            val notes = j.getJSONArray("notes")
            ITEMS.clear()
            for (i in 0 until notes.length()) {
                val note = notes.getJSONObject(i)
                val noteItem = NoteItem(
                        note.getInt("rowid"),
                        note.getString("uuid4"),
                        note.getString("title"),
                        note.getString("url"),
                        note.getString("tags"),
                        note.getString("description"),
                        note.getString("comments"),
                        note.getString("annotations"),
                        note.getString("created_at"),
                        note.getBoolean("is_public")
                )
                addItem(noteItem)
            }
        } catch (e: Exception) {
            Log.d("JSON Parse Exception", e.toString())
        }

        return count
    }

    private fun addItem(item: NoteItem) {
        ITEMS.add(item)
        ITEM_MAP[item.rowid!!] = item
    }

    /**
     * Note item representing a note.
     */
    class NoteItem(val rowid: Int?, val uuid4: String, val title: String, val url: String,
                   val tags: String, val description: String, val comments: String,
                   val annotations: String, val created_at: String, val is_public: Boolean?) :
        Serializable {

        override fun toString(): String {
            return title
        }
    }
}
