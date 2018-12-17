package app.localnative.android;

import android.util.Log;

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

    public static void refresh(String s){
        try{
            JSONArray notes = new JSONObject(s).getJSONArray("notes");
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
