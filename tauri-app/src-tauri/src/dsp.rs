use nnnoiseless::DenoiseState;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;

/// Audio DSP settings, synced from the frontend.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDspSettings {
    pub gain: f32,           // dB, -50 to +50
    pub ns_enabled: bool,
    pub ns_type: String,     // "RNNoise", "Ulunas", "Speexdsp", "None"
    pub ns_intensity: f32,   // 0..100
    pub dereverb_enabled: bool,
    pub dereverb_level: f32, // 0..100
    pub agc_enabled: bool,
    pub agc_target: f32,     // 0..32767
    pub agc_attack: f32,     // raw slider value 1..100, maps to 0.001..0.1
    pub agc_decay: f32,      // raw slider value 1..100, maps to 0.0001..0.01
    pub vad_enabled: bool,
    pub vad_threshold: f32,  // dB, -100..0
}

impl Default for AudioDspSettings {
    fn default() -> Self {
        Self {
            gain: 0.0,
            ns_enabled: false,
            ns_type: "RNNoise".to_string(),
            ns_intensity: 50.0,
            dereverb_enabled: false,
            dereverb_level: 50.0,
            agc_enabled: false,
            agc_target: 16000.0,
            agc_attack: 50.0,
            agc_decay: 50.0,
            vad_enabled: false,
            vad_threshold: -40.0,
        }
    }
}

// ─── Ulunas ONNX Processor ─────────────────────────────────────────────────

/// Port of UlunasProcessor.kt - ONNX Runtime AI denoiser
struct UlunasProcessor {
    session: ort::session::Session,
    frame_size: usize,   // 960
    hop_length: usize,   // 480
    window: Vec<f32>,
    ola_gain: f32,
    previous: Vec<f32>,
    ola_accumulator: Vec<f32>,
    state_data: Vec<Vec<f32>>,
    state_shapes: Vec<Vec<usize>>,
}

impl UlunasProcessor {
    fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let frame_size = 960;
        let hop_length = 480;

        let session = ort::session::Session::builder()?
            .with_intra_threads(1)?
            .with_inter_threads(1)?
            .commit_from_file(model_path)?;

        let window = Self::hanning_window(frame_size);
        let ola_gain = Self::calc_ola_gain(&window, hop_length);

        // State shapes
        let state_shapes: Vec<Vec<usize>> = vec![
            vec![1, 1, 2, 121], vec![1, 24, 1, 61], vec![1, 24, 1, 31],
            vec![1, 1, 24], vec![1, 1, 48], vec![1, 1, 48],
            vec![1, 1, 64], vec![1, 1, 32], vec![1, 31, 16],
            vec![1, 31, 16], vec![1, 24, 1, 31], vec![1, 12, 1, 31],
            vec![1, 12, 2, 61], vec![1, 1, 64], vec![1, 1, 48],
            vec![1, 1, 48], vec![1, 1, 24], vec![1, 1, 2],
        ];

        let state_data: Vec<Vec<f32>> = state_shapes
            .iter()
            .map(|shape| vec![0.0f32; shape.iter().product()])
            .collect();

