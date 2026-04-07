use crate::profile::ProfileIndex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppScreen {
    Main,
    ConfirmSwitch,
    ConfirmDelete,
    InputName,
    InputNote,
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
    pub message: Option<String>,
    pub decrypt_output: Option<String>,
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
            message: None,
            decrypt_output: None,
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
