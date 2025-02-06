use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Stream,
};
use ringbuf::traits::{Consumer as _, Producer as _, Split as _};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::{compressor::Compressor, VOLUME};

pub fn get_devices() -> Vec<Device> {
    cpal::default_host()
        .devices()
        .expect("Could not find devices")
        .collect()
}

pub fn create_stream(
    input_device: &Device,
    output_device: &Device,
    target_loudness: f32,
    debug: bool,
) -> Option<(Stream, Stream)> {
    let input_config: cpal::StreamConfig = input_device.default_input_config().ok()?.into();
    let output_config: cpal::StreamConfig = output_device.default_output_config().ok()?.into();

    // 确保输入输出配置匹配
    assert_eq!(
        input_config.channels, output_config.channels,
        "Input and output channel count must match"
    );
    assert_eq!(
        input_config.sample_rate, output_config.sample_rate,
        "Input and output sample rate must match"
    );

    let comp = Arc::new(Mutex::new(Compressor::new(
        input_config.sample_rate.0,
        input_config.channels as u32,
        target_loudness as f64,
    )));

    // 创建一个线程定期生成波形图
    if debug {
        let comp_clone = Arc::clone(&comp);
        thread::spawn(move || {
            let mut counter = 0;
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(comp) = comp_clone.lock() {
                    if let Err(e) = comp.plot_waveforms(&format!("waveform_{}.png", counter)) {
                        eprintln!("Failed to plot waveform: {}", e);
                    }
                }
                counter = (counter + 1) % 10;
            }
        });
    }

    let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);

    let latency = 100.0;
    let latency_frames = (latency / 1000.0) * input_config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * input_config.channels as usize;

    let ring = Box::new(ringbuf::HeapRb::new(latency_samples * 2));
    let (mut producer, mut consumer) = ring.split();

    for _ in 0..latency_samples {
        producer.try_push(0.0).unwrap();
    }

    let comp_clone = Arc::clone(&comp);
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        if let Ok(mut comp) = comp_clone.lock() {
            comp.set_target_loudness(VOLUME.load(Ordering::SeqCst) as f64);

            // 处理整个帧
            let output = comp.compress_frame(data);

            // 写入输出缓冲区
            for &sample in &output {
                if producer.try_push(sample).is_err() {
                    // eprintln!("Output stream fell behind: try increasing latency");
                }
            }
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data {
            *sample = consumer.try_pop().unwrap_or(0.0);
        }
    };

    let input_stream = input_device
        .build_input_stream(&input_config, input_data_fn, err_fn, None)
        .expect("Error building input stream");
    let output_stream = output_device
        .build_output_stream(&output_config, output_data_fn, err_fn, None)
        .expect("Error building output stream");

    input_stream.play().expect("Could not play input stream");
    output_stream.play().expect("Could not play output stream");

    Some((input_stream, output_stream))
}
