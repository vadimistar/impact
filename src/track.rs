use color_eyre::Result;
use rodio::{Sink, OutputStreamHandle};
use std::{
    fs::File,
    io::BufReader,
};

pub struct Track(Sink);

impl Track {
    pub fn try_new(source: File, stream_handle: &OutputStreamHandle) -> Result<Track> {
        let sink = stream_handle.play_once(BufReader::new(source))?;
        Ok(Track(sink))
    }

    pub fn play(&self) {
        self.0.play();
    }

    pub fn pause(&self) {
        self.0.pause();
    }

    pub fn stop(&self) {
        self.0.stop();
    }
}