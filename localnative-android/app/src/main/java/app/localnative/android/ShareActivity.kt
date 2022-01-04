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
import android.os.Bundle
import android.text.Editable
import android.text.method.ScrollingMovementMethod
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.databinding.DataBindingUtil
import app.localnative.R
import app.localnative.databinding.ActivityShareBinding
import com.android.volley.Request
import com.android.volley.toolbox.StringRequest
import com.android.volley.toolbox.Volley
import org.json.JSONObject


class ShareActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityShareBinding.inflate(layoutInflater)
        setContentView(binding.root)


        binding.tagsText.requestFocus()
        binding.textView.movementMethod = ScrollingMovementMethod()
        binding.btnCancel.setOnClickListener {
            Log.d("shareCancel","cancel...")
            finish()
        }
        Log.d("debugShare",binding.titleText.text.toString())
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
        when (intent?.action) {
            Intent.ACTION_SEND -> {
                if ("text/plain" == intent.type) {
                    handleSendText(intent,binding) // Handle text being sent
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
    private fun handleSendText(intent: Intent,binding: ActivityShareBinding) {
        binding.urlText.setText("hello")
        intent.getStringExtra(Intent.EXTRA_TEXT)?.let {
            Log.d("handleUrlShare",it)
            binding.urlText.setText(it)
            binding.textView.text = "title fetched."
            val queue = Volley.newRequestQueue(this)
            val url = it
            val stringRequest = StringRequest(Request.Method.GET, url,
                { response ->
                    val r =  response
                    Log.d("reponse",r)
                    val re = Regex("""<(?i)(title)>(.*?)<\\?/(?i)(title)>""")
                    re.find(r)?.let{
                        val (_, t, _)=it.destructured
                        val title = String(t.toByteArray())
                        Log.d("handleTitleShare",title)
                        binding.titleText.text = Editable.Factory.getInstance().newEditable(title)
                        binding.textView.text = "title fetched."
                    }
                },
                { binding.textView.text = "can not fetch title :-( but you can still type your own title" })

            queue.add(stringRequest)
        }
    }
}
