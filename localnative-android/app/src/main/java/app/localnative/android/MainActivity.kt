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

import android.annotation.SuppressLint
import android.app.AlertDialog
import android.content.Intent
import android.os.Bundle
import android.text.method.LinkMovementMethod
import android.util.Log
import android.view.Menu
import android.view.MenuItem
import android.view.View
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.appcompat.widget.SearchView
import androidx.appcompat.widget.Toolbar
import app.localnative.R
import com.google.zxing.integration.android.IntentIntegrator


class MainActivity : AppCompatActivity(), SearchView.OnQueryTextListener, NoteListFragment.OnListFragmentInteractionListener, View.OnClickListener {

    private lateinit var searchView: SearchView

//    private val mRecyclerView: RecyclerView? = null
//    private val mAdapter: RecyclerView.Adapter<*>? = null
//    private val mLayoutManager: RecyclerView.LayoutManager? = null

    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        // Get the SearchView and set the searchable configuration
        // SearchManager searchManager = (SearchManager) getSystemService(Context.SEARCH_SERVICE);
        menuInflater.inflate(R.menu.toolbar, menu)
        val searchItem = menu.findItem(R.id.toolbar_search)
        searchView = searchItem.actionView as SearchView

        //searchView.setSearchableInfo(searchManager.getSearchableInfo(getComponentName()));
        searchView.setIconifiedByDefault(false)
        searchView.setOnQueryTextListener(this)
        searchView.queryHint = getString(R.string.search_hint)
        searchView.requestFocusFromTouch()

        val width = resources.displayMetrics.widthPixels
        searchView.maxWidth = width * 2 / 3
        searchView.clearFocus()

        return super.onCreateOptionsMenu(menu)
    }

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            R.id.action_sync -> {
                val integrator = IntentIntegrator(this)
                integrator.setBeepEnabled(false)
                integrator.setCaptureActivity(QRScanActivity::class.java).initiateScan()
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }

    override fun onQueryTextSubmit(query: String): Boolean {
        Log.d("MainActivity", "onQueryTextSubmit: $query")
        return false
    }

    override fun onQueryTextChange(query: String): Boolean {
        Log.d("MainActivity", "onQueryTextChange: $query")
        AppState.clearOffset()
        doSearch(query, 0L)

        return false
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContentView(R.layout.activity_main)
        val toolbar = findViewById<View>(R.id.toolbar) as Toolbar
        setSupportActionBar(toolbar)

//        toolbar.setOnClickListener { Log.d("sync", "toolbar")
//            val integrator = IntentIntegrator(this)
//            integrator.setBeepEnabled(false)
//            integrator.setCaptureActivity(QRScanActivity::class.java).initiateScan()
//        }
//        for get files and cache directory:
//        val context = this.baseContext;
//
//        val files = context.filesDir // /data/data/包名/files/ 持久性文件根目录 路径在不同的手机上可能会不同
//        val cache = context.cacheDir  ///data/data/包名/cache/ 缓存性文件根目录
//        Log.d("test-f",files.toString())
//        Log.d("test-c",cache.toString())

        doSearch("", 0L)

        val prevButton = findViewById<View>(R.id.prev_button) as Button
        prevButton.setOnClickListener(this)
        val nextButton = findViewById<View>(R.id.next_button) as Button
        nextButton.setOnClickListener(this)
        findViewById<TextView>(R.id.bottomTextView).movementMethod = LinkMovementMethod.getInstance()

//        KeyboardVisibilityEvent.setEventListener(
//            this
//        ) { show ->
//            if (!show) {
//                searchView.setQuery("", false)
//                searchView.onActionViewCollapsed()
//            }
//        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        val result = IntentIntegrator.parseActivityResult(requestCode, resultCode, data)
        if (result != null) {
            if (result.contents == null) {
                Toast.makeText(this, "Cancelled Sync", Toast.LENGTH_LONG).show()
            } else {
                Toast.makeText(this, "Scanned server address and port: " + result.contents, Toast.LENGTH_LONG).show()
                val builder = AlertDialog.Builder(this, R.style.AlertDialogCustom)
                builder.setMessage(R.string.dialog_sync)
                        .setPositiveButton(R.string.sync) { _, _ ->
                            val cmd = ("{\"action\": \"client-sync\", \"addr\": \""
                                    + result.contents
                                    + "\""
                                    + "}")
                            Log.d("doClientSyncCmd", cmd)
                            val s = RustBridge.run(cmd)
                            Log.d("doClientSyncCmdResp", s)
                        }
                        .setNegativeButton(R.string.cancel) { _, _ ->
                            // User cancelled the dialog
                        }
                // Create the AlertDialog object and return it
                val alert = builder.create()
                alert.show()
            }
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    // button click events handler
    override fun onClick(v: View) {
        when (v.id) {
            R.id.prev_button -> {
                Log.d("click", "prev")
                val offset = AppState.decOffset()
                doSearch(AppState.getQuery(), offset)
                return
            }

            R.id.next_button -> {
                Log.d("click", "next")
                val offset = AppState.incOffset()
                doSearch(AppState.getQuery(), offset)
                return
            }
        }

        // tag
        val btn = v as Button
//        searchView.onActionViewExpanded()
        searchView.setQuery(btn.text, true)
        //        doSearch(btn.getText().toString(), 0L);

    }


    @SuppressLint("NotifyDataSetChanged")
    fun doSearch(query: String, offset: Long?) {
        AppState.setQuery(query)
        Log.d("doSearch", query + offset!!)

        val cmd = ("{\"action\": \"search\", \"query\": \""
                + query
                + "\", \"limit\":10, \"offset\":" +
                offset.toString() +
                "}")
        Log.d("doSearchCmd", cmd)
        val s = RustBridge.run(cmd)
        Log.d("doSearchResult", s)
        val noteListFragment = supportFragmentManager.findFragmentById(R.id.notes_recycler_view) as NoteListFragment?
        val count = NoteContent.refresh(s)
        AppState.setCount(count!!)
        val paginationText = AppState.makePaginationText()
        noteListFragment!!.mViewAdpater.notifyDataSetChanged()
        (findViewById<View>(R.id.pagination_text) as TextView).text = paginationText
    }


    override fun onListFragmentInteraction(item: NoteContent.NoteItem) {

    }
}
