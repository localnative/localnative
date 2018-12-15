package app.localnative.android;

import android.util.Log;

import org.json.JSONArray;
import org.json.JSONObject;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Helper class for providing sample content for user interfaces created by
 * Android template wizards.
 * <p>
 * TODO: Replace all uses of this class before publishing your app.
 */
public class NoteContent {

    /**
     * An array of sample (dummy) items.
     */
    public static final List<NoteItem> ITEMS = new ArrayList<NoteItem>();

    /**
     * A map of sample (dummy) items, by ID.
     */
    public static final Map<String, NoteItem> ITEM_MAP = new HashMap<String, NoteItem>();

    public static void refresh(String s){
        try{
            JSONArray notes = new JSONObject(s).getJSONArray("notes");
            ITEMS.clear();
            // first item is a hack as placeholder under toolbar
            addItem(createDummyItem(1));
            for (int i = 0; i < notes.length(); i++ ){
                JSONObject note = notes.getJSONObject(i);
                NoteItem noteItem = new NoteItem(
                        note.getString("rowid"),
                        note.getString("title"),
                        note.getString("url")
                );
                addItem(noteItem);
            }
        }catch (Exception e){
            Log.d("JSON Parse Exception", e.toString());
        }
    }

    private static void addItem(NoteItem item) {
        ITEMS.add(item);
        ITEM_MAP.put(item.id, item);
    }

    private static NoteItem createDummyItem(int position) {
        return new NoteItem(String.valueOf(position), "Item " + position, makeDetails(position));
    }

    private static String makeDetails(int position) {
        StringBuilder builder = new StringBuilder();
        builder.append("Details about Item: ").append(position);
        for (int i = 0; i < position; i++) {
            builder.append("\nMore details information here.");
        }
        return builder.toString();
    }

    /**
     * A dummy item representing a piece of content.
     */
    public static class NoteItem {
        public final String id;
        public final String content;
        public final String details;

        public NoteItem(String id, String content, String details) {
            this.id = id;
            this.content = content;
            this.details = details;
        }

        @Override
        public String toString() {
            return content;
        }
    }
}
