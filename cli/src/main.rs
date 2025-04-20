//use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SupportedStreamConfig};
use sstv_decoder_lib::sstv_decoder::SSTVDecoder;
fn main() {
    // Get the first command line argument.
    //let args: Vec<String> = std::env::args().collect();
    //let path = args.get(1).expect("file path not provided");

    // Open the media source.
    let src = std::fs::File::open("/home/superbart/下载/20250405_154539.wav")
        .expect("failed to open media");

    // read from file
    let (head, samples) = wav_io::read_from_file(src).unwrap();

    // show header info
    println!("header={:?}", head);

    // show samples
    println!("samples.len={}", samples.len());

    let mut sstv_decoder: SSTVDecoder = SSTVDecoder::new(head.sample_rate as f32);

    let mut cache: Vec<f32> = vec![];
    for (i, v) in samples.iter().enumerate() {
        if i % 1024 == 0 && i != 0 {
            sstv_decoder.decode(&cache);
            cache.clear();
        }
        cache.push(*v);
    }
    if cache.len() != 0 {
        sstv_decoder.decode(&cache);
    }

    /*
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("failed to find input device");
    let default_config = device.default_input_config().unwrap();
    let config = SupportedStreamConfig::new(
        1,                            // mono
        default_config.sample_rate(), // sample rate
        default_config.buffer_size().clone(),
        cpal::SampleFormat::I16,
    );
    eprintln!("Device info {:?}", &config);

    let mut sstv_decoder: SSTVDecoder = SSTVDecoder::new(default_config.sample_rate().0 as f32);
    let stream = device
        .build_input_stream(
            &config.config(),
            &config.config(),
            move |data: & [f32], _| {
                println!("{:?}\n",&data);
                sstv_decoder.decode( data);
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();
    loop {}
     */
}
