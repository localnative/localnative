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

import androidx.fragment.app.DialogFragment;
import androidx.fragment.app.FragmentManager;
import androidx.recyclerview.widget.RecyclerView;

import android.app.AlertDialog;
import android.app.Dialog;
import android.content.Context;
import android.content.DialogInterface;
import android.graphics.Color;
import android.os.Bundle;
import android.util.Log;
import android.util.TypedValue;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

import app.localnative.R;
import app.localnative.android.NoteListFragment.OnListFragmentInteractionListener;
import app.localnative.android.NoteContent.NoteItem;

import java.util.List;

/**
 * {@link RecyclerView.Adapter} that can display a {@link NoteItem} and makes a call to the
 * specified {@link OnListFragmentInteractionListener}.
 * TODO: Replace the implementation with code for your data type.
 */
public class NoteRecyclerViewAdapter extends RecyclerView.Adapter<NoteRecyclerViewAdapter.ViewHolder> {

    private final List<NoteItem> mValues;
    private final OnListFragmentInteractionListener mListener;
    private Context context;

    public NoteRecyclerViewAdapter(List<NoteItem> items, OnListFragmentInteractionListener listener) {
        mValues = items;
        mListener = listener;
    }

    @Override
    public ViewHolder onCreateViewHolder(ViewGroup parent, int viewType) {
        context = parent.getContext();
        View view = LayoutInflater.from(context)
                .inflate(R.layout.fragment_note, parent, false);
        return new ViewHolder(view);
    }

    @Override
    public void onBindViewHolder(final ViewHolder holder, int position) {
        holder.mItem = mValues.get(position);
        final NoteItem note = mValues.get(position);
        holder.mTagsContainer.removeAllViews();
        String[] arr = note.tags.split(",");
        Button deleteButton = new Button(context);
        deleteButton.setText("X");
        int height = (int) TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, 45, context.getResources().getDisplayMetrics());
        int width = (int) TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, 45, context.getResources().getDisplayMetrics());
        deleteButton.setLayoutParams(new RecyclerView.LayoutParams(width, height));
        deleteButton.setTextColor(Color.RED);
        deleteButton.setOnClickListener(new View.OnClickListener(){
            @Override
            public void onClick(View v) {
                AlertDialog.Builder builder = new AlertDialog.Builder(context);
                builder.setMessage(R.string.dialog_delete_note)
                        .setPositiveButton(R.string.delete, new DialogInterface.OnClickListener() {
                            public void onClick(DialogInterface dialog, int id) {
                                String query = AppState.getQuery();
                                Long offset = AppState.getOffset();
                                String cmd = "{\"action\": \"delete\", \"query\": \""
                                        + query
                                        + "\", \"rowid\":" + note.rowid.toString() + ", \"limit\":10, \"offset\":"
                                        + offset
                                        + "}";
                                Log.d("doSearchCmd", cmd);
                                String s = RustBridge.run(cmd);
                                ((MainActivity)context).doSearch(query, offset);
                            }
                        })
                        .setNegativeButton(R.string.cancel, new DialogInterface.OnClickListener() {
                            public void onClick(DialogInterface dialog, int id) {
                                // User cancelled the dialog
                            }
                        });
                // Create the AlertDialog object and return it
                AlertDialog alert = builder.create();
                alert.show();
            }
        });
        holder.mTagsContainer.addView(deleteButton);
        for (int i = 0; i < arr.length; i++)  {
            if (arr[i].length() > 0){
                Button btn = new Button(context);
                btn.setText(arr[i]);
                btn.setAllCaps(false);
                holder.mTagsContainer.addView(btn);
                btn.setOnClickListener((MainActivity)context);
            }
        }

        holder.mContentView.setText(note.created_at + " rowid: " + note.rowid + "\n"
                + note.title + "\n"
                + note.description + "\n"
                + note.url
        );

        holder.mView.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if (null != mListener) {
                    // Notify the active callbacks interface (the activity, if the
                    // fragment is attached to one) that an item has been selected.
                    mListener.onListFragmentInteraction(holder.mItem);
                }
            }
        });
    }

    @Override
    public int getItemCount() {
        return mValues.size();
    }


    public class ViewHolder extends RecyclerView.ViewHolder {
        public final View mView;
        public final TextView mContentView;
        public final LinearLayout mTagsContainer;
        public NoteItem mItem;

        public ViewHolder(View view) {
            super(view);
            mView = view;
            mContentView = (TextView) view.findViewById(R.id.content);
            mTagsContainer = (LinearLayout) view.findViewById(R.id.tagsContainer);
        }

        @Override
        public String toString() {
            return super.toString() + " '" + mContentView.getText() + "'";
        }
    }
}

