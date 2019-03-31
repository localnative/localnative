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
import android.content.Intent

import androidx.recyclerview.widget.RecyclerView

import android.app.AlertDialog
import android.content.Context
import android.graphics.Color
import android.util.Log
import android.util.TypedValue
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

import app.localnative.R
import app.localnative.android.NoteListFragment.OnListFragmentInteractionListener
import app.localnative.android.NoteContent.NoteItem

/**
 * [RecyclerView.Adapter] that can display a [NoteItem] and makes a call to the
 * specified [OnListFragmentInteractionListener].
 * TODO: Replace the implementation with code for your data type.
 */
class NoteRecyclerViewAdapter(private val mValues: List<NoteItem>, private val mListener: OnListFragmentInteractionListener?) : RecyclerView.Adapter<NoteRecyclerViewAdapter.ViewHolder>() {
    private var context: Context? = null

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
        context = parent.context
        val view = LayoutInflater.from(context)
                .inflate(R.layout.fragment_note, parent, false)
        return ViewHolder(view)
    }

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        holder.mItem = mValues[position]
        val note = mValues[position]
        holder.mTagsContainer.removeAllViews()
        val arr = note.tags.split(",".toRegex()).dropLastWhile { it.isEmpty() }.toTypedArray()

        // deleteButton
        val deleteButton = Button(context)
        deleteButton.text = "X"
        val height = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, 36f, context!!.resources.displayMetrics).toInt()
        val width = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, 45f, context!!.resources.displayMetrics).toInt()
        deleteButton.layoutParams = RecyclerView.LayoutParams(width, height)
        deleteButton.setTextColor(Color.RED)
        deleteButton.setBackgroundColor(Color.WHITE)
        deleteButton.setOnClickListener {
            val builder = AlertDialog.Builder(context)
            builder.setMessage(R.string.dialog_delete_note)
                    .setPositiveButton(R.string.delete) { dialog, id ->
                        val query = AppState.getQuery()
                        val offset = AppState.getOffset()
                        val cmd = ("{\"action\": \"delete\", \"query\": \""
                                + query
                                + "\", \"rowid\":" + note.rowid.toString() + ", \"limit\":10, \"offset\":"
                                + offset
                                + "}")
                        Log.d("doSearchCmd", cmd)
                        val s = RustBridge.run(cmd)
                        (context as MainActivity).doSearch(query, offset)
                    }
                    .setNegativeButton(R.string.cancel) { dialog, id ->
                        // User cancelled the dialog
                    }
            // Create the AlertDialog object and return it
            val alert = builder.create()
            alert.show()
        }
        holder.mTagsContainer.addView(deleteButton)

        // qrCodeButton
        val qrCodeButton = Button(context)
        qrCodeButton.text = "QR"
        qrCodeButton.layoutParams = RecyclerView.LayoutParams(width, height)
        qrCodeButton.setTextColor(Color.WHITE)
        qrCodeButton.setBackgroundColor(Color.BLACK)
        qrCodeButton.setOnClickListener {
            val intent = Intent(context, QRCodeActivity::class.java)
            (context as AppCompatActivity).startActivity(intent)
        }
        holder.mTagsContainer.addView(qrCodeButton)

        // tags
        for (i in arr.indices) {
            if (arr[i].length > 0) {
                val btn = Button(context)
                btn.text = arr[i]
                btn.isAllCaps = false
                btn.layoutParams = RecyclerView.LayoutParams((arr[i].length.toDouble() * width.toDouble() * 0.2 + width * 0.5).toInt(), height)
                holder.mTagsContainer.addView(btn)
                btn.setOnClickListener(context as MainActivity?)
            }
        }

        holder.mContentView.text = (note.created_at + " rowid: " + note.rowid + "\n"
                + note.title + "\n"
                + note.description + "\n"
                + note.url)

        holder.mView.setOnClickListener {
            mListener?.onListFragmentInteraction(holder.mItem)
        }
    }

    override fun getItemCount(): Int {
        return mValues.size
    }


    inner class ViewHolder(val mView: View) : RecyclerView.ViewHolder(mView) {
        val mContentView: TextView
        val mTagsContainer: LinearLayout
        var mItem: NoteItem? = null

        init {
            mContentView = mView.findViewById<View>(R.id.content) as TextView
            mTagsContainer = mView.findViewById<View>(R.id.tagsContainer) as LinearLayout
        }

        override fun toString(): String {
            return super.toString() + " '" + mContentView.text + "'"
        }
    }
}