        Ok(Self {
            session,
            frame_size,
            hop_length,
            window,
            ola_gain,
            previous: vec![0.0; hop_length],
            ola_accumulator: vec![0.0; frame_size],
            state_data,
            state_shapes,
        })
    }

    fn hanning_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| {
                let v = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (size - 1) as f64).cos();
                v.sqrt() as f32
            })
            .collect()
    }

    fn calc_ola_gain(window: &[f32], hop_length: usize) -> f32 {
        let mut sum_sq = 0.0_f32;
        for i in 0..hop_length {
            let w1 = window[i];
            let w2 = window[i + hop_length];
            sum_sq += w1 * w1 + w2 * w2;
        }
        let avg = sum_sq / hop_length as f32;
        if avg > 0.001 { 1.0 / avg.sqrt() } else { 1.0 }
    }

    fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if input.len() != self.hop_length {
            return input.to_vec();
        }

        let frame_size = self.frame_size;
        let hop_length = self.hop_length;
        let spec_size = frame_size / 2 + 1;

        // Build frame: previous + current
        let mut fft_buffer = vec![0.0_f32; frame_size];
        fft_buffer[..hop_length].copy_from_slice(&self.previous);
        fft_buffer[hop_length..].copy_from_slice(input);
        self.previous.copy_from_slice(input);

        // Apply window
        for i in 0..frame_size {
            fft_buffer[i] *= self.window[i];
        }

        // FFT using rustfft
        use rustfft::FftPlanner;
        use rustfft::num_complex::Complex;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(frame_size);

        let mut complex_buf: Vec<Complex<f32>> = fft_buffer
            .iter()
            .map(|&v| Complex::new(v, 0.0))
            .collect();
        fft.process(&mut complex_buf);

        // Convert to model input format: flat vec for [1, spec_size, 1, 2]
        let mut spec_flat = vec![0.0f32; spec_size * 2];
        for i in 0..spec_size {
            spec_flat[i * 2] = complex_buf[i].re;
            spec_flat[i * 2 + 1] = complex_buf[i].im;
        }

        // Convert to ort Values using (shape, data) tuple
        let spec_shape = vec![1, spec_size, 1, 2];
        let val_spec = ort::value::Value::from_array((spec_shape, spec_flat)).unwrap();
        
        let val_states: Vec<_> = self.state_data.iter().zip(self.state_shapes.iter())
            .map(|(data, shape)| ort::value::Value::from_array((shape.clone(), data.clone())).unwrap())
            .collect();

        // Run inference
        let outputs = match self.session.run(ort::inputs![
            &val_spec,
            &val_states[0], &val_states[1], &val_states[2], &val_states[3],
            &val_states[4], &val_states[5], &val_states[6], &val_states[7],
            &val_states[8], &val_states[9], &val_states[10], &val_states[11],
            &val_states[12], &val_states[13], &val_states[14], &val_states[15],
            &val_states[16], &val_states[17]
        ]) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("ONNX inference failed: {}", e);
                return input.to_vec();
            }
        };

        // Extract output spectrum
        if let Ok(output_tensor) = outputs[0].try_extract_tensor::<f32>() {
            let output_data = output_tensor.1; // (Shape, slice)
            for i in 0..spec_size {
                complex_buf[i] = Complex::new(output_data[i * 2], output_data[i * 2 + 1]);
            }
            for i in spec_size..frame_size {
                complex_buf[i] = complex_buf[frame_size - i].conj();
            }
        }

        // Update states from outputs 1..18
        for i in 1..outputs.len().min(19) {
            if let Ok(state_tensor) = outputs[i].try_extract_tensor::<f32>() {
                let state_data = state_tensor.1;
                if i - 1 < self.state_data.len() && state_data.len() == self.state_data[i - 1].len() {
                    self.state_data[i - 1].copy_from_slice(state_data);
                }
            }
        }

        // IFFT
        let ifft = planner.plan_fft_inverse(frame_size);
        ifft.process(&mut complex_buf);
        let scale = 1.0 / frame_size as f32;
        for i in 0..frame_size {
            fft_buffer[i] = complex_buf[i].re * scale * self.window[i];
        }

        // OLA
        for i in 0..frame_size {
            self.ola_accumulator[i] += fft_buffer[i];
        }

        let mut output = vec![0.0_f32; hop_length];
        for i in 0..hop_length {
            output[i] = self.ola_accumulator[i] * self.ola_gain;
        }

        // Shift accumulator
        for i in 0..frame_size - hop_length {
            self.ola_accumulator[i] = self.ola_accumulator[i + hop_length];
        }
        for i in frame_size - hop_length..frame_size {
            self.ola_accumulator[i] = 0.0;
        }

        output
    }
}

// ─── Speexdsp-style spectral subtraction noise suppression ──────────────────

