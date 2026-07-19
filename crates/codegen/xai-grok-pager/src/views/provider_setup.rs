//! Provider setup form (WeepCode).
//!
//! Rendered on the welcome screen when the shell advertises the
//! `provider.setup` auth method (no usable credentials). Collects the five
//! pieces a provider profile needs — API format, base URL, API key, model id,
//! display name — and submits them to the agent via the
//! `weepcode/provider/save` ACP extension, which persists a `[model.*]` entry
//! in `config.toml`. Replaces the old xAI browser-OAuth login wall.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::theme::Theme;

/// Selectable API formats, in menu order. The `wire` string is what the
/// `weepcode/provider/save` extension expects; `base_url_preset` pre-fills the
/// URL field while the user hasn't typed one of their own.
pub struct ProviderFormatSpec {
    pub wire: &'static str,
    pub label: &'static str,
    pub base_url_preset: &'static str,
}

pub const PROVIDER_FORMATS: [ProviderFormatSpec; 3] = [
    ProviderFormatSpec {
        wire: "openai-responses",
        label: "OpenAI Responses",
        base_url_preset: "https://api.openai.com/v1",
    },
    ProviderFormatSpec {
        wire: "openai-compatible",
        label: "OpenAI Compatible",
        base_url_preset: "https://api.openai.com/v1",
    },
    ProviderFormatSpec {
        wire: "anthropic",
        label: "Anthropic",
        base_url_preset: "https://api.anthropic.com/v1",
    },
];

/// The four text inputs, in tab order (the format selector sits above them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderSetupField {
    Format,
    BaseUrl,
    ApiKey,
    ModelId,
    DisplayName,
}

impl ProviderSetupField {
    pub const ALL: [ProviderSetupField; 5] = [
        Self::Format,
        Self::BaseUrl,
        Self::ApiKey,
        Self::ModelId,
        Self::DisplayName,
    ];

    fn index(self) -> isize {
        match self {
            Self::Format => 0,
            Self::BaseUrl => 1,
            Self::ApiKey => 2,
            Self::ModelId => 3,
            Self::DisplayName => 4,
        }
    }

    fn from_index(index: isize) -> Self {
        Self::ALL[index.rem_euclid(Self::ALL.len() as isize) as usize]
    }
}

/// Outcome of feeding one key to the form; the caller maps it onto
/// `InputOutcome` / `Action`.
#[derive(Debug, PartialEq, Eq)]
pub enum ProviderSetupOutcome {
    Unchanged,
    Changed,
    /// All fields valid — caller should dispatch the submit action.
    Submit,
    /// Esc pressed — caller should close the form.
    Cancel,
}

/// Live state of the on-screen form.
pub struct ProviderSetupForm {
    pub format_index: usize,
    pub base_url: String,
    pub api_key: String,
    pub model_id: String,
    pub display_name: String,
    pub focused: ProviderSetupField,
    /// Last validation or save error, shown under the fields.
    pub error: Option<String>,
    /// True while the `weepcode/provider/save` request is in flight.
    pub submitting: bool,
    /// Tracks whether the base URL still holds a preset (so a format switch
    /// may replace it) or was edited by the user (and must be preserved).
    base_url_is_preset: bool,
}

impl Default for ProviderSetupForm {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderSetupForm {
    pub fn new() -> Self {
        Self {
            format_index: 0,
            base_url: PROVIDER_FORMATS[0].base_url_preset.to_string(),
            api_key: String::new(),
            model_id: String::new(),
            display_name: String::new(),
            focused: ProviderSetupField::Format,
            error: None,
            submitting: false,
            base_url_is_preset: true,
        }
    }

    pub fn selected_format(&self) -> &'static ProviderFormatSpec {
        &PROVIDER_FORMATS[self.format_index]
    }

