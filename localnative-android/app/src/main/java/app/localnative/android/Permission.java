package app.localnative.android;

import android.Manifest;
import android.app.Activity;
import android.content.pm.PackageManager;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;
import android.util.Log;
import android.widget.Toast;


public class Permission {
    public static String WRITE_EXTERNAL_STORAGE =
            "android.permission.\nWRITE_EXTERNAL_STORAGE\n"
            + "must be allowed to access\n"
            + "/sdcard/localnative.sqlite3 file.";

    public static void invoke_WRITE_EXTERNAL_STORAGE(Activity activity,
                              String str
                              ) {
        if (ContextCompat.checkSelfPermission(activity,
                Manifest.permission.WRITE_EXTERNAL_STORAGE)
                != PackageManager.PERMISSION_GRANTED) {

            // Permission is not granted
            // Should we show an explanation?
            if (ActivityCompat.shouldShowRequestPermissionRationale(activity,
                    Manifest.permission.WRITE_EXTERNAL_STORAGE)) {
                // Show an explanation to the user *asynchronously* -- don't block
                // this thread waiting for the user's response! After the user
                // sees the explanation, try again to request the permission.
                Toast.makeText(activity, WRITE_EXTERNAL_STORAGE,
                        Toast.LENGTH_LONG).show();
            } else {
                // No explanation needed; request the permission
                ActivityCompat.requestPermissions(activity,
                        new String[]{Manifest.permission.WRITE_EXTERNAL_STORAGE},
                        1);

                Log.d("PERM no explain", "show permission");
                // 1 is an
                // app-defined int constant. The callback method gets the
                // result of the request.
            }
        } else {
            // Permission has already been granted
            ((OnPermissonGrantedListenr)activity).onPermissonGranted(str);
        }
    }

    public interface OnPermissonGrantedListenr{
        void onPermissonGranted(String str);
    }

}