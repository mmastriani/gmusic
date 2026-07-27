use crate::models::Song;
use crate::audio::AudioEngine;
use gtk4 as gtk;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub struct LcdWidgets {
    pub stack: gtk::Stack,
    pub title_label: gtk::Label,
    pub artist_label: gtk::Label,
    pub time_label: gtk::Label,
    pub duration_label: gtk::Label,
    pub progress_scale: gtk::Scale,
}

pub struct PlayerState {
    pub playlist: Vec<Song>,
    pub current_index: Option<usize>,
    pub engine: Rc<AudioEngine>,
    pub current_media: Option<PathBuf>,
    pub volume: f64,
    pub lcd_widgets: Option<LcdWidgets>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::default_with_vol()
    }
}

impl PlayerState {
    pub fn default_with_vol() -> Self {
        Self {
            playlist: Vec::new(),
            current_index: None,
            engine: Rc::new(AudioEngine::default()),
            current_media: None,
            volume: 0.5,
            lcd_widgets: None,
        }
    }
}

pub type SharedPlayerState = Rc<RefCell<PlayerState>>;