/// A simple spectral subtraction noise suppressor inspired by Speex's approach.
/// Uses FFT to estimate noise floor and subtract it from the signal.
struct SpeexStyleNS {
    frame_size: usize,
    noise_estimate: Vec<f32>, // Running noise floor estimate per frequency bin
    adaptation_rate: f32,
}

impl SpeexStyleNS {
    fn new() -> Self {
        let frame_size = 480;
        Self {
            frame_size,
            noise_estimate: vec![0.0; frame_size / 2 + 1],
            adaptation_rate: 0.02,
        }
    }

    fn process(&mut self, data: &mut [f32], intensity: f32) {
        use rustfft::FftPlanner;
        use rustfft::num_complex::Complex;

        let len = data.len();
        if len < self.frame_size {
            return;
        }

        let mut planner = FftPlanner::new();
        let fft_forward = planner.plan_fft_forward(self.frame_size);
        let fft_inverse = planner.plan_fft_inverse(self.frame_size);

        let num_frames = len / self.frame_size;
        let mix = (intensity / 100.0).clamp(0.0, 1.0);

        for frame_idx in 0..num_frames {
            let offset = frame_idx * self.frame_size;
            let frame = &data[offset..offset + self.frame_size];

            let mut complex: Vec<Complex<f32>> = frame
                .iter()
                .map(|&s| Complex::new(s, 0.0))
                .collect();

            fft_forward.process(&mut complex);

            let spec_size = self.frame_size / 2 + 1;

            // Compute magnitude and update noise estimate
            for i in 0..spec_size {
                let mag = complex[i].norm();
                // Slow adaptation: update noise floor estimate
                self.noise_estimate[i] = self.noise_estimate[i] * (1.0 - self.adaptation_rate)
                    + mag * self.adaptation_rate;
            }

            // Spectral subtraction
            for i in 0..spec_size {
                let mag = complex[i].norm();
                let phase = complex[i].arg();
                let noise = self.noise_estimate[i] * mix * 2.0;
                let clean_mag = (mag - noise).max(mag * 0.05); // Spectral floor
                complex[i] = Complex::from_polar(clean_mag, phase);
                // Mirror
                if i > 0 && i < self.frame_size - i {
                    complex[self.frame_size - i] = complex[i].conj();
                }
            }

            fft_inverse.process(&mut complex);

            let scale = 1.0 / self.frame_size as f32;
            for i in 0..self.frame_size {
                data[offset + i] = complex[i].re * scale;
            }
        }
    }
}

// ─── Main DSP Processor ─────────────────────────────────────────────────────

/// The main DSP processor. Operates on f32 PCM samples at 48kHz.
pub struct DspProcessor {
    settings: Arc<RwLock<AudioDspSettings>>,
    // RNNoise state (expects 480-sample frames at 48kHz)
    denoiser: Box<DenoiseState<'static>>,
    ns_buffer: Vec<f32>,
    // Ulunas ONNX processor
    ulunas: Option<UlunasProcessor>,
    ulunas_buffer: Vec<f32>,
    ulunas_model_path: Option<PathBuf>,
    // Speexdsp-style NS
    speex_ns: SpeexStyleNS,
    // AGC envelope follower
    agc_envelope: f32,
    // VAD fade state (0.0 = muted, 1.0 = full)
    vad_fade: f32,
    // Spectrum snapshots
    raw_spectrum: Vec<f32>,
    processed_spectrum: Vec<f32>,
}

const RNNOISE_FRAME_SIZE: usize = 480;

impl DspProcessor {
    pub fn new(settings: Arc<RwLock<AudioDspSettings>>, model_dir: Option<PathBuf>) -> Self {
        let ulunas_model_path = model_dir.map(|d| d.join("ulunas.onnx"));
        Self {
            settings,
            denoiser: DenoiseState::new(),
            ns_buffer: Vec::with_capacity(RNNOISE_FRAME_SIZE * 2),
            ulunas: None,
            ulunas_buffer: Vec::with_capacity(RNNOISE_FRAME_SIZE * 2),
            ulunas_model_path,
            speex_ns: SpeexStyleNS::new(),
            agc_envelope: 0.0,
            vad_fade: 1.0,
            raw_spectrum: vec![0.0; 64],
            processed_spectrum: vec![0.0; 64],
        }
    }

