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

import android.content.pm.PackageManager;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.support.v7.widget.RecyclerView;
import android.support.v7.widget.SearchView;
import android.support.v7.widget.Toolbar;
import android.util.Log;
import android.view.Menu;
import android.view.MenuItem;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;

import app.localnative.R;

import static app.localnative.android.Permission.*;

public class MainActivity extends AppCompatActivity implements SearchView.OnQueryTextListener,
        NoteListFragment.OnListFragmentInteractionListener,
        OnPermissonGrantedListenr, View.OnClickListener {

    @Override
    public boolean onCreateOptionsMenu(Menu menu) {
        // Get the SearchView and set the searchable configuration
        // SearchManager searchManager = (SearchManager) getSystemService(Context.SEARCH_SERVICE);
        getMenuInflater().inflate(R.menu.toolbar, menu);
        MenuItem searchItem = menu.findItem(R.id.toolbar_search);
        SearchView searchView =
                (SearchView) searchItem.getActionView();
        //searchView.setSearchableInfo(searchManager.getSearchableInfo(getComponentName()));
        searchView.setIconifiedByDefault(false);
        if (searchView != null) {
            searchView.setOnQueryTextListener(this);
        }
        searchView.setQueryHint(getString(R.string.search_hint));
        searchView.requestFocusFromTouch();
        return super.onCreateOptionsMenu(menu);
    }


    @Override
    public boolean onQueryTextSubmit(String query) {
        Log.d("onQueryTextSubmit",  query);
        return false;
    }

    @Override
    public boolean onQueryTextChange(String query) {
        Log.d("onQueryTextChange", query);
        doSearch(query);
        return false;
    }

    private RecyclerView mRecyclerView;
    private RecyclerView.Adapter mAdapter;
    private RecyclerView.LayoutManager mLayoutManager;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        Toolbar toolbar = (Toolbar) findViewById(R.id.toolbar);
        setSupportActionBar(toolbar);
        doSearch("");

        Button prevButton = (Button)findViewById(R.id.prev_button);
        prevButton.setOnClickListener(this);
        Button nextButton = (Button)findViewById(R.id.next_button);
        nextButton.setOnClickListener(this);
    }

    // button click events handler
    @Override
    public void onClick(View v) {
        switch (v.getId()) {
            case  R.id.prev_button: {
                Log.d("click", "prev");
                break;
            }

            case R.id.next_button: {
                Log.d("click", "next");
                break;
            }
        }
    }


    private void doSearch(String query) {
        Log.d("doSearch", query);
        // request allow write to storage permission
        invoke_WRITE_EXTERNAL_STORAGE(this, query);
    }

    @Override
    public void onPermissonGranted (String query) {
        String cmd = "{\"action\": \"search\", \"query\": \""
                + query
                +"\", \"limit\":10, \"offset\":0}";
        Log.d("doSearchCmd", cmd);
        String s = RustBridge.run(cmd);
        Log.d("doSearchResult", s);
        NoteListFragment noteListFragment = (NoteListFragment) getSupportFragmentManager().findFragmentById(R.id.notes_recycler_view);
        String paginationText = NoteContent.refresh(s);
        noteListFragment.mViewAdpater.notifyDataSetChanged();
        ((TextView)findViewById(R.id.pagination_text)).setText(paginationText);
    }

    @Override
    public void onListFragmentInteraction(NoteContent.NoteItem item) {

    }

    @Override
    public void onRequestPermissionsResult(int requestCode,
                                           String permissions[], int[] grantResults) {
        switch (requestCode) {
            case 1: {
                // If request is cancelled, the result arrays are empty.
                if (grantResults.length > 0
                        && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    // permission was granted, yay! Do the
                    // contacts-related task you need to do.
                } else {
                    // permission denied, boo! Disable the
                    // functionality that depends on this permission.
                }
                return;
            }

            // other 'case' lines to check for other
            // permissions this app might request.
        }
    }

}
