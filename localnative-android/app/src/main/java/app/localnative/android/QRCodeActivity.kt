package app.localnative.android

import android.os.Bundle
import android.util.DisplayMetrics
import android.view.View
import androidx.appcompat.app.AppCompatActivity
import app.localnative.android.NoteContent.NOTE_ITEM
import app.localnative.databinding.ActivityQrcodeBinding
import com.google.zxing.BarcodeFormat
import com.journeyapps.barcodescanner.BarcodeEncoder


class QRCodeActivity : AppCompatActivity(), View.OnClickListener {

    private lateinit var binding: ActivityQrcodeBinding

    override fun onClick(v: View) {
        finish()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityQrcodeBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.qrBackButton.setOnClickListener(this)

        try {
            val note = intent.getSerializableExtra(NOTE_ITEM) as NoteContent.NoteItem
            binding.datetimeTextView.text = note.created_at.substring(0,19)
            binding.titleTextView.text = note.title
            binding.linkTextView.text = note.url
            val outMetrics = DisplayMetrics()
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
                display?.getRealMetrics(outMetrics)
            } else {
                @Suppress("DEPRECATION")
                windowManager.defaultDisplay.getMetrics(outMetrics)
            }
            val width = outMetrics.widthPixels
            val barcodeEncoder = BarcodeEncoder()
            val bitmap = barcodeEncoder.encodeBitmap(AppState.getCurrentUrl(), BarcodeFormat.QR_CODE, width, width)
            binding.qrCodeImageView.setImageBitmap(bitmap)
        } catch (e: Exception) {
            e.printStackTrace()
        }

    }

}
