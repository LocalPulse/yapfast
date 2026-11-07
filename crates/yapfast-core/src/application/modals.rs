//! Discord apps' menus on messages, and the forms (modals) apps open in
//! answer to a press: filling them in, checking them as Discord's client
//! does, and sending the answer.

use super::*;
use crate::chat::{CardMenu, CardRow, ChatModal, ModalAnswer, ModalInputKind};

/// A Discord app's form while it's open: the form, what's filled in each
/// field so far, and whether the answer is on its way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalForm {
    pub modal: ChatModal,
    /// One for each of the form's fields, in order.
    pub answers: Vec<ModalAnswer>,
    pub sending: bool,
    pub error: Option<String>,
}

impl ModalForm {
    pub fn new(modal: ChatModal) -> Self {
        let answers = modal
            .fields
            .iter()
            .map(|field| match &field.kind {
                ModalInputKind::Text { value, .. } => ModalAnswer::Text(value.clone()),
                ModalInputKind::Select { options, .. } => ModalAnswer::Picked(
                    options
                        .iter()
                        .filter(|option| option.default)
                        .map(|option| option.value.clone())
                        .collect(),
                ),
                ModalInputKind::Note(_) => ModalAnswer::None,
            })
            .collect();
        Self {
            modal,
            answers,
            sending: false,
            error: None,
        }
    }

    /// Why a field's answer can't go yet, as Discord's client words it;
    /// none when it can.
    pub fn problem(&self, field: usize) -> Option<String> {
        let input = self.modal.fields.get(field)?;
        match (&input.kind, self.answers.get(field)?) {
            (
                ModalInputKind::Text {
                    min_length,
                    max_length,
                    required,
                    ..
                },
                ModalAnswer::Text(text),
            ) => {
                let length = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
                if text.trim().is_empty() {
                    return required.then(|| "This field is required.".into());
                }
                if length < *min_length {
                    return Some(format!("Must be at least {min_length} characters long."));
                }
                (length > *max_length)
                    .then(|| format!("Must be {max_length} or fewer characters long."))
            }
            (
                ModalInputKind::Select {
                    min_values,
                    required,
                    ..
                },
                ModalAnswer::Picked(values),
            ) => {
                let count = u32::try_from(values.len()).unwrap_or(u32::MAX);
                if values.is_empty() {
                    return (*required && *min_values > 0)
                        .then(|| "This field is required.".into());
                }
                (count < *min_values).then(|| format!("Pick at least {min_values}."))
            }
            _ => None,
        }
    }

    /// Every field can go.
    pub fn complete(&self) -> bool {
        (0..self.modal.fields.len()).all(|field| self.problem(field).is_none())
    }
}

/// A Discord app's menu on a message, open: which, and what's picked so
/// far in one of several.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenMenu {
    pub item: String,
    pub custom_id: String,
    pub picked: Vec<String>,
}

impl Application {
    /// The menu `custom_id` of `item`, if it opens.
    fn menu(&self, item: &str, custom_id: &str) -> Option<&CardMenu> {
        self.state
            .message(item)?
            .cards
            .iter()
            .flat_map(|card| &card.rows)
            .find_map(|row| match row {
                CardRow::Select {
                    disabled: false,
                    menu: Some(menu),
                    ..
                } if menu.custom_id == custom_id => Some(menu),
                _ => None,
            })
    }

    pub(super) fn open_menu(&mut self, item: String, custom_id: String) -> Vec<Effect> {
        let Some(menu) = self.menu(&item, &custom_id) else {
            return vec![];
        };
        // It starts from the options the app picked by default.
        let picked = menu
            .options
            .iter()
            .filter(|option| option.default)
            .map(|option| option.value.clone())
            .collect();
        self.state.open_menu = Some(OpenMenu {
            item,
            custom_id,
            picked,
        });
        vec![]
    }

