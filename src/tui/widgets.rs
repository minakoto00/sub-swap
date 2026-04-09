use crate::profile::ProfileIndex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppScreen {
    Main,
    ConfirmSwitch,
    ConfirmDelete,
    InputName,
    InputNote,
    InputPassphrase,
    ViewDecrypt,
    ForceSwitch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Switch,
    Add,
    Rename,
    Delete,
    Note,
    View,
}

pub struct AppState {
    pub screen: AppScreen,
    pub selected: usize,
    pub profile_names: Vec<String>,
    pub active_profile: Option<String>,
    pub pending_action: Option<Action>,
    pub input_buffer: String,
    pub passphrase_buffer: String,
    pub staged_input: Option<String>,
    pub message: Option<String>,
    pub decrypt_output: Option<String>,
    pub scroll_offset: u16,
    pub should_quit: bool,
}

impl AppState {
    pub fn from_index(index: &ProfileIndex) -> Self {
        let profile_names: Vec<String> =
            index.names().into_iter().map(ToString::to_string).collect();
        let active_profile = index.active_profile.clone();
        Self {
            screen: AppScreen::Main,
            selected: 0,
            profile_names,
            active_profile,
            pending_action: None,
            input_buffer: String::new(),
            passphrase_buffer: String::new(),
            staged_input: None,
            message: None,
            decrypt_output: None,
            scroll_offset: 0,
            should_quit: false,
        }
    }

    pub fn selected_name(&self) -> Option<&str> {
        self.profile_names.get(self.selected).map(String::as_str)
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.profile_names.is_empty() && self.selected < self.profile_names.len() - 1 {
            self.selected += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{Profile, ProfileIndex};

    fn test_index() -> ProfileIndex {
        let mut index = ProfileIndex::default();
        index.add(Profile::new("work", Some("Work account".into())));
        index.add(Profile::new("personal", None));
        index.set_active("work");
        index
    }

    #[test]
    fn test_from_index_initializes_scroll_offset_to_zero() {
        let state = AppState::from_index(&test_index());
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_from_index_sets_active_profile() {
        let state = AppState::from_index(&test_index());
        assert_eq!(state.active_profile.as_deref(), Some("work"));
    }

    #[test]
    fn test_from_index_collects_sorted_names() {
        let state = AppState::from_index(&test_index());
        assert_eq!(state.profile_names, vec!["personal", "work"]);
    }

    #[test]
    fn test_from_index_initializes_empty_passphrase_buffer() {
        let state = AppState::from_index(&test_index());
        assert!(state.passphrase_buffer.is_empty());
    }
}
