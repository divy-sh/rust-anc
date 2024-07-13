use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use rodio::{Decoder, OutputStream, Sink};
use std::sync::{Arc, Mutex};

fn main() {
    // Get the default host.
    let host = cpal::default_host();

    // Get the default input device.
    let input_device = host.default_input_device().expect("Failed to get default input device");
    println!("Using input device: {}", input_device.name().unwrap());

    // Get the default output device.
    let output_device = host.default_output_device().expect("Failed to get default output device");
    println!("Using output device: {}", output_device.name().unwrap());

    // Get the default input format.
    let format = input_device.default_input_format().expect("Failed to get default input format");

    // Create an event loop.
    let event_loop = host.event_loop();

    // Create an input stream.
    let input_stream_id = event_loop.build_input_stream(&input_device, &format).expect("Failed to build input stream");

    // Create an output stream.
    let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to create output stream");
    let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");

    // Shared buffer for audio data.
    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    // Clone buffer for the event loop closure.
    let audio_buffer_clone = audio_buffer.clone();

    // Play the stream.
    event_loop.play_stream(input_stream_id.clone()).expect("Failed to play input stream");

    // Handle input stream data.
    event_loop.run(move |_stream_id, stream_result| {
        let stream_data = match stream_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Stream error: {:?}", err);
                return;
            }
        };

        if let cpal::StreamData::Input { buffer } = stream_data {
            match buffer {
                cpal::UnknownTypeInputBuffer::U16(buffer) => {
                    let mut data: Vec<u16> = buffer.iter().cloned().collect();
                    data.reverse();
                    let reversed_data: Vec<u8> = data.iter().flat_map(|&sample| sample.to_ne_bytes()).collect();
                    let mut audio_buffer = audio_buffer_clone.lock().unwrap();
                    audio_buffer.extend(reversed_data);
                },
                cpal::UnknownTypeInputBuffer::I16(buffer) => {
                    let mut data: Vec<i16> = buffer.iter().cloned().collect();
                    data.reverse();
                    let reversed_data: Vec<u8> = data.iter().flat_map(|&sample| sample.to_ne_bytes()).collect();
                    let mut audio_buffer = audio_buffer_clone.lock().unwrap();
                    audio_buffer.extend(reversed_data);
                },
                cpal::UnknownTypeInputBuffer::F32(buffer) => {
                    let mut data: Vec<f32> = buffer.iter().cloned().collect();
                    data.reverse();
                    let reversed_data: Vec<u8> = data.iter().flat_map(|&sample| sample.to_ne_bytes()).collect();
                    let mut audio_buffer = audio_buffer_clone.lock().unwrap();
                    audio_buffer.extend(reversed_data);
                },
            }

            // Play the reversed audio data.
            let audio_data = {
                let mut buffer = audio_buffer.lock().unwrap();
                if buffer.is_empty() {
                    return;
                }
                buffer.split_off(0)
            };
            if !audio_data.is_empty() {
                let cursor = std::io::Cursor::new(audio_data);
                let source = Decoder::new(cursor).expect("Failed to decode audio data");
                sink.append(source);
            }
        }
    });
}
