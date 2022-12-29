package com.example.dioxus_wallet

import android.app.Activity
import android.content.Intent
import androidx.activity.result.contract.ActivityResultContracts.StartActivityForResult

class MainActivity : TauriActivity() {
    private external fun onQrCodeScanned(channel: Long, qrcode: String)

    fun scanQrCode(channel: Long) {
        val intent = Intent(this, QrCodeScanner::class.java)
        intent.putExtra("channel", channel)
        resultLauncher.launch(intent)
    }

    val resultLauncher = registerForActivityResult(StartActivityForResult()) { result ->
        if (result.resultCode == Activity.RESULT_OK) {
            val intent = result.data ?: return@registerForActivityResult
            val channel = intent.getLongExtra("channel", 0)
            val qrcode = intent.getStringExtra("qrcode")
            if (qrcode != null) {
                onQrCodeScanned(channel, qrcode)
            }
        }
    }
}
