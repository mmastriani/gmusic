use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

use gstreamer as gst;
use gstreamer::prelude::*;

pub enum AudioCommand {
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
    Seek(u64), // microseconds
    SetVolume(f64),
}

pub struct AudioEngine {
    tx: std::sync::mpsc::Sender<AudioCommand>,
    pub timestamp_us: Arc<AtomicU64>,
    pub is_playing: Arc<AtomicBool>,
    pub is_ended: Arc<AtomicBool>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<AudioCommand>();
        let timestamp_us = Arc::new(AtomicU64::new(0));
        let is_playing = Arc::new(AtomicBool::new(false));
        let is_ended = Arc::new(AtomicBool::new(false));

        let ts_clone = timestamp_us.clone();
        let play_clone = is_playing.clone();
        let ended_clone = is_ended.clone();

        thread::spawn(move || {
            let playbin = gst::ElementFactory::make("playbin").build().expect("Failed to create playbin element");
            let bus = playbin.bus().unwrap();
            
            let mut volume = 0.5;
            let mut is_media_playing = false;

            loop {
                // Process incoming commands
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        AudioCommand::Play(path) => {
                            let _ = playbin.set_state(gst::State::Ready);
                            let uri = format!("file://{}", path.to_string_lossy());
                            playbin.set_property("uri", uri);
                            playbin.set_property("volume", volume);
                            let _ = playbin.set_state(gst::State::Playing);
                            
                            is_media_playing = true;
                            ts_clone.store(0, Ordering::SeqCst);
                            play_clone.store(true, Ordering::SeqCst);
                            ended_clone.store(false, Ordering::SeqCst);
                        }
                        AudioCommand::Pause => {
                            if is_media_playing {
                                let _ = playbin.set_state(gst::State::Paused);
                                play_clone.store(false, Ordering::SeqCst);
                            }
                        }
                        AudioCommand::Resume => {
                            if is_media_playing {
                                let _ = playbin.set_state(gst::State::Playing);
                                play_clone.store(true, Ordering::SeqCst);
                            }
                        }
                        AudioCommand::Stop => {
                            let _ = playbin.set_state(gst::State::Null);
                            is_media_playing = false;
                            play_clone.store(false, Ordering::SeqCst);
                            ended_clone.store(true, Ordering::SeqCst);
                            ts_clone.store(0, Ordering::SeqCst);
                        }
                        AudioCommand::Seek(us) => {
                            if is_media_playing {
                                let pos = gst::ClockTime::from_useconds(us);
                                let _ = playbin.seek_simple(
                                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                    pos,
                                );
                            }
                        }
                        AudioCommand::SetVolume(v) => {
                            volume = v;
                            playbin.set_property("volume", volume);
                        }
                    }
                }

                // Process bus messages
                for msg in bus.iter() {
                    match msg.view() {
                        gst::MessageView::Eos(..) => {
                            let _ = playbin.set_state(gst::State::Null);
                            is_media_playing = false;
                            play_clone.store(false, Ordering::SeqCst);
                            ended_clone.store(true, Ordering::SeqCst);
                        }
                        gst::MessageView::Error(err) => {
                            eprintln!("Error from {:?}: {} ({:?})",
                                err.src().map(|s| s.path_string()),
                                err.error(),
                                err.debug()
                            );
                            let _ = playbin.set_state(gst::State::Null);
                            is_media_playing = false;
                            play_clone.store(false, Ordering::SeqCst);
                            ended_clone.store(true, Ordering::SeqCst);
                        }
                        _ => (),
                    }
                }

                // Update timestamp
                if is_media_playing {
                    if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                        ts_clone.store(pos.useconds(), Ordering::SeqCst);
                    }
                }

                thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        Self {
            tx,
            timestamp_us,
            is_playing,
            is_ended,
        }
    }
}

impl AudioEngine {
    pub fn play_file(&self, path: PathBuf) {
        let _ = self.tx.send(AudioCommand::Play(path));
    }
    pub fn pause(&self) { let _ = self.tx.send(AudioCommand::Pause); }
    pub fn resume(&self) { let _ = self.tx.send(AudioCommand::Resume); }
    pub fn stop(&self) { let _ = self.tx.send(AudioCommand::Stop); }
    pub fn seek(&self, us: u64) { let _ = self.tx.send(AudioCommand::Seek(us)); }
    pub fn set_volume(&self, vol: f64) { let _ = self.tx.send(AudioCommand::SetVolume(vol)); }
}
