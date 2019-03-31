package app.localnative.android

import android.os.Bundle
import android.view.View
import android.widget.Button
import androidx.appcompat.app.AppCompatActivity
import app.localnative.R

class QRCodeActivity : AppCompatActivity(), View.OnClickListener {

    override fun onClick(v: View) {
        finish()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_qrcode)

        val backButton = findViewById<View>(R.id.qr_back_button) as Button
        backButton.setOnClickListener(this)

    }

}
