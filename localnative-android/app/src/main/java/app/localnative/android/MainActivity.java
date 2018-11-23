package app.localnative.android;

import android.support.v7.app.AppCompatActivity;
import android.os.Bundle;
import android.widget.TextView;

public class MainActivity extends AppCompatActivity {
    static {
        System.loadLibrary("localnative_core");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        RustBridge r = new RustBridge();
        //TODO detect allow write to storage permission
        String s = r.run("{\"action\": \"select\", \"limit\":10,\"offset\":0}");
        ((TextView)findViewById(R.id.searchText)).setText(s);
    }

}
