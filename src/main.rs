use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the cpal host
    let host = cpal::default_host();

    // Get the default input and output devices
    let input_device = host.default_input_device().expect("No input device available");
    let output_device = host.default_output_device().expect("No output device available");

    // Get the input and output stream configurations
    let input_config = input_device.default_input_config()?;
    let output_config = output_device.default_output_config()?;

    // Create an OutputStream and Sink for playback
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // Buffer to hold the audio data
    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    // Clone the audio buffer for the input stream closure
    let audio_buffer_input = Arc::clone(&audio_buffer);

    // Build the input stream
    let input_stream = input_device.build_input_stream(
        &input_config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut buffer = audio_buffer_input.lock().unwrap();
            // Invert the audio signal
            buffer.extend(data.iter().map(|&sample| -sample));
        },
        move |err| {
            eprintln!("An error occurred on the input audio stream: {:?}", err);
        },
        Some(Duration::from_millis(100)), // Added missing argument
    )?;

    // Start the input stream
    input_stream.play()?;

    // Playback loop
    loop {
        // Get the audio data from the buffer
        let mut buffer = audio_buffer.lock().unwrap();
        if !buffer.is_empty() {
            let source = rodio::buffer::SamplesBuffer::new(
                output_config.channels() as u16,
                output_config.sample_rate().0,
                buffer.clone(),
            );
            sink.append(source);
            buffer.clear();
        }
    }
}