    /// Process a chunk of f32 PCM audio in-place.
    /// Returns (raw_rms, processed_rms) for level metering.
    pub fn process(&mut self, data: &mut Vec<f32>) -> (f32, f32) {
        if data.is_empty() {
            return (0.0, 0.0);
        }

        let raw_rms = compute_rms(data);
        self.compute_spectrum(data, true);

        let settings = self.settings.read().unwrap().clone();

        // 1. Gain
        if settings.gain.abs() > 0.01 {
            let gain_linear = 10.0_f32.powf(settings.gain / 20.0);
            for sample in data.iter_mut() {
                *sample *= gain_linear;
            }
        }

        // 2. Noise Suppression
        if settings.ns_enabled {
            match settings.ns_type.as_str() {
                "RNNoise" => self.apply_rnnoise(data, settings.ns_intensity),
                "Ulunas" => self.apply_ulunas(data, settings.ns_intensity),
                "Speexdsp" => self.apply_speex(data, settings.ns_intensity),
                "Lightweight" => self.apply_lightweight(data, settings.ns_intensity),
                _ => {} // "None"
            }
        }

        // 3. Dereverb
        if settings.dereverb_enabled {
            self.apply_dereverb(data, settings.dereverb_level);
        }

        // 4. AGC
        if settings.agc_enabled {
            let attack_rate = settings.agc_attack / 1000.0;
            let decay_rate = settings.agc_decay / 10000.0;
            self.apply_agc(data, settings.agc_target, attack_rate, decay_rate);
        }

        // 5. VAD
        if settings.vad_enabled {
            self.apply_vad(data, settings.vad_threshold);
        }

        // Clamp
        for sample in data.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }

        let processed_rms = compute_rms(data);
        self.compute_spectrum(data, false);

