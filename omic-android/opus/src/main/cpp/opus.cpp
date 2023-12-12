#include <jni.h>
#include <string>
#include <opus.h>
#include <android/log.h>


#define LOGE(tag, ...) __android_log_print(ANDROID_LOG_ERROR,    tag, __VA_ARGS__)
#define LOGW(tag, ...) __android_log_print(ANDROID_LOG_WARN,     tag, __VA_ARGS__)
#define LOGI(tag, ...) __android_log_print(ANDROID_LOG_INFO,     tag, __VA_ARGS__)
#define LOGD(tag, ...) __android_log_print(ANDROID_LOG_DEBUG,    tag, __VA_ARGS__)

extern "C" JNIEXPORT jstring JNICALL
Java_com_omic_opus_NativeLib_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    std::string hello = "Hello from C++";
    return env->NewStringUTF(hello.c_str());
}

#define TAG "C++"

OpusEncoder *encoder = nullptr;

extern "C"
JNIEXPORT jint JNICALL Java_com_omic_opus_NativeLib_encoderInit(JNIEnv *env, jobject thiz, jint sample_rate, jint num_channels, jint application) {
    if (num_channels != 1 && num_channels != 2) LOGE(TAG, "[encoderInit] num_channels is incorrect: %d - it must be either 1 or 2, otherwise the encoder may works incorrectly", num_channels);

    int size = opus_encoder_get_size(num_channels);

    if (size <= 0) {
        LOGE(TAG, "[encoderInit] couldn't init encoder with size: %d", size);
        return size;
    }

    encoder = (OpusEncoder*) malloc((size_t) size);

    int ret = opus_encoder_init(encoder, sample_rate, num_channels, application);

    if (ret) {
        LOGE(TAG, "[encoderInit] couldn't init encoder ret: %d; error: %s", ret, opus_strerror(ret));
        free(encoder);
        return -1;
    } else LOGD(TAG, "[encoderInit] encoder successfully initialized");

    return 0;
}
