package app.localnative.android

import android.os.Bundle
import android.util.DisplayMetrics
import android.view.MenuItem
import android.view.View
import androidx.appcompat.app.AppCompatActivity
import androidx.appcompat.widget.Toolbar
import app.localnative.R
import app.localnative.android.NoteContent.NOTE_ITEM
import app.localnative.databinding.ActivityQrcodeBinding
import com.google.zxing.BarcodeFormat
import com.journeyapps.barcodescanner.BarcodeEncoder


class QRCodeActivity : AppCompatActivity() {

    private lateinit var binding: ActivityQrcodeBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityQrcodeBinding.inflate(layoutInflater)
        setContentView(binding.root)

        val toolbar = findViewById<View>(R.id.qr_code_toolbar) as Toolbar
        toolbar.title = getString(R.string.qrcode)
        setSupportActionBar(toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        supportActionBar?.setDisplayShowHomeEnabled(true)

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

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            android.R.id.home -> {
                onBackPressedDispatcher.onBackPressed()
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }

}
