package app.localnative.flutter

import io.flutter.embedding.android.FlutterActivity

class MainActivity: FlutterActivity() {
    init {
        // Load native library
        System.loadLibrary("localnative_flutter")
    }
}
