use std::sync::mpsc::channel;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use freq_det::FreqDetector;

fn main() {
    let host = cpal::default_host();
    let dev = host.default_output_device().unwrap();
    let conf = dev.default_input_config().unwrap();

    let input_channels = conf.channels();
    let sample_rate = conf.sample_rate().0;

    let (sound_sender, sound_receiver) = channel();
    let stream = dev
        .build_input_stream(
            &conf.into(),
            move |data: &[f32], _| {
                // take only samples from the first channel, usually this means left
                for sample in data.iter().step_by(input_channels as usize) {
                    sound_sender.send(*sample).unwrap();
                }
            },
            |e| eprintln!("Error reading data from input: {e}"),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    let sample_count = 4096;
    let freq_det = FreqDetector::new(sample_rate as usize, sample_count).unwrap();
    let mut buffer = vec![];
    loop {
        while buffer.len() < sample_count {
            buffer.push(sound_receiver.recv().unwrap());
        }
        println!("{}", freq_det.detect(&buffer).unwrap());
        buffer.clear();
    }
}
