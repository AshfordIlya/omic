package com.omic.oboe

class OboeFFI {
    /**
     * A native method that is implemented by the 'oboe' native library,
     * which is packaged with this application.
     */
    external fun stringFromJNISecond(buffer: ByteArray): String

    external fun killStream(): Void
    companion object {
        // Used to load the 'oboe' library on application startup.
        init {
            System.loadLibrary("oboe-ffi")
        }
    }
}
