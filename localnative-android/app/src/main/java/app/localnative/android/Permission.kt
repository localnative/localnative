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

import android.Manifest
import android.content.pm.PackageManager
import android.util.Log
import android.widget.Toast

import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat


object Permission {
    var WRITE_EXTERNAL_STORAGE = (
            "android.permission.\nWRITE_EXTERNAL_STORAGE\n"
                    + "must be allowed to access\n"
                    + "/sdcard/localnative.sqlite3 file.")

    fun invoke_WRITE_EXTERNAL_STORAGE(activity: AppCompatActivity,
                                      str: String,
                                      offset: Long?
    ) {
        if (ContextCompat.checkSelfPermission(activity,
                        Manifest.permission.WRITE_EXTERNAL_STORAGE) != PackageManager.PERMISSION_GRANTED) {

            // Permission is not granted
            // Should we show an explanation?
            if (ActivityCompat.shouldShowRequestPermissionRationale(activity,
                            Manifest.permission.WRITE_EXTERNAL_STORAGE)) {
                // Show an explanation to the user *asynchronously* -- don't block
                // this thread waiting for the user's response! After the user
                // sees the explanation, try again to request the permission.
                Toast.makeText(activity, WRITE_EXTERNAL_STORAGE,
                        Toast.LENGTH_LONG).show()
            } else {
                // No explanation needed; request the permission
                ActivityCompat.requestPermissions(activity,
                        arrayOf(Manifest.permission.WRITE_EXTERNAL_STORAGE),
                        1)

                Log.d("PERM no explain", "show permission")
                // 1 is an
                // app-defined int constant. The callback method gets the
                // result of the request.
            }
        } else {
            // Permission has already been granted
            (activity as OnPermissonGrantedListenr).onPermissonGranted(str, offset)
        }
    }

    interface OnPermissonGrantedListenr {
        fun onPermissonGranted(str: String, offset: Long?)
    }

}
