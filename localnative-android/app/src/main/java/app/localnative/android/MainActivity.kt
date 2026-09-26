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

import android.app.AlertDialog
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.View
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import com.google.zxing.integration.android.IntentIntegrator
import app.localnative.R

class MainActivity : ComponentActivity(), View.OnClickListener {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        RustBridge.init(this)

        setContent {
            MaterialTheme {
                Surface {
                    MainScreen(
                        onQRScanClick = {
                            val integrator = IntentIntegrator(this)
                            integrator.setBeepEnabled(false)
                            integrator.setCaptureActivity(QRScanActivity::class.java)
                            integrator.initiateScan()
                        }
                    )
                }
            }
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        val result = IntentIntegrator.parseActivityResult(requestCode, resultCode, data)
        if (result != null) {
            if (result.contents == null) {
                Toast.makeText(this, "Cancelled Sync", Toast.LENGTH_LONG).show()
            } else {
                Toast.makeText(this, "Scanned server address and port: ${result.contents}", Toast.LENGTH_LONG).show()
                val builder = AlertDialog.Builder(this, R.style.AlertDialogCustom)
                builder.setMessage(R.string.dialog_sync)
                    .setPositiveButton(R.string.sync) { _, _ ->
                        // JSONObject escapes the scanned address; JNI runs off
                        // the main thread so a long sync can't ANR.
                        val cmd = JSONObject()
                            .put("action", "client-sync")
                            .put("addr", result.contents)
                            .toString()
                        lifecycleScope.launch(Dispatchers.IO) {
                            val response = RustBridge.run(cmd)
                            val message = try {
                                val json = JSONObject(response)
                                when {
                                    json.has("error") -> json.getString("error")
                                    json.has("client-sync") -> json.getString("client-sync")
                                    else -> response
                                }
                            } catch (_: Exception) { response }
                            withContext(Dispatchers.Main) {
                                Toast.makeText(this@MainActivity, message, Toast.LENGTH_LONG).show()
                            }
                        }
                    }
                    .setNegativeButton(R.string.cancel) { _, _ ->
                        // User cancelled the dialog
                    }
                val alert = builder.create()
                alert.show()
            }
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    // Legacy method for backward compatibility with old RecyclerView adapter
    @Deprecated("This method is for legacy code compatibility only")
    fun doSearch(query: String, offset: Long) {
        // This method is no longer used in the Compose-based UI
        Log.d("MainActivity", "Legacy doSearch called: query=$query, offset=$offset")
    }

    // Legacy OnClickListener implementation for backward compatibility
    @Deprecated("This method is for legacy code compatibility only")
    override fun onClick(v: View?) {
        // This method is no longer used in the Compose-based UI
        Log.d("MainActivity", "Legacy onClick called")
    }
}
