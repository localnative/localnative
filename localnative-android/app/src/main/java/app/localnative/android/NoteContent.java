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
package app.localnative.android;

import android.util.Log;

import com.android.volley.toolbox.JsonObjectRequest;

import org.json.JSONArray;
import org.json.JSONObject;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class NoteContent {

    /**
     * An array of note items.
     */
    public static final List<NoteItem> ITEMS = new ArrayList<NoteItem>();

    /**
     * A map of note items, by rowid.
     */
    public static final Map<Integer, NoteItem> ITEM_MAP = new HashMap<Integer, NoteItem>();

    public static String refresh(String s){
        Long count = null;
        try{
            JSONObject j = new JSONObject(s);
            count = j.getLong("count");
            JSONArray notes = j.getJSONArray("notes");
            ITEMS.clear();
            for (int i = 0; i < notes.length(); i++ ){
                JSONObject note = notes.getJSONObject(i);
                NoteItem noteItem = new NoteItem(
                        note.getInt("rowid"),
                        note.getString("title"),
                        note.getString("url"),
                        note.getString("tags"),
                        note.getString("description"),
                        note.getString("comments"),
                        note.getString("annotations"),
                        note.getString("created_at"),
                        note.getBoolean("is_public")
                );
                addItem(noteItem);
            }
        }catch (Exception e){
            Log.d("JSON Parse Exception", e.toString());
        }
        return count.toString();
    }

    private static void addItem(NoteItem item) {
        ITEMS.add(item);
        ITEM_MAP.put(item.rowid, item);
    }

    /**
     * Note item representing a note.
     */
    public static class NoteItem {
        public final Integer rowid;
        public final String title;
        public final String url;
        public final String tags;
        public final String description;
        public final String comments;
        public final String annotations;
        public final String created_at;
        public final Boolean is_public;

        public NoteItem(Integer rowid, String title, String url, String tags, String description, String comments, String annotations, String created_at, Boolean is_public) {
            this.rowid = rowid;
            this.title = title;
            this.url = url;
            this.tags = tags;
            this.description = description;
            this.comments = comments;
            this.annotations = annotations;
            this.created_at = created_at;
            this.is_public = is_public;
        }

        @Override
        public String toString() {
            return title;
        }
    }
}
