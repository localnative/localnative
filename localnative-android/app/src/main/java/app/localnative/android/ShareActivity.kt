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
import android.content.Intent
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import app.localnative.R
import com.android.volley.Request
import com.android.volley.toolbox.StringRequest
import com.android.volley.toolbox.Volley
import org.json.JSONObject
import android.text.method.ScrollingMovementMethod
import app.localnative.databinding.ActivityShareBinding

class ShareActivity : AppCompatActivity() {
    private lateinit var binding: ActivityShareBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_share)
        binding = ActivityShareBinding.inflate(layoutInflater)

        binding.tagsText.requestFocus()
        binding.textView.movementMethod = ScrollingMovementMethod()
        binding.btnCancel.setOnClickListener {
            finish()
        }
        binding.btnSave.setOnClickListener {

            val j = JSONObject()
            j.put("action", "insert")
            j.put("title", binding.titleText.text)
            j.put("url", binding.urlText.text)
            j.put("tags", binding.tagsText.text)
            j.put("description", binding.descText.text)
            j.put("comments", "")
            j.put("annotations", "")
            j.put("limit", 15)
            j.put("offset", 0)
            j.put("is_public", false)

            val cmd = j.toString()
            Log.d("CmdInsert", cmd)
            insert(cmd, 0)
        }
        when {
            intent?.action == Intent.ACTION_SEND -> {
                if ("text/plain" == intent.type) {
                    handleSendText(intent) // Handle text being sent
                } else if (intent.type?.startsWith("image/") == true) {
                    // handleSendImage(intent) // Handle single image being sent
                }
            }
            else -> {
                // Handle other intents, such as being started from the home screen
            }
        }
    }

    fun insert(cmd: String, offset: Long?) {
        val s = RustBridge.run(cmd)
        Log.d("CmdInsertResult", s)
        finish()
        val intent = Intent(this, MainActivity::class.java)
        startActivity(intent)
    }

    @SuppressLint("SetTextI18n")
    private fun handleSendText(intent: Intent) {
        intent.getStringExtra(Intent.EXTRA_TEXT)?.let {
            binding.urlText.setText(it)

            val queue = Volley.newRequestQueue(this)
            val url = it

            val stringRequest = StringRequest(Request.Method.GET, url,
                { response ->
                    val r =  response.trim()
                    //textView.text = r.substring(0, minOf(50000, r.length))
                    val re = Regex("""<(?i)(Title)>(.*?)<\\?/(?i)(title)>""")
                    re.find(r)?.let{
                        val (_, t, _)=it.destructured
                        val title =  t.trim()
                        binding.titleText.setText(title.substring(0, minOf(500,title.length)))
                        binding.textView.text = "title fetched."
                    }
                },
                { binding.textView.text = "can not fetch title :-( but you can still type your own title" })

            queue.add(stringRequest)
        }
    }
}
