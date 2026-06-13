package com.lanrhyme.micyou.audio
import com.lanrhyme.micyou.R

import android.media.MediaRecorder
import androidx.annotation.StringRes
import com.lanrhyme.micyou.audio.AndroidAudioSource
import com.lanrhyme.micyou.audio.AudioSourceOption
import com.lanrhyme.micyou.audio.getAudioSourceOptions

enum class AndroidAudioSource(@StringRes val labelRes: Int, val sourceId: Int) {
    Mic(R.string.audioSourceMic, MediaRecorder.AudioSource.MIC),
    VoiceCommunication(R.string.audioSourceVoiceCommunication, MediaRecorder.AudioSource.VOICE_COMMUNICATION),
    VoiceRecognition(R.string.audioSourceVoiceRecognition, MediaRecorder.AudioSource.VOICE_RECOGNITION),
    VoicePerformance(R.string.audioSourceVoicePerformance, MediaRecorder.AudioSource.VOICE_PERFORMANCE),
    Camcorder(R.string.audioSourceCamcorder, MediaRecorder.AudioSource.CAMCORDER),
    Unprocessed(R.string.audioSourceUnprocessed, MediaRecorder.AudioSource.UNPROCESSED)
}

data class AudioSourceOption(
    val name: String,
    @StringRes val labelRes: Int? = null,
    val label: String? = null
)

fun getAudioSourceOptions(): List<AudioSourceOption> {
    return AndroidAudioSource.entries.map { AudioSourceOption(it.name, it.labelRes) }
}