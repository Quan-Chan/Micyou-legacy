use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig, OutputCallbackInfo};
use ringbuf::{HeapRb, Producer};
use std::sync::Arc;

pub struct AudioOutputManager {
    stream: Option<cpal::Stream>,
    producer: Option<Producer<f32, Arc<HeapRb<f32>>>>,
}

impl AudioOutputManager {
    pub fn new() -> Self {
        Self {
            stream: None,
            producer: None,
        }
    }

    pub fn start(&mut self, target_device: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        
        let device = if let Some(target) = target_device.clone() {
            let mut matched_device = None;
            if let Ok(devices) = host.output_devices() {
                for dev in devices {
                    if let Ok(name) = dev.name() {
                        if name == target {
                            matched_device = Some(dev);
                            break;
                        }
                    }
                }
            }
            if matched_device.is_none() {
                eprintln!("Could not find exact device: {}, falling back to default.", target);
            }
            matched_device.or_else(|| host.default_output_device())
        } else {
            host.default_output_device()
        };

        let device = device.ok_or("No output device available")?;
        
        let config = device.default_output_config()?;
        let _sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        // Initialize a ring buffer for 1 second of audio
        let ring_buffer = HeapRb::<f32>::new(48000 * channels);
        let (producer, mut consumer) = ring_buffer.split();

        self.producer = Some(producer);

        let stream_config: StreamConfig = config.clone().into();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        *sample = consumer.pop().unwrap_or(0.0);
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => device.build_output_stream(
                &stream_config,
                move |data: &mut [i16], _: &OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        let f_sample = consumer.pop().unwrap_or(0.0);
                        *sample = (f_sample * i16::MAX as f32) as i16;
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn push_audio_data(&mut self, data: &[f32]) {
        if let Some(producer) = &mut self.producer {
            producer.push_slice(data);
        }
    }

    pub fn queued_samples(&self) -> usize {
        if let Some(producer) = &self.producer {
            producer.len()
        } else {
            0
        }
    }
}
