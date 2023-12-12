package com.omic.opus

const val OPUS_APPLICATION_RESTRICTED_LOWDELAY = 2051
class NativeLib {

    /**
     * A native method that is implemented by the 'opus' native library,
     * which is packaged with this application.
     */
    fun encoderInit(sampleRate: Int, channels: Int): Int {
        return encoderInit(sampleRate, channels, OPUS_APPLICATION_RESTRICTED_LOWDELAY)
    }

    private external fun encoderInit(sampleRate: Int, numChannels: Int, application: Int): Int

    companion object {
        // Used to load the 'opus' library on application startup.
        init {
            System.loadLibrary("opus-ffi")
        }
    }
}
