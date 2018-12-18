package app.localnative.android

import android.content.Intent
import android.support.v7.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import app.localnative.R
import com.android.volley.Request
import com.android.volley.Response
import com.android.volley.toolbox.StringRequest
import com.android.volley.toolbox.Volley
import kotlinx.android.synthetic.main.activity_share.*
import org.json.JSONObject
import android.text.method.ScrollingMovementMethod
import java.net.URLDecoder

class ShareActivity : AppCompatActivity(), Permission.OnPermissonGrantedListenr {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_share)
        tagsText.requestFocus()
        textView.movementMethod = ScrollingMovementMethod()
        btnCancel.setOnClickListener {
            finish()
        }
        btnSave.setOnClickListener {

            val j = JSONObject()
            j.put("action", "insert")
            j.put("title", titleText.text)
            j.put("url", urlText.text)
            j.put("tags", tagsText.text)
            j.put("description", descText.text)
            j.put("comments", "")
            j.put("annotations", "")
            j.put("limit", 15)
            j.put("offset", 0)
            j.put("is_public", false)

            val cmd = j.toString()
            Log.d("CmdInsert", cmd)
            Permission.invoke_WRITE_EXTERNAL_STORAGE(this, cmd)
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

    override fun onPermissonGranted(cmd: String?) {
        val s = RustBridge.run(cmd)
        Log.d("CmdInsertResult", s)
        finish()
        val intent = Intent(this, MainActivity::class.java)
        startActivity(intent)
    }

    private fun handleSendText(intent: Intent) {
        intent.getStringExtra(Intent.EXTRA_TEXT)?.let {
            urlText.setText(it)

            val queue = Volley.newRequestQueue(this)
            val url = it

            val stringRequest = StringRequest(Request.Method.GET, url,
                    Response.Listener<String> { response ->
                        val r =  response.trim()
                        textView.text = r.substring(0, minOf(50000, r.length))
                        val re = Regex("""<(?i)(Title)>(.*?)<\\?/(?i)(title)>""")
                        re.find(r)?.let{
                            val (_, t, _)=it.destructured
                            val title =  t.trim()
                            titleText.setText(title.substring(0, minOf(500,title.length)))
                        }
                    },
                    Response.ErrorListener { textView.text = "url response error!" })

            queue.add(stringRequest)
        }
    }
}