    fn focused_text_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            ProviderSetupField::Format => None,
            ProviderSetupField::BaseUrl => Some(&mut self.base_url),
            ProviderSetupField::ApiKey => Some(&mut self.api_key),
            ProviderSetupField::ModelId => Some(&mut self.model_id),
            ProviderSetupField::DisplayName => Some(&mut self.display_name),
        }
    }

    fn cycle_format(&mut self, forward: bool) {
        let len = PROVIDER_FORMATS.len();
        self.format_index = if forward {
            (self.format_index + 1) % len
        } else {
            (self.format_index + len - 1) % len
        };
        if self.base_url_is_preset {
            self.base_url = self.selected_format().base_url_preset.to_string();
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.base_url.trim().is_empty() {
            return Err("Base URL is required".to_string());
        }
        if !(self.base_url.trim().starts_with("http://")
            || self.base_url.trim().starts_with("https://"))
        {
            return Err("Base URL must start with http:// or https://".to_string());
        }
        if self.api_key.trim().is_empty() {
            return Err("API key is required".to_string());
        }
        if self.model_id.trim().is_empty() {
            return Err("Model id is required".to_string());
        }
        if self.display_name.trim().is_empty() {
            return Err("Display name is required".to_string());
        }
        Ok(())
    }

    /// Feed one key press. Text fields take literal chars/backspace; the
    /// format row cycles with ←/→; Tab/↓ and Shift-Tab/↑ move focus; Enter
    /// validates and submits; Esc cancels.
    pub fn handle_key(&mut self, key: &KeyEvent) -> ProviderSetupOutcome {
        if self.submitting {
            return ProviderSetupOutcome::Unchanged;
        }
        self.error = None;
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => return ProviderSetupOutcome::Cancel,
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
                self.focused = ProviderSetupField::from_index(self.focused.index() + 1);
                return ProviderSetupOutcome::Changed;
            }
            (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Up, KeyModifiers::NONE) => {
                self.focused = ProviderSetupField::from_index(self.focused.index() - 1);
                return ProviderSetupOutcome::Changed;
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                return match self.validate() {
                    Ok(()) => {
                        self.submitting = true;
                        ProviderSetupOutcome::Submit
                    }
                    Err(e) => {
                        self.error = Some(e);
                        ProviderSetupOutcome::Changed
                    }
                };
            }
            _ => {}
        }
        if self.focused == ProviderSetupField::Format {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    self.cycle_format(false);
                    return ProviderSetupOutcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.cycle_format(true);
                    return ProviderSetupOutcome::Changed;
                }
                _ => return ProviderSetupOutcome::Unchanged,
            }
        }
        match key.code {
            KeyCode::Backspace => {
                if let Some(text) = self.focused_text_mut() {
                    text.pop();
                }
                if self.focused == ProviderSetupField::BaseUrl {
                    self.base_url_is_preset = false;
                }
                ProviderSetupOutcome::Changed
            }
            KeyCode::Char(c) => {
                if let Some(text) = self.focused_text_mut() {
                    text.push(c);
                }
                if self.focused == ProviderSetupField::BaseUrl {
                    self.base_url_is_preset = false;
                }
                ProviderSetupOutcome::Changed
            }
            _ => ProviderSetupOutcome::Unchanged,
        }
    }

    /// Paste support: append the cleaned clipboard text to the focused field.
    pub fn handle_paste(&mut self, text: &str) -> ProviderSetupOutcome {
        if self.submitting {
            return ProviderSetupOutcome::Unchanged;
        }
        let cleaned: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        if let Some(field) = self.focused_text_mut() {
            field.push_str(&cleaned);
            if self.focused == ProviderSetupField::BaseUrl {
                self.base_url_is_preset = false;
            }
            return ProviderSetupOutcome::Changed;
        }
        ProviderSetupOutcome::Unchanged
    }
}

