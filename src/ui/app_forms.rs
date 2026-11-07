//! Discord apps' forms (modals) and the menus on their messages, as the
//! window shows them: `AppModalDialog` and `AppMenuPopup` in
//! ui/app_modal.slint.

use yapfast_core::chat::{CardOption, CardRow, ModalAnswer, ModalInputKind};

use super::*;

/// A menu's or a form menu's options as the window lists them, `picked`
/// those chosen so far.
fn option_rows(options: &[CardOption], picked: &[String]) -> Vec<ModalOptionRow> {
    options
        .iter()
        .map(|option| ModalOptionRow {
            label: option.label.as_str().into(),
            value: option.value.as_str().into(),
            description: option.description.as_deref().unwrap_or_default().into(),
            picked: picked.contains(&option.value),
        })
        .collect()
}

impl Adapter {
    pub(super) fn show_app_modal(&self, window: &YapfastWindow) {
        let form = self.app.state.modal.as_ref();
        window.set_app_modal_open(form.is_some());
        let Some(form) = form else {
            if self.models.app_modal_fields.row_count() > 0 {
                self.models.app_modal_fields.set_vec(Vec::new());
            }
            return;
        };
        window.set_app_modal_title(form.modal.title.as_str().into());
        window.set_app_modal_can_submit(form.complete());
        window.set_app_modal_sending(form.sending);
        window.set_app_modal_problem(form.error.as_deref().unwrap_or_default().into());
        let rows = form
            .modal
            .fields
            .iter()
            .zip(&form.answers)
            .enumerate()
            .map(|(index, (field, answer))| {
                let problem = form.problem(index).unwrap_or_default();
                let mut row = ModalFieldRow {
                    label: field.label.as_str().into(),
                    description: field.description.as_deref().unwrap_or_default().into(),
                    problem: problem.into(),
                    ..Default::default()
                };
                match (&field.kind, answer) {
                    (
                        ModalInputKind::Text {
                            paragraph,
                            placeholder,
                            max_length,
                            required,
                            ..
                        },
                        ModalAnswer::Text(text),
                    ) => {
                        row.kind = if *paragraph {
                            ModalFieldKind::Paragraph
                        } else {
                            ModalFieldKind::Text
                        };
                        row.placeholder = placeholder.as_deref().unwrap_or_default().into();
                        row.value = text.as_str().into();
                        row.length = i32::try_from(text.chars().count()).unwrap_or(i32::MAX);
                        row.max_length = i32::try_from(*max_length).unwrap_or(i32::MAX);
                        row.required = *required;
                    }
                    (
                        ModalInputKind::Select {
                            options,
                            placeholder,
                            max_values,
                            required,
                            ..
                        },
                        ModalAnswer::Picked(picked),
                    ) => {
                        row.kind = ModalFieldKind::Select;
                        row.placeholder = placeholder.as_deref().unwrap_or_default().into();
                        row.options = ModelRc::new(VecModel::from(option_rows(options, picked)));
                        row.multi = *max_values > 1;
                        row.required = *required;
                    }
                    (ModalInputKind::Note(text), _) => {
                        row.kind = ModalFieldKind::Note;
                        row.value = text.as_str().into();
                    }
                    _ => {}
                }
                row
            })
            .collect();
        // By place: a field keeps its row, and the text typed in it, while
        // the rest of the form changes.
        update_rows_in_place(&self.models.app_modal_fields, rows);
    }

    pub(super) fn show_app_menu(&self, window: &YapfastWindow) {
        let state = &self.app.state;
        let menu = state.open_menu.as_ref().and_then(|open| {
            let message = state.message(&open.item)?;
            let menu =
                message
                    .cards
                    .iter()
                    .flat_map(|card| &card.rows)
                    .find_map(|row| match row {
                        CardRow::Select {
                            menu: Some(menu), ..
                        } if menu.custom_id == open.custom_id => Some(menu),
                        _ => None,
                    })?;
            Some((open, menu))
        });
        window.set_app_menu_open(menu.is_some());
        let Some((open, menu)) = menu else {
            if self.models.app_menu_options.row_count() > 0 {
                self.models.app_menu_options.set_vec(Vec::new());
            }
            return;
        };
        window.set_app_menu_multi(menu.max_values > 1);
        window.set_app_menu_min(i32::try_from(menu.min_values).unwrap_or(i32::MAX));
        window.set_app_menu_max(i32::try_from(menu.max_values).unwrap_or(i32::MAX));
        window.set_app_menu_picked_count(i32::try_from(open.picked.len()).unwrap_or(i32::MAX));
        update_rows_in_place(
            &self.models.app_menu_options,
            option_rows(&menu.options, &open.picked),
        );
    }
}

/// Sets `model` to `rows` place by place, so rows that stay keep their
/// items in the window.
fn update_rows_in_place<T: Clone + PartialEq + 'static>(model: &VecModel<T>, rows: Vec<T>) {
    while model.row_count() > rows.len() {
        model.remove(model.row_count() - 1);
    }
    for (index, row) in rows.into_iter().enumerate() {
        if index >= model.row_count() {
            model.push(row);
        } else if model.row_data(index).as_ref() != Some(&row) {
            model.set_row_data(index, row);
        }
    }
}

pub(super) fn bind(window: &YapfastWindow, adapter: &Rc<RefCell<Adapter>>) {
    macro_rules! act {
        ($callback:ident($($arg:ident),*) => $action:expr) => {{
            let state = adapter.clone();
            let weak = window.as_weak();
            window.$callback(move |$($arg),*| {
                if let Some(window) = weak.upgrade() {
                    state.borrow_mut().action(&window, $action);
                }
            });
        }};
    }
    let index = |index: i32| usize::try_from(index).unwrap_or(usize::MAX);
    act!(on_app_modal_text(at, text) => Action::ModalText {
        field: index(at),
        text: text.into(),
    });
    act!(on_app_modal_pick(at, value) => Action::ModalPick {
        field: index(at),
        value: value.into(),
    });
    act!(on_app_modal_submit() => Action::SubmitModal);
    act!(on_app_modal_close() => Action::CloseModal);
    act!(on_open_app_menu(item, custom_id) => Action::OpenMenu {
        item: item.into(),
        custom_id: custom_id.into(),
    });
    act!(on_app_menu_pick(value) => Action::MenuPick(value.into()));
    act!(on_app_menu_done() => Action::MenuDone);
    act!(on_app_menu_close() => Action::CloseMenu);
}
