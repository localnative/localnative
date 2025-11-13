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
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import com.google.zxing.integration.android.IntentIntegrator
import app.localnative.R

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

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
                        val cmd = """{"action": "client-sync", "addr": "${result.contents}"}"""
                        Log.d("doClientSyncCmd", cmd)
                        val response = RustBridge.run(cmd)
                        Log.d("doClientSyncCmdResp", response)
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
}