        (raw_rms, processed_rms)
    }

    pub fn get_spectrums(&self) -> (Vec<f32>, Vec<f32>) {
        (self.raw_spectrum.clone(), self.processed_spectrum.clone())
    }

    // ── Lightweight (Noise Gate) ─────────────────────────────────────────────

    fn apply_lightweight(&mut self, data: &mut Vec<f32>, intensity: f32) {
        // A simple soft noise gate (expander) based on intensity
        // intensity 0..100 maps to a threshold of -60dB to -30dB
        let threshold_db = -60.0 + (intensity / 100.0) * 30.0;
        let threshold = 10.0_f32.powf(threshold_db / 20.0);
        
        let ratio = 2.0; // Expansion ratio
        
        for sample in data.iter_mut() {
            let abs_val = sample.abs();
            if abs_val < threshold && abs_val > 0.0 {
                // Apply soft downward expansion
                let gain = (abs_val / threshold).powf(ratio - 1.0);
                *sample *= gain;
            }
        }
    }

    // ── RNNoise (nnnoiseless) ───────────────────────────────────────────────

    fn apply_rnnoise(&mut self, data: &mut Vec<f32>, intensity: f32) {
        let mix = (intensity / 100.0).clamp(0.0, 1.0);
        self.ns_buffer.extend_from_slice(data);

        let mut output = Vec::with_capacity(data.len());

        while self.ns_buffer.len() >= RNNOISE_FRAME_SIZE {
            let frame: Vec<f32> = self.ns_buffer.drain(..RNNOISE_FRAME_SIZE).collect();

            let input_frame: Vec<f32> = frame.iter().map(|s| s * 32767.0).collect();
            let mut output_frame = vec![0.0f32; RNNOISE_FRAME_SIZE];

            let _vad_prob = self.denoiser.process_frame(&mut output_frame, &input_frame);

            for i in 0..RNNOISE_FRAME_SIZE {
                let clean = output_frame[i] / 32767.0;
                let original = frame[i];
                output.push(original * (1.0 - mix) + clean * mix);
            }
        }

        if !output.is_empty() {
            for (i, sample) in output.iter().enumerate() {
                if i < data.len() {
                    data[i] = *sample;
                }
            }
        }
    }

    // ── Ulunas (ONNX) ──────────────────────────────────────────────────────

    fn apply_ulunas(&mut self, data: &mut Vec<f32>, intensity: f32) {
        // Lazy init
        if self.ulunas.is_none() {
            if let Some(path) = &self.ulunas_model_path {
                if path.exists() {
                    match UlunasProcessor::new(path.to_str().unwrap_or("")) {
                        Ok(proc) => {
                            eprintln!("[DSP] Ulunas ONNX model loaded: {:?}", path);
                            self.ulunas = Some(proc);
                        }
                        Err(e) => {
                            eprintln!("[DSP] Failed to load Ulunas model: {}", e);
                            // Fallback to RNNoise
                            self.apply_rnnoise(data, intensity);
                            return;
                        }
                    }
                } else {
                    eprintln!("[DSP] Ulunas model not found at {:?}, falling back to RNNoise", path);
                    self.apply_rnnoise(data, intensity);
                    return;
                }
            } else {
                self.apply_rnnoise(data, intensity);
                return;
            }
        }

        let mix = (intensity / 100.0).clamp(0.0, 1.0);
        self.ulunas_buffer.extend_from_slice(data);

        let mut output = Vec::with_capacity(data.len());

        while self.ulunas_buffer.len() >= RNNOISE_FRAME_SIZE {
            let frame: Vec<f32> = self.ulunas_buffer.drain(..RNNOISE_FRAME_SIZE).collect();

            if let Some(ulunas) = &mut self.ulunas {
                let processed = ulunas.process(&frame);
                for i in 0..RNNOISE_FRAME_SIZE {
                    let clean = if i < processed.len() { processed[i] } else { frame[i] };
                    output.push(frame[i] * (1.0 - mix) + clean * mix);
                }
            } else {
                output.extend_from_slice(&frame);
            }
        }

        if !output.is_empty() {
            for (i, sample) in output.iter().enumerate() {
                if i < data.len() {
                    data[i] = *sample;
                }
            }
        }
    }

    // ── Speexdsp (spectral subtraction) ─────────────────────────────────────

    fn apply_speex(&mut self, data: &mut Vec<f32>, intensity: f32) {
        self.speex_ns.process(data, intensity);
    }

    // ── Dereverb ────────────────────────────────────────────────────────────

    fn apply_dereverb(&self, data: &mut Vec<f32>, level: f32) {
        let alpha = 0.02 + (level / 100.0) * 0.15;
        let mut prev = 0.0_f32;
        for sample in data.iter_mut() {
            let filtered = *sample - prev + (1.0 - alpha) * *sample;
            prev = *sample;
            *sample = filtered * 0.5;
        }
    }

    // ── AGC ─────────────────────────────────────────────────────────────────

    fn apply_agc(&mut self, data: &mut Vec<f32>, target: f32, attack: f32, decay: f32) {
        let target_linear = target / 32767.0;
        for sample in data.iter_mut() {
            let abs_sample = sample.abs();
            if abs_sample > self.agc_envelope {
                self.agc_envelope += attack * (abs_sample - self.agc_envelope);
            } else {
                self.agc_envelope += decay * (abs_sample - self.agc_envelope);
            }
            if self.agc_envelope > 1e-6 {
                let desired_gain = target_linear / self.agc_envelope;
                let clamped_gain = desired_gain.clamp(0.1, 10.0);
                *sample *= clamped_gain;
            }
        }
    }

    // ── VAD ─────────────────────────────────────────────────────────────────

    fn apply_vad(&mut self, data: &mut Vec<f32>, threshold_db: f32) {
        let rms = compute_rms(data);
        let rms_db = if rms > 1e-10 { 20.0 * rms.log10() } else { -100.0 };
        let target_fade = if rms_db >= threshold_db { 1.0 } else { 0.0 };
        let fade_speed = if target_fade > self.vad_fade { 0.1 } else { 0.02 };
        self.vad_fade += fade_speed * (target_fade - self.vad_fade);
        self.vad_fade = self.vad_fade.clamp(0.0, 1.0);
        for sample in data.iter_mut() {
            *sample *= self.vad_fade;
        }
    }

    // ── Spectrum ─────────────────────────────────────────────────────────────

    fn compute_spectrum(&mut self, data: &[f32], is_raw: bool) {
        let bands = 64;
        let target = if is_raw { &mut self.raw_spectrum } else { &mut self.processed_spectrum };
        if target.len() != bands { target.resize(bands, 0.0); }
        if data.is_empty() {
            for v in target.iter_mut() { *v = 0.0; }
            return;
        }

        for (band_idx, band_val) in target.iter_mut().enumerate() {
            let start = (band_idx as f32 / bands as f32).powf(1.5) * data.len() as f32;
            let end = (((band_idx + 1) as f32) / bands as f32).powf(1.5) * data.len() as f32;
            let start = start as usize;
            let end = (end as usize).min(data.len());
            if start >= end { *band_val *= 0.85; continue; }
            let mut sum = 0.0_f32;
            for i in start..end { sum += data[i] * data[i]; }
            let rms = (sum / (end - start) as f32).sqrt();
            let db = if rms > 1e-10 { 20.0 * rms.log10() } else { -100.0 };
            let normalized = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
            if normalized > *band_val { *band_val = normalized; }
            else { *band_val = *band_val * 0.85 + normalized * 0.15; }
        }
    }
}

