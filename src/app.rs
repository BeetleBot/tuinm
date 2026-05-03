use ratatui::widgets::TableState;
use crate::nm::{APInfo, NMService, SavedConnection};
use anyhow::Result;

#[derive(PartialEq, Clone, Copy)]
pub enum FocusedPane {
    Saved,
    Available,
}

pub struct App {
    pub aps: Vec<APInfo>,
    pub saved: Vec<SavedConnection>,
    pub active_ssids: Vec<String>,
    pub available_state: TableState,
    pub saved_state: TableState,
    pub focused_pane: FocusedPane,
    pub nm: NMService,
    pub should_quit: bool,
    pub is_confirming_delete: bool,
    pub is_inputting_password: bool,
    pub password_input: String,
    pub status_msg: Option<String>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let nm = NMService::new().await?;
        let mut available_state = TableState::default();
        available_state.select(Some(0));
        let mut saved_state = TableState::default();
        saved_state.select(Some(0));

        Ok(Self {
            aps: Vec::new(),
            saved: Vec::new(),
            active_ssids: Vec::new(),
            available_state,
            saved_state,
            focused_pane: FocusedPane::Available,
            nm,
            should_quit: false,
            is_confirming_delete: false,
            is_inputting_password: false,
            password_input: String::new(),
            status_msg: None,
        })
    }

    pub async fn refresh_data(&mut self) -> Result<()> {
        let devices = self.nm.list_wifi_devices().await?;
        if let Some(device_path) = devices.first() {
            self.aps = self.nm.get_access_points(device_path.clone()).await?;
        }
        self.saved = self.nm.get_saved_connections().await?;
        self.active_ssids = self.nm.get_active_ssids().await.unwrap_or_default();
        Ok(())
    }

    pub fn next(&mut self) {
        match self.focused_pane {
            FocusedPane::Available => {
                if self.aps.is_empty() { return; }
                let i = match self.available_state.selected() {
                    Some(i) => if i >= self.aps.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.available_state.select(Some(i));
            }
            FocusedPane::Saved => {
                if self.saved.is_empty() { return; }
                let i = match self.saved_state.selected() {
                    Some(i) => if i >= self.saved.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.saved_state.select(Some(i));
            }
        }
    }

    pub fn previous(&mut self) {
        match self.focused_pane {
            FocusedPane::Available => {
                if self.aps.is_empty() { return; }
                let i = match self.available_state.selected() {
                    Some(i) => if i == 0 { self.aps.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.available_state.select(Some(i));
            }
            FocusedPane::Saved => {
                if self.saved.is_empty() { return; }
                let i = match self.saved_state.selected() {
                    Some(i) => if i == 0 { self.saved.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.saved_state.select(Some(i));
            }
        }
    }

    pub fn switch_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Available => FocusedPane::Saved,
            FocusedPane::Saved => FocusedPane::Available,
        };
    }
}