/// Render the form into `content_area` (the welcome screen's main region).
/// Kept deliberately plain: one row per field, `>` marks the focused row,
/// the API key is masked, an error line and a key-hint line sit at the bottom.
pub fn render_provider_setup_form(
    content_area: Rect,
    buf: &mut Buffer,
    form: &ProviderSetupForm,
    compact: bool,
) {
    let theme = Theme::current();
    let label_width = 14u16;
    let rows = 5u16; // format + 4 text fields
    let hints_height = 2u16;
    let error_height = if form.error.is_some() { 1u16 } else { 0u16 };
    let total = rows + hints_height + error_height + 2; // + title + spacing

    let [_, centered_y, _] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(total),
        Constraint::Min(0),
    ])
    .areas(content_area);
    let width = if compact { 56u16 } else { 72u16 };
    let [_, form_area, _] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(width.min(centered_y.width)),
        Constraint::Min(0),
    ])
    .flex(Flex::Center)
    .areas(centered_y);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Configure API Provider",
        Style::default().fg(theme.accent_assistant),
    )));
    lines.push(Line::default());

    let label_style = Style::default().fg(theme.gray_bright);
    let value_style = Style::default().fg(theme.text_primary);
    let focused_marker = |field: ProviderSetupField| {
        if form.focused == field {
            Span::styled("> ", Style::default().fg(theme.accent_assistant))
        } else {
            Span::raw("  ")
        }
    };
    let pad_label = |label: &str| {
        Span::styled(
            format!("{label:<width$}", width = label_width as usize),
            label_style,
        )
    };

    // Format selector row.
    let mut format_spans = vec![focused_marker(ProviderSetupField::Format), pad_label("Format")];
    for (i, spec) in PROVIDER_FORMATS.iter().enumerate() {
        if i == form.format_index {
            format_spans.push(Span::styled(
                format!("[{}]", spec.label),
                Style::default().fg(theme.accent_assistant),
            ));
        } else {
            format_spans.push(Span::styled(format!(" {} ", spec.label), label_style));
        }
        format_spans.push(Span::raw("  "));
    }
    lines.push(Line::from(format_spans));

    let masked = |value: &str| {
        if value.len() <= 4 {
            "•".repeat(value.len())
        } else {
            format!("{}…{}", "•".repeat(value.len() - 4), &value[value.len() - 4..])
        }
    };
    let text_rows: [(ProviderSetupField, &str, String); 4] = [
        (ProviderSetupField::BaseUrl, "Base URL", form.base_url.clone()),
        (ProviderSetupField::ApiKey, "API key", masked(&form.api_key)),
        (ProviderSetupField::ModelId, "Model id", form.model_id.clone()),
        (
            ProviderSetupField::DisplayName,
            "Display name",
            form.display_name.clone(),
        ),
    ];
    for (field, label, value) in text_rows {
        let mut spans = vec![focused_marker(field), pad_label(label)];
        let mut rendered = value;
        if form.focused == field {
            rendered.push('▌');
        }
        spans.push(Span::styled(rendered, value_style));
        lines.push(Line::from(spans));
    }

    lines.push(Line::default());
    if let Some(error) = &form.error {
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(theme.accent_error),
        )));
    }
    let hint = if form.submitting {
        "Saving…".to_string()
    } else {
        "Tab/↓ next · Shift-Tab/↑ prev · ←/→ switch format · Enter save · Esc cancel".to_string()
    };
    lines.push(Line::from(Span::styled(
        hint,
        Style::default().fg(theme.gray_bright),
    )));

    Paragraph::new(lines).render(form_area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn format_switch_updates_preset_url_until_user_edits() {
        let mut form = ProviderSetupForm::new();
        assert_eq!(form.base_url, "https://api.openai.com/v1");
        form.handle_key(&key(KeyCode::Right));
        assert_eq!(form.selected_format().wire, "openai-compatible");
        form.handle_key(&key(KeyCode::Right));
        assert_eq!(form.selected_format().wire, "anthropic");
        assert_eq!(form.base_url, "https://api.anthropic.com/v1");

        // After the user edits the URL, format switches leave it alone.
        form.focused = ProviderSetupField::BaseUrl;
        form.handle_key(&key(KeyCode::Char('x')));
        form.focused = ProviderSetupField::Format;
        form.handle_key(&key(KeyCode::Right));
        assert!(form.base_url.ends_with('x'));
    }

    #[test]
    fn tab_cycles_focus_through_all_fields() {
        let mut form = ProviderSetupForm::new();
        assert_eq!(form.focused, ProviderSetupField::Format);
        form.handle_key(&key(KeyCode::Tab));
        assert_eq!(form.focused, ProviderSetupField::BaseUrl);
        form.handle_key(&key(KeyCode::Tab));
        assert_eq!(form.focused, ProviderSetupField::ApiKey);
        form.handle_key(&key(KeyCode::Tab));
        assert_eq!(form.focused, ProviderSetupField::ModelId);
        form.handle_key(&key(KeyCode::Tab));
        assert_eq!(form.focused, ProviderSetupField::DisplayName);
        form.handle_key(&key(KeyCode::Tab));
        assert_eq!(form.focused, ProviderSetupField::Format);
    }

    #[test]
    fn enter_validates_before_submitting() {
        let mut form = ProviderSetupForm::new();
        // Missing api_key / model_id / display_name → no submit.
        assert_eq!(form.handle_key(&key(KeyCode::Enter)), ProviderSetupOutcome::Changed);
        assert!(form.error.is_some());
        assert!(!form.submitting);

        form.api_key = "sk-test".into();
        form.model_id = "gpt-5".into();
        form.display_name = "OpenAI".into();
        assert_eq!(form.handle_key(&key(KeyCode::Enter)), ProviderSetupOutcome::Submit);
        assert!(form.submitting);
    }

    #[test]
    fn esc_cancels_and_typing_edits_focused_field() {
        let mut form = ProviderSetupForm::new();
        assert_eq!(form.handle_key(&key(KeyCode::Esc)), ProviderSetupOutcome::Cancel);

        form.focused = ProviderSetupField::ModelId;
        for c in "gpt-5".chars() {
            form.handle_key(&key(KeyCode::Char(c)));
        }
        assert_eq!(form.model_id, "gpt-5");
        form.handle_key(&key(KeyCode::Backspace));
        assert_eq!(form.model_id, "gpt-");
    }
}
