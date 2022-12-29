package com.example.dioxus_wallet

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import androidx.camera.core.*
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import com.google.mlkit.vision.barcode.BarcodeScanner
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.common.InputImage

class QrCodeScanner : AppCompatActivity() {
    val preview = Preview.Builder().build()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val view = PreviewView(this)
        preview.setSurfaceProvider(view.getSurfaceProvider())
        setContentView(view)

        startCameraOrRequestPermission()
    }

    private fun startCameraOrRequestPermission() {
        val permission = ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA)
        if (permission == PackageManager.PERMISSION_GRANTED) {
            startCamera()
        } else {
            ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.CAMERA), 0)
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        startCameraOrRequestPermission()
    }

    private var cameraProvider: ProcessCameraProvider? = null
    private var camera: Camera? = null

    private fun startCamera() {
        val executor = ContextCompat.getMainExecutor(this)
        val future = ProcessCameraProvider.getInstance(this)
        future.addListener({
            cameraProvider = future.get()
            cameraProvider!!.unbindAll()

            val scanner = BarcodeScanning.getClient(
                BarcodeScannerOptions.Builder()
                    .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
                    .build()
            )

            @ExperimentalGetImage
            val analyzer = ImageAnalysis.Analyzer { imageProxy ->
                val mediaImage = imageProxy.image ?: return@Analyzer
                val image = InputImage.fromMediaImage(mediaImage, imageProxy.imageInfo.rotationDegrees)
                scanner.process(image)
                    .addOnSuccessListener { barcodes ->
                        if (barcodes.size < 1) {
                            return@addOnSuccessListener
                        }
                        val barcode = barcodes[0].getRawValue();
                        if (barcode == null) {
                            return@addOnSuccessListener
                        }
                        val channel = getIntent().getLongExtra("channel", 0)
                        var intent = Intent()
                        intent.putExtra("channel", channel)
                        intent.putExtra("qrcode", barcode)
                        setResult(Activity.RESULT_OK, intent)
                        finish()
                    }
                    .addOnFailureListener {}
                    .addOnCompleteListener { imageProxy.close() }
            }

            val analysis = ImageAnalysis.Builder()
                .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                .build()
                .apply { setAnalyzer(executor, analyzer) }

            camera = cameraProvider!!.bindToLifecycle(
                this,
                CameraSelector.DEFAULT_BACK_CAMERA,
                preview,
                analysis
            )
        }, executor)
    }
}
