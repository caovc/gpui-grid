//! A single Rust GPUI view shared by the macOS desktop and WebAssembly viewer.
//! The editor is a GPUI InputState. JavaScript never stores or modifies grid rows.
use crate::{CellAddress, DataType, Edit, GridDelegate, parse_cell_value, sample_orders};
use gpui_kit::*;
use gpui_kit::base::StyledExt;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::table::{DataTable, TableEvent, TableState};

// Read-only diagnostics for the Chromium smoke test. The actual button and
// editor handlers below publish counts after mutating the real Rust model.
#[cfg(target_family = "wasm")]
thread_local! {
    static PREVIEW_VISIBLE_ROWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn publish_visible_rows(count: usize) {
    #[cfg(target_family = "wasm")]
    PREVIEW_VISIBLE_ROWS.with(|rows| rows.set(count));
    #[cfg(not(target_family = "wasm"))]
    let _ = count;
}

#[cfg(target_family = "wasm")]
pub fn preview_visible_rows() -> usize {
    PREVIEW_VISIBLE_ROWS.with(std::cell::Cell::get)
}

struct EditSession {
    address: CellAddress,
    data_type: DataType,
    input: Entity<InputState>,
    error: Option<String>,
}

pub struct GridShowcase {
    table: Entity<TableState<GridDelegate>>,
    selected: Option<CellAddress>,
    editor: Option<EditSession>,
    feedback: Option<String>,
    _table_subscription: Subscription,
    _input_subscription: Option<Subscription>,
}

impl GridShowcase {
    /// Shared entry point for desktop and the browser's real GPUI WebPlatform.
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let data = sample_orders(12_480);
        publish_visible_rows(data.len());
        cx.new(|cx| {
            let table = cx.new(|cx| {
                TableState::new(GridDelegate::new(data), window, cx)
                    .cell_selectable(true)
                    .row_selectable(true)
                    .col_resizable(true)
                    .col_movable(true)
            });
            let subscription = cx.subscribe_in(&table, window, Self::on_table_event);
            Self {
                table,
                selected: None,
                editor: None,
                feedback: None,
                _table_subscription: subscription,
                _input_subscription: None,
            }
        })
    }

    fn on_table_event(
        &mut self,
        table: &Entity<TableState<GridDelegate>>,
        event: &TableEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TableEvent::SelectCell(row_ix, col_ix) => {
                let address = table.read(cx).delegate().cell_info(*row_ix, *col_ix).map(|info| info.address);
                // Avoid re-entering TableState while it is emitting its event.
                // The edit session always keeps the stable row/column identity.
                self.selected = address;
                cx.notify();
            }
            TableEvent::DoubleClickedCell(row_ix, col_ix) => {
                self.begin_cell_edit(*row_ix, *col_ix, window, cx);
            }
            _ => {}
        }
    }

    fn begin_selected_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected = self.selected.clone();
        let info = selected.as_ref().and_then(|selected| {
            let table = self.table.read(cx);
            let delegate = table.delegate();
            let row_ix = (0..delegate.row_count()).find(|&ix| {
                delegate.visible_row(ix).is_some_and(|row| row.id == selected.row_id)
            })?;
            let col_ix = (0..delegate.model.state.visible_column_keys().count())
                .find(|&ix| delegate.cell_info(row_ix, ix).is_some_and(|cell| cell.address.column_key == selected.column_key))?;
            delegate.cell_info(row_ix, col_ix)
        });
        if let Some(info) = info {
            self.start_edit(info, window, cx);
        } else {
            self.feedback = Some("Select a visible cell before editing.".into());
            cx.notify();
        }
    }

    fn begin_cell_edit(&mut self, row_ix: usize, col_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let info = self.table.read(cx).delegate().cell_info(row_ix, col_ix);
        if let Some(info) = info {
            self.selected = Some(info.address.clone());
            self.start_edit(info, window, cx);
        }
    }

    fn start_edit(&mut self, info: crate::CellInfo, window: &mut Window, cx: &mut Context<Self>) {
        if !info.editable {
            self.editor = None;
            self._input_subscription = None;
            self.feedback = Some(format!("{} is read-only", info.address.column_key));
            cx.notify();
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx).default_value(info.value.text()));
        let subscription = cx.subscribe_in(&input, window, Self::on_input_event);
        self.editor = Some(EditSession {
            address: info.address,
            data_type: info.data_type,
            input: input.clone(),
            error: None,
        });
        self._input_subscription = Some(subscription);
        self.feedback = None;
        cx.notify();
        input.focus_handle(cx).focus(window, cx);
    }

    fn on_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.editor.as_ref().is_none_or(|editor| &editor.input != state) { return; }
        match event {
            InputEvent::PressEnter { .. } => self.commit_edit(cx),
            InputEvent::Change => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.error = None;
                }
                cx.notify();
            }
            // Deliberately do not commit on Blur: input IME and validation may
            // be mid-composition. The explicit Save/Cancel path is deterministic.
            InputEvent::Blur | InputEvent::Focus => {}
        }
    }

    fn set_boolean_input(&mut self, value: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(editor) = self.editor.as_ref() {
            let input = editor.input.clone();
            input.update(cx, |state, cx| {
                state.set_value(if value { "true" } else { "false" }.into(), window, cx);
            });
        }
    }

    fn commit_edit(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.editor.as_ref() else { return };
        let address = session.address.clone();
        let raw = session.input.read(cx).value().to_string();
        let value = match parse_cell_value(session.data_type, &raw) {
            Ok(value) => value,
            Err(error) => {
                if let Some(editor) = self.editor.as_mut() { editor.error = Some(error.into()); }
                cx.notify();
                return;
            }
        };
        let result = self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            let result = delegate.edit(&[Edit {
                row_id: address.row_id,
                column: address.column_key.clone(),
                value,
            }]);
            if result.is_ok() {
                publish_visible_rows(delegate.row_count());
                table.refresh(cx);
            }
            result
        });
        match result {
            Ok(_) => {
                self.feedback = Some(format!("Saved row {} · {}", address.row_id, address.column_key));
                self.editor = None;
                self._input_subscription = None;
            }
            Err(error) => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.error = Some(error.to_string());
                }
            }
        }
        cx.notify();
    }

    fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.editor = None;
        self._input_subscription = None;
        self.feedback = Some("Edit canceled".into());
        cx.notify();
    }
}

