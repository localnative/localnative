package app.localnative.android;

import android.app.SearchManager;
import android.content.Context;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.support.v7.widget.RecyclerView;
import android.support.v7.widget.SearchView;
import android.support.v7.widget.Toolbar;
import android.util.Log;
import android.view.Menu;
import android.view.MenuItem;
import android.view.Window;
import android.view.WindowManager;
import android.widget.TextView;

import app.ln.R;

public class MainActivity extends AppCompatActivity implements SearchView.OnQueryTextListener,
        NoteListFragment.OnListFragmentInteractionListener {
    RustBridge r = new RustBridge();
    static {
        System.loadLibrary("localnative_core");
    }

    @Override
    public boolean onCreateOptionsMenu(Menu menu) {
        // Get the SearchView and set the searchable configuration
        SearchManager searchManager = (SearchManager) getSystemService(Context.SEARCH_SERVICE);
        getMenuInflater().inflate(R.menu.toolbar, menu);
        MenuItem searchItem = menu.findItem(R.id.toolbar_search);
        SearchView searchView =
                (SearchView) searchItem.getActionView();
        searchView.setSearchableInfo(searchManager.getSearchableInfo(getComponentName()));
        searchView.setIconifiedByDefault(false);
        if (searchView != null) {
            searchView.setOnQueryTextListener(this);
        }
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

        requestWindowFeature(Window.FEATURE_NO_TITLE);
        getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                WindowManager.LayoutParams.FLAG_FULLSCREEN);

        setContentView(R.layout.activity_main);

        Toolbar toolbar = (Toolbar) findViewById(R.id.toolbar);
        setSupportActionBar(toolbar);
        getSupportActionBar().setDisplayShowTitleEnabled(true);

        doSearch("");

    }


    private void doSearch(String query) {
        Log.d("doSearch", query);
        //TODO detect allow write to storage permission
        String cmd = "{\"action\": \"search\", \"query\": \""
                + query
                +"\", \"limit\":10, \"offset\":0}";
        Log.d("doSearchCmd", cmd);
        String s = r.run(cmd);
        Log.d("doSearchResult", s);
        ((TextView)findViewById(R.id.searchText)).setText(s);

    }

    @Override
    public void onListFragmentInteraction(NoteContent.NoteItem item) {

    }
}