    pub(super) fn menu_pick(&mut self, value: String) -> Vec<Effect> {
        let Some(open) = self.state.open_menu.clone() else {
            return vec![];
        };
        let Some(menu) = self.menu(&open.item, &open.custom_id) else {
            self.state.open_menu = None;
            return vec![];
        };
        if !menu.options.iter().any(|option| option.value == value) {
            return vec![];
        }
        if menu.max_values <= 1 {
            self.state.open_menu = None;
            return self.pick_options(open.item, open.custom_id, vec![value]);
        }
        let max = usize::try_from(menu.max_values).unwrap_or(usize::MAX);
        if let Some(open) = &mut self.state.open_menu {
            if let Some(index) = open.picked.iter().position(|each| *each == value) {
                open.picked.remove(index);
            } else if open.picked.len() < max {
                open.picked.push(value);
            }
        }
        vec![]
    }

    pub(super) fn menu_done(&mut self) -> Vec<Effect> {
        let Some(open) = self.state.open_menu.take() else {
            return vec![];
        };
        self.pick_options(open.item, open.custom_id, open.picked)
    }

    pub(super) fn pick_options(
        &mut self,
        item: String,
        custom_id: String,
        values: Vec<String>,
    ) -> Vec<Effect> {
        let Some(room) = self.state.room_of(&item).cloned() else {
            return vec![];
        };
        if self.state.preview {
            return vec![];
        }
        let fits = |menu: &CardMenu| {
            let count = u32::try_from(values.len()).unwrap_or(u32::MAX);
            (menu.min_values..=menu.max_values).contains(&count)
                && values
                    .iter()
                    .all(|value| menu.options.iter().any(|option| option.value == *value))
        };
        let pickable = self.state.message(&item).is_some_and(|message| {
            message.cards.iter().flat_map(|card| &card.rows).any(|row| {
                matches!(
                    row,
                    CardRow::Select { disabled: false, menu: Some(menu), .. }
                        if menu.custom_id == custom_id && fits(menu)
                )
            })
        });
        if !pickable {
            return vec![];
        }
        vec![Effect::Backend(Command::PickOptions {
            room,
            item,
            custom_id,
            values,
        })]
    }

    pub(super) fn modal_text(&mut self, field: usize, text: String) -> Vec<Effect> {
        if let Some(form) = &mut self.state.modal
            && let Some(ModalAnswer::Text(answer)) = form.answers.get_mut(field)
        {
            *answer = text;
            form.error = None;
        }
        vec![]
    }

    pub(super) fn modal_pick(&mut self, field: usize, value: String) -> Vec<Effect> {
        let Some(form) = &mut self.state.modal else {
            return vec![];
        };
        let Some(ModalInputKind::Select {
            options,
            max_values,
            ..
        }) = form.modal.fields.get(field).map(|input| &input.kind)
        else {
            return vec![];
        };
        if !options.iter().any(|option| option.value == value) {
            return vec![];
        }
        let max = usize::try_from(*max_values).unwrap_or(usize::MAX);
        if let Some(ModalAnswer::Picked(picked)) = form.answers.get_mut(field) {
            if let Some(index) = picked.iter().position(|each| *each == value) {
                picked.remove(index);
            } else if max <= 1 {
                *picked = vec![value];
            } else if picked.len() < max {
                picked.push(value);
            }
            form.error = None;
        }
        vec![]
    }

    pub(super) fn submit_modal(&mut self) -> Vec<Effect> {
        let Some(form) = &mut self.state.modal else {
            return vec![];
        };
        if form.sending || !form.complete() {
            return vec![];
        }
        form.sending = true;
        form.error = None;
        // Text goes as typed, but without the spaces around it, as
        // Discord's client sends it.
        let answers = form
            .answers
            .iter()
            .map(|answer| match answer {
                ModalAnswer::Text(text) => ModalAnswer::Text(text.trim().to_owned()),
                other => other.clone(),
            })
            .collect();
        vec![Effect::Backend(Command::Discord(
            DiscordCommand::SubmitModal {
                id: form.modal.id.clone(),
                answers,
            },
        ))]
    }

    pub(super) fn modal_submitted(&mut self, result: Result<(), String>) {
        let Some(form) = &mut self.state.modal else {
            return;
        };
        match result {
            Ok(()) => self.state.modal = None,
            Err(error) => {
                form.sending = false;
                form.error = Some(error);
            }
        }
    }
}