impl Render for GridShowcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let only_completed = self.table.clone();
        let show_all = self.table.clone();
        let mut content = div().size_full().flex().flex_col().bg(rgb(0xf7f9fc))
            .child(
                div().flex().items_center().justify_between()
                    .px_6().py_4().bg(rgb(0xffffff))
                    .border_b_1().border_color(rgb(0xe4e9f1))
                    .child(
                        div().flex().flex_col().gap_1()
                            .child(div().text_xl().font_bold().text_color(rgb(0x273550)).child("GPUI Grid"))
                            .child(div().text_sm().text_color(rgb(0x7d899b)).child("Rust GPUI • WebAssembly + Desktop • Double-click a cell to edit"))
                    )
                    .child(
                        div().flex().gap_2()
                            .child(Button::new("edit-selected").label("Edit cell").on_click(cx.listener(|this, _, window, cx| {
                                this.begin_selected_edit(window, cx);
                            })))
                            .child(Button::new("completed-only").label("Completed only").on_click(move |_, _, cx| {
                                only_completed.update(cx, |table, cx| {
                                    let delegate = table.delegate_mut();
                                    delegate.set_quick_filter("Completed");
                                    publish_visible_rows(delegate.row_count());
                                    table.refresh(cx);
                                });
                            }))
                            .child(Button::new("all-rows").label("All rows").on_click(move |_, _, cx| {
                                show_all.update(cx, |table, cx| {
                                    let delegate = table.delegate_mut();
                                    delegate.set_quick_filter("");
                                    publish_visible_rows(delegate.row_count());
                                    table.refresh(cx);
                                });
                            }))
                    )
            );

        if let Some(editor) = &self.editor {
            let heading = format!("Row {} · {}", editor.address.row_id, editor.address.column_key);
            let input = editor.input.clone();
            let kind = editor.data_type;
            let error = editor.error.clone();
            let mut panel = div().flex().items_center().gap_3().px_4().py_2()
                .bg(rgb(0xeff3ff)).border_b_1().border_color(rgb(0xdbe3f7))
                .child(div().text_sm().font_bold().text_color(rgb(0x425578)).child(heading))
                .child(Input::new(&input).w(px(260.0)).aria_label("Edit grid cell"));
            if kind == DataType::Boolean {
                panel = panel
                    .child(Button::new("editor-true").label("True").on_click(cx.listener(|this, _, window, cx| this.set_boolean_input(true, window, cx))))
                    .child(Button::new("editor-false").label("False").on_click(cx.listener(|this, _, window, cx| this.set_boolean_input(false, window, cx))));
            }
            if let Some(error) = error {
                panel = panel.child(div().text_sm().text_color(rgb(0xb83248)).child(error));
            }
            content = content.child(panel
                .child(Button::new("editor-save").label("Save").on_click(cx.listener(|this, _, _, cx| this.commit_edit(cx))))
                .child(Button::new("editor-cancel").label("Cancel").on_click(cx.listener(|this, _, _, cx| this.cancel_edit(cx)))));
        } else if let Some(feedback) = &self.feedback {
            content = content.child(div().px_4().py_1().text_sm().text_color(rgb(0x556987)).child(feedback.clone()));
        }

        content.child(div().flex_1().min_h_0().p_4()
            .child(DataTable::new(&self.table).bordered(true).stripe(true)))
    }
}