fn compute_rms(data: &[f32]) -> f32 {
    if data.is_empty() { return 0.0; }
    let sum: f32 = data.iter().map(|s| s * s).sum();
    (sum / data.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_positive() {
        let settings = Arc::new(RwLock::new(AudioDspSettings {
            gain: 20.0,
            ..Default::default()
        }));
        let mut processor = DspProcessor::new(settings, None);
        let mut data = vec![0.1; 480];
        processor.process(&mut data);
        assert!(data[0] > 0.9, "Expected amplified sample, got {}", data[0]);
    }

    #[test]
    fn test_gain_negative() {
        let settings = Arc::new(RwLock::new(AudioDspSettings {
            gain: -20.0,
            ..Default::default()
        }));
        let mut processor = DspProcessor::new(settings, None);
        let mut data = vec![0.5; 480];
        processor.process(&mut data);
        assert!(data[0] < 0.1, "Expected attenuated sample, got {}", data[0]);
    }

    #[test]
    fn test_vad_mutes_quiet() {
        let settings = Arc::new(RwLock::new(AudioDspSettings {
            vad_enabled: true,
            vad_threshold: -10.0,
            ..Default::default()
        }));
        let mut processor = DspProcessor::new(settings, None);
        let mut data = vec![0.001; 960];
        for _ in 0..20 { processor.process(&mut data); }
        assert!(data[data.len() - 1].abs() < 0.01, "Expected muted, got {}", data[data.len() - 1]);
    }

    #[test]
    fn test_agc_boosts_quiet() {
        let settings = Arc::new(RwLock::new(AudioDspSettings {
            agc_enabled: true,
            agc_target: 16000.0,
            agc_attack: 90.0,
            agc_decay: 10.0,
            ..Default::default()
        }));
        let mut processor = DspProcessor::new(settings, None);
        let mut data: Vec<f32> = vec![0.01; 4800];
        for _ in 0..10 { processor.process(&mut data); }
        assert!(data[data.len() - 1].abs() > 0.01, "AGC should have amplified the signal");
    }
}
