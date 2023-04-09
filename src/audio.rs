use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

pub struct Audio {
    stream_handle: OutputStreamHandle,
}

impl Audio {
    pub fn new() -> Audio {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        Audio { stream_handle }
    }
    pub fn play(&self) {
        let file = BufReader::new(File::open("beep.wav").unwrap());
        let source = Decoder::new(file).unwrap();
        match self.stream_handle.play_raw(source.convert_samples()) {
            Ok(_) => {}
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
