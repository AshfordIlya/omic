#include <jni.h>
#include <string>
#include <oboe/Oboe.h>
#include <android/log.h>
#include <iostream>
#include <arpa/inet.h> // htons, inet_addr
#include <netinet/in.h> // sockaddr_in
#include <sys/types.h> // uint16_t
#include <sys/socket.h> // socket, sendto
#include <unistd.h> // close

#define LOGE(tag, ...) __android_log_print(ANDROID_LOG_ERROR,    tag, __VA_ARGS__)
#define LOGW(tag, ...) __android_log_print(ANDROID_LOG_WARN,     tag, __VA_ARGS__)
#define LOGI(tag, ...) __android_log_print(ANDROID_LOG_INFO,     tag, __VA_ARGS__)
#define LOGD(tag, ...) __android_log_print(ANDROID_LOG_DEBUG,    tag, __VA_ARGS__)

class MyCallback : public oboe::AudioStreamDataCallback {
public:
    oboe::DataCallbackResult
    onAudioReady(oboe::AudioStream *audioStream, void *audioData, int32_t numFrames) override {

        // We requested AudioFormat::Float. So if the stream opens
        // we know we got the Float format.
        // If you do not specify a format then you should check what format
        // the stream has and cast to the appropriate type.
        auto *outputData = static_cast<float *>(audioData);
        // Generate random numbers (white noise) centered around zero.
        const float amplitude = 0.2f;
        for (int i = 0; i < numFrames; ++i) {
            outputData[i] = ((float) drand48() - 0.5f) * 2 * amplitude;
        }
        return oboe::DataCallbackResult::Continue;
    }
};

MyCallback myCallback;
std::shared_ptr<oboe::AudioStream> mStream;
extern "C"
JNIEXPORT jstring JNICALL
Java_com_omic_oboe_OboeFFI_stringFromJNISecond(JNIEnv *env, jobject thiz, void* buffer) {
    std::string hello = "Created a recorder";

    oboe::AudioStreamBuilder speaker;
    speaker.setDirection(oboe::Direction::Input);
    speaker.setPerformanceMode(oboe::PerformanceMode::LowLatency);
    speaker.setSharingMode(oboe::SharingMode::Exclusive);
    speaker.setFormat(oboe::AudioFormat::I16);
    speaker.setChannelCount(oboe::ChannelCount::Mono);
    oboe::Result result = speaker.openStream(mStream);
    if (result != oboe::Result::OK) {
        LOGE("Failed to create stream. Error:", "%s", oboe::convertToText(result));
    }
    mStream->requestStart();
    auto byte_buffer = malloc(3000);

    return env->NewStringUTF(hello.c_str());
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_omic_oboe_OboeFFI_killStream(JNIEnv *env, jobject thiz) {
    mStream->requestStop();
    return nullptr;
}
//
//int main(int argc, char const *argv[]) {
//    std::string hostname{"192.168.50.166"};
//    uint16_t port = 58809;
//
//    int sock = ::socket(AF_INET, SOCK_DGRAM, 0);
//
//    sockaddr_in destination;
//    destination.sin_family = AF_INET;
//    destination.sin_port = htons(port);
//    destination.sin_addr.s_addr = inet_addr(hostname.c_str());
//
//    std::string msg = "Jane Doe";
//    int n_bytes = ::sendto(sock, msg.c_str(), msg.length(), 0,
//                           reinterpret_cast<sockaddr *>(&destination), sizeof(destination));
//    std::cout << n_bytes << " bytes sent" << std::endl;
//    ::close(sock);
//
//    return 0;
//}


