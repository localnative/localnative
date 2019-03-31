package app.localnative.android

import android.os.Bundle
import android.view.View
import android.widget.Button
import androidx.appcompat.app.AppCompatActivity
import app.localnative.R
import com.google.zxing.BarcodeFormat
import com.journeyapps.barcodescanner.BarcodeEncoder
import android.widget.ImageView


class QRCodeActivity : AppCompatActivity(), View.OnClickListener {

    override fun onClick(v: View) {
        finish()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_qrcode)

        val backButton = findViewById<View>(R.id.qr_back_button) as Button
        backButton.setOnClickListener(this)

        try {
            val imageViewQrCode = findViewById<View>(R.id.qrCodeImageView) as ImageView
            val width = imageViewQrCode.width
            val height = imageViewQrCode.height
            val barcodeEncoder = BarcodeEncoder()
            val bitmap = barcodeEncoder.encodeBitmap(AppState.getCurrentUrl(), BarcodeFormat.QR_CODE, 1000, 1000)
            imageViewQrCode.setImageBitmap(bitmap)
        } catch (e: Exception) {

        }

    }

}
