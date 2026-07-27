use crate::state::SharedPlayerState;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::Button;

pub fn play_track(state: &SharedPlayerState, index: usize, play_btn: &Button) {
    let song = {
        let st = state.borrow();
        if index >= st.playlist.len() {
            return;
        }
        st.playlist[index].clone()
    };

    let vol = state.borrow().volume;

    state.borrow().engine.play_file(song.path.clone());
    state.borrow().engine.set_volume(vol);

    state.borrow_mut().current_index = Some(index);
    state.borrow_mut().current_media = Some(song.path.clone());

    if let Some(ref lcd) = state.borrow().lcd_widgets {
        lcd.stack.set_visible_child_name("player");
        lcd.title_label.set_label(&song.title);
        lcd.artist_label.set_label(&song.artist);

        let duration = song.duration_secs;
        let mins = duration / 60;
        let secs = duration % 60;
        lcd.duration_label
            .set_label(&format!("{}:{:02}", mins, secs));

        lcd.progress_scale.set_range(0.0, duration as f64);
        lcd.progress_scale.set_value(0.0);
        lcd.time_label.set_label("0:00");
    }

    play_btn.set_icon_name("media-playback-pause-symbolic");
}

pub fn toggle_play_pause(state: &SharedPlayerState, play_btn: &Button) {
    let has_media = state.borrow().current_media.is_some();
    let playlist_empty = state.borrow().playlist.is_empty();

    if playlist_empty {
        return;
    }

    if has_media {
        let is_playing = state
            .borrow()
            .engine
            .is_playing
            .load(std::sync::atomic::Ordering::SeqCst);
        if is_playing {
            state.borrow().engine.pause();
            play_btn.set_icon_name("media-playback-start-symbolic");
        } else {
            state.borrow().engine.resume();
            play_btn.set_icon_name("media-playback-pause-symbolic");
        }
    } else {
        play_track(state, 0, play_btn);
    }
}

pub fn play_previous_track(state: &SharedPlayerState, play_btn: &Button) {
    let (has_media, current_idx, playlist_len) = {
        let st = state.borrow();
        (
            st.current_media.is_some(),
            st.current_index,
            st.playlist.len(),
        )
    };

    if playlist_len == 0 {
        return;
    }

    if has_media {
        let timestamp = state
            .borrow()
            .engine
            .timestamp_us
            .load(std::sync::atomic::Ordering::SeqCst);
        if timestamp > 3_000_000 {
            state.borrow().engine.seek(0);
            return;
        }
    }

    if let Some(idx) = current_idx {
        if idx > 0 {
            play_track(state, idx - 1, play_btn);
        } else {
            if has_media {
                state.borrow().engine.seek(0);
            } else {
                play_track(state, 0, play_btn);
            }
        }
    } else {
        play_track(state, 0, play_btn);
    }
}

pub fn play_next_track(state: &SharedPlayerState, play_btn: &Button) {
    let (current_idx, playlist_len) = {
        let st = state.borrow();
        (st.current_index, st.playlist.len())
    };

    if playlist_len == 0 {
        return;
    }

    if let Some(idx) = current_idx {
        if idx + 1 < playlist_len {
            play_track(state, idx + 1, play_btn);
        } else {
            state.borrow().engine.stop();
            play_btn.set_icon_name("media-playback-start-symbolic");
            if let Some(ref mut lcd) = state.borrow_mut().lcd_widgets {
                lcd.time_label.set_label("0:00");
                lcd.progress_scale.set_value(0.0);
            }
            state.borrow_mut().current_media = None;
            state.borrow_mut().current_index = None;
        }
    } else {
        play_track(state, 0, play_btn);
    }
}
