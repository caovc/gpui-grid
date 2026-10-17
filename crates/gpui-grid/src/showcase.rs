//! A single Rust GPUI view shared by the macOS desktop and WebAssembly viewer.
//! The editor is a GPUI InputState. JavaScript never stores or modifies grid rows.
use crate::{CellAddress, DataType, Edit, GridDelegate, InlineEditor, InlineEditorSlot, parse_cell_value, sample_orders};
use gpui_kit::*;
use gpui_kit::base::StyledExt;
use gpui_kit::component::input::{Escape, InputEvent, InputState};
use gpui_kit::component::table::{DataTable, TableEvent, TableState};

gpui_kit::actions!(
    gpui_grid,
    [
        CommitEditorNext, CommitEditorPrevious,
        ExtendSelectionUp, ExtendSelectionDown, ExtendSelectionLeft, ExtendSelectionRight,
        SelectGridAll,
    ]
);

/// A feature-focused showcase still renders exactly the same reusable GPUI
/// DataTable and GridDelegate as the desktop application. The docs shell only
/// selects which Rust demo to launch; it has no table implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridFeature {
    Overview, Sorting, Filtering, Selection, Editing, Columns, Virtualization,
}
impl GridFeature {
    pub fn from_key(key: &str) -> Self {
        match key {
            "sorting" => Self::Sorting,
            "filtering" => Self::Filtering,
            "selection" => Self::Selection,
            "editing" => Self::Editing,
            "columns" => Self::Columns,
            "virtualization" => Self::Virtualization,
            _ => Self::Overview,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Sorting => "sorting",
            Self::Filtering => "filtering",
            Self::Selection => "selection",
            Self::Editing => "editing",
            Self::Columns => "columns",
            Self::Virtualization => "virtualization",
        }
    }
    pub fn row_count(self) -> usize {
        if self == Self::Virtualization { 50_000 } else { 12_480 }
    }
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Overview => "Rust GPUI • Desktop + WASM • Shift+Click / Shift+Arrows range • Cmd/Ctrl+A all • F2 edit",
            Self::Sorting => "Sorting · Use each GPUI column header menu for ascending, descending or multi-sort",
            Self::Filtering => "Filtering · Use the column header ⋯ menu and GPUI InputState to filter typed values",
            Self::Selection => "Selection · Shift+click or Shift+arrows; select every visible cell",
            Self::Editing => "Editing · Double-click or F2; Enter/Tab save; Esc cancels",
            Self::Columns => "Columns · Resize, reorder and hide/show from native GPUI header menus",
            Self::Virtualization => "Virtualization · 50,000 typed Rust rows; only the viewport is drawn",
        }
    }
}

// Read-only diagnostics for the Chromium smoke test. The actual button and
// editor handlers below publish counts after mutating the real Rust model.
#[cfg(target_family = "wasm")]
thread_local! {
    static PREVIEW_VISIBLE_ROWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static PREVIEW_ACTIVE_EDIT_ROW: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static PREVIEW_LAST_SAVED: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    static PREVIEW_SELECTED_CELLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static PREVIEW_FEATURE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    static PREVIEW_SORT_COLUMNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static PREVIEW_VISIBLE_COLUMNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn publish_feature(feature: GridFeature) {
    #[cfg(target_family = "wasm")]
    PREVIEW_FEATURE.with(|active| *active.borrow_mut() = feature.key().to_string());
    #[cfg(not(target_family = "wasm"))]
    let _ = feature;
}

#[cfg(target_family = "wasm")]
pub fn preview_feature() -> String {
    PREVIEW_FEATURE.with(|feature| feature.borrow().clone())
}

fn publish_sort_columns(count: usize) {
    #[cfg(target_family = "wasm")]
    PREVIEW_SORT_COLUMNS.with(|columns| columns.set(count));
    #[cfg(not(target_family = "wasm"))]
    let _ = count;
}

#[cfg(target_family = "wasm")]
pub fn preview_sort_columns() -> u32 {
    PREVIEW_SORT_COLUMNS.with(std::cell::Cell::get) as u32
}

fn publish_visible_columns(count: usize) {
    #[cfg(target_family = "wasm")]
    PREVIEW_VISIBLE_COLUMNS.with(|columns| columns.set(count));
    #[cfg(not(target_family = "wasm"))]
    let _ = count;
}

#[cfg(target_family = "wasm")]
pub fn preview_visible_columns() -> u32 {
    PREVIEW_VISIBLE_COLUMNS.with(std::cell::Cell::get) as u32
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

// Observability only; exported for Playwright so the browser verifies that
// pointer/IME events reached Rust. No JavaScript code can edit the grid model.
fn publish_active_edit(row_id: u64) {
    #[cfg(target_family = "wasm")]
    PREVIEW_ACTIVE_EDIT_ROW.with(|row| row.set(row_id));
    #[cfg(not(target_family = "wasm"))]
    let _ = row_id;
}

fn publish_saved_edit(address: &CellAddress, raw: &str) {
    #[cfg(target_family = "wasm")]
    PREVIEW_LAST_SAVED.with(|saved| *saved.borrow_mut() = format!("{}:{}:{}", address.row_id, address.column_key, raw));
    #[cfg(not(target_family = "wasm"))]
    let _ = (address, raw);
}

fn publish_selected_cells(count: usize) {
    #[cfg(target_family = "wasm")]
    PREVIEW_SELECTED_CELLS.with(|selected| selected.set(count));
    #[cfg(not(target_family = "wasm"))]
    let _ = count;
}

#[cfg(target_family = "wasm")]
pub fn preview_selected_cell_count() -> u32 {
    PREVIEW_SELECTED_CELLS.with(std::cell::Cell::get) as u32
}

#[cfg(target_family = "wasm")]
pub fn preview_active_editor_row_id() -> u32 {
    PREVIEW_ACTIVE_EDIT_ROW.with(std::cell::Cell::get) as u32
}

#[cfg(target_family = "wasm")]
pub fn preview_last_saved_edit() -> String {
    PREVIEW_LAST_SAVED.with(|saved| saved.borrow().clone())
}

struct EditSession {
    address: CellAddress,
    data_type: DataType,
    input: Entity<InputState>,
    error: Option<String>,
}

pub struct GridShowcase {
    feature: GridFeature,
    table: Entity<TableState<GridDelegate>>,
    inline_editor: InlineEditorSlot,
    selected: Option<CellAddress>,
    /// One-shot override: keyboard Shift+Arrow selection must retain its
    /// anchor even if modifier state changes before TableEvent delivery.
    keyboard_extend_to: Option<CellAddress>,
    editor: Option<EditSession>,
    feedback: Option<String>,
    _table_subscription: Subscription,
    _input_subscription: Option<Subscription>,
}

impl GridShowcase {
    /// Shared entry point for desktop and the browser's real GPUI WebPlatform.
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        Self::new_with_feature("overview", window, cx)
    }

    /// Start a live, feature-specific Rust/GPUI example for the docs site.
    /// This API is also usable in a native app; it is not WASM-only logic.
    pub fn new_with_feature(key: &str, window: &mut Window, cx: &mut App) -> Entity<Self> {
        let feature = GridFeature::from_key(key);
        // GPUI dispatches tab/shift-tab as key-bound actions before raw
        // KeyDown events (Root and DataTable already bind these keys).
        // This context exists only inside the active in-cell editor.
        cx.bind_keys([
            KeyBinding::new("tab", CommitEditorNext, Some("GPUIGridInlineEditor")),
            KeyBinding::new("shift-tab", CommitEditorPrevious, Some("GPUIGridInlineEditor")),
            KeyBinding::new("shift-up", ExtendSelectionUp, Some("DataTable")),
            KeyBinding::new("shift-down", ExtendSelectionDown, Some("DataTable")),
            KeyBinding::new("shift-left", ExtendSelectionLeft, Some("DataTable")),
            KeyBinding::new("shift-right", ExtendSelectionRight, Some("DataTable")),
            KeyBinding::new("secondary-a", SelectGridAll, Some("DataTable")),
        ]);
        let data = sample_orders(feature.row_count());
        publish_feature(feature);
        publish_visible_rows(data.len());
        publish_visible_columns(data.columns.len());
        publish_sort_columns(0);
        publish_selected_cells(0);
        cx.new(|cx| {
            let table = cx.new(|cx| {
                TableState::new(GridDelegate::new(data), window, cx)
                    .cell_selectable(true)
                    .row_selectable(true)
                    .col_resizable(true)
                    .col_movable(true)
            });
            let inline_editor = table.read(cx).delegate().inline_editor_slot();
            let subscription = cx.subscribe_in(&table, window, Self::on_table_event);
            Self {
                feature,
                table,
                inline_editor,
                selected: None,
                keyboard_extend_to: None,
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
                // TableState is emitting now: defer the model update rather
                // than borrowing the emitting state mutably a second time.
                let extend = if self.keyboard_extend_to.as_ref() == address.as_ref()
                    && address.is_some()
                {
                    self.keyboard_extend_to.take();
                    true
                } else {
                    self.keyboard_extend_to = None;
                    window.modifiers().shift
                };
                self.selected = address.clone();
                if let Some(address) = address {
                    let target = table.clone();
                    cx.defer(move |cx| {
                        let _ = target.update(cx, |state, cx| {
                            if state.delegate_mut().select_address(Some(address), extend).is_ok() {
                                publish_selected_cells(state.delegate().selected_cell_count());
                                state.refresh(cx);
                            }
                        });
                    });
                }
                cx.notify();
            }
            TableEvent::ClearSelection => {
                self.selected = None;
                self.keyboard_extend_to = None;
                let target = table.clone();
                cx.defer(move |cx| {
                    let _ = target.update(cx, |state, cx| {
                        if state.delegate_mut().select_address(None, false).is_ok() {
                            publish_selected_cells(0);
                            state.refresh(cx);
                        }
                    });
                });
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
        let info = selected.as_ref().and_then(|address| {
            self.table.read(cx).delegate().cell_info_by_address(address)
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
            *self.inline_editor.borrow_mut() = None;
            self._input_subscription = None;
            publish_active_edit(0);
            self.feedback = Some(format!("{} is read-only", info.address.column_key));
            cx.notify();
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx).default_value(info.value.text()));
        let subscription = cx.subscribe_in(&input, window, Self::on_input_event);
        *self.inline_editor.borrow_mut() = Some(InlineEditor {
            address: info.address.clone(),
            input: input.clone(),
        });
        self.editor = Some(EditSession {
            address: info.address,
            data_type: info.data_type,
            input: input.clone(),
            error: None,
        });
        self._input_subscription = Some(subscription);
        publish_active_edit(self.editor.as_ref().expect("just created editor").address.row_id);
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
            InputEvent::PressEnter { .. } => { self.commit_edit(cx); },
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
                publish_saved_edit(&address, &raw);
                publish_active_edit(0);
                self.feedback = Some(format!("Saved row {} · {}", address.row_id, address.column_key));
                *self.inline_editor.borrow_mut() = None;
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

    /// Tab action is dispatched to the editor's own key context before
    /// DataTable/Root take focus. Invalid edits keep the editor open.
    fn commit_and_advance(&mut self, backwards: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(address) = self.editor.as_ref().map(|session| session.address.clone()) else { return };
        self.commit_edit(cx);
        if self.editor.is_none() {
            let next = self.table.read(cx).delegate().next_editable_cell(&address, backwards);
            if let Some(info) = next {
                self.selected = Some(info.address.clone());
                self.start_edit(info, window, cx);
            }
        }
    }

    /// Extend from the current GPUI-selected cell, preserving a stable Rust
    /// anchor. The TableState mutation is deferred to avoid reentering the
    /// table (and its TableEvent listener) during the parent action callback.
    fn extend_by_arrow(&mut self, row_delta: isize, col_delta: isize, cx: &mut Context<Self>) {
        if self.editor.is_some() {
            cx.propagate();
            return;
        }
        let target = {
            let state = self.table.read(cx);
            let delegate = state.delegate();
            let rows = delegate.row_count();
            let cols = delegate.displayed_columns_count();
            if rows == 0 || cols == 0 {
                return;
            }
            let (row, col) = state.selected_cell().unwrap_or((0, 0));
            let row = (row as isize + row_delta).clamp(0, (rows - 1) as isize) as usize;
            let col = (col as isize + col_delta).clamp(0, (cols - 1) as isize) as usize;
            delegate.cell_info(row, col).map(|info| info.address)
        };
        if let Some(address) = target {
            self.keyboard_extend_to = Some(address.clone());
            let table = self.table.clone();
            cx.defer(move |cx| {
                let _ = table.update(cx, |state, cx| {
                    // Resolve the stable address again in case row order changed.
                    if let Some((row, col)) = state.delegate().display_position(&address) {
                        state.set_selected_cell(row, col, cx);
                    }
                });
            });
            cx.stop_propagation();
        }
    }

    /// Select only currently visible rows/columns; no cells are materialized.
    fn select_all_cells(&mut self, cx: &mut Context<Self>) {
        if self.editor.is_some() {
            cx.propagate();
            return;
        }
        let result = self.table.update(cx, |state, cx| {
            let result = state.delegate_mut().select_all_visible();
            if result.is_ok() {
                publish_selected_cells(state.delegate().selected_cell_count());
                state.refresh(cx);
            }
            result
        });
        if result.is_ok() {
            cx.stop_propagation();
        }
    }

    /// F2 is not bound by DataTable. Escape is intercepted as the input's
    /// Escape action in Render because raw keydown is consumed by GPUI.
    fn on_grid_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.is_held { return; }
        let key = event.keystroke.key.to_ascii_lowercase();
        match key.as_str() {
            "f2" if self.editor.is_none() => {
                self.begin_selected_edit(window, cx);
                cx.stop_propagation();
            }
            "escape" | "esc" if self.editor.is_some() => {
                self.cancel_edit(cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        publish_active_edit(0);
        *self.inline_editor.borrow_mut() = None;
        self.editor = None;
        self._input_subscription = None;
        self.feedback = Some("Edit canceled".into());
        cx.notify();
    }
}

impl Render for GridShowcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Feature-specific actions belong in reusable GPUI column/row/cell
        // controls. The showcase header only labels the example; it is not a
        // second toolbar duplicating grid operations.
        let mut content = div().size_full().flex().flex_col().bg(rgb(0xf7f9fc))
            // The GPUI InputState binds Escape as an action. Once the input
            // owns focus its action is dispatched instead of a raw KeyDown,
            // so the grid must capture the action before the input consumes it.
            .capture_action(cx.listener(|this, _: &Escape, _, cx| {
                if this.editor.is_some() {
                    this.cancel_edit(cx);
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            }))
            .capture_key_down(cx.listener(Self::on_grid_key_down))
            .on_action(cx.listener(|this, _: &CommitEditorNext, window, cx| {
                this.commit_and_advance(false, window, cx);
            }))
            .on_action(cx.listener(|this, _: &CommitEditorPrevious, window, cx| {
                this.commit_and_advance(true, window, cx);
            }))
            .on_action(cx.listener(|this, _: &ExtendSelectionUp, _, cx| {
                this.extend_by_arrow(-1, 0, cx);
            }))
            .on_action(cx.listener(|this, _: &ExtendSelectionDown, _, cx| {
                this.extend_by_arrow(1, 0, cx);
            }))
            .on_action(cx.listener(|this, _: &ExtendSelectionLeft, _, cx| {
                this.extend_by_arrow(0, -1, cx);
            }))
            .on_action(cx.listener(|this, _: &ExtendSelectionRight, _, cx| {
                this.extend_by_arrow(0, 1, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectGridAll, _, cx| {
                this.select_all_cells(cx);
            }))
            .child(
                div().flex().items_center().justify_between()
                    .px_6().py_4().bg(rgb(0xffffff))
                    .border_b_1().border_color(rgb(0xe4e9f1))
                    .child(
                        div().flex().flex_col().gap_1()
                            .child(div().text_xl().font_bold().text_color(rgb(0x273550)).child("GPUI Grid"))
                            .child(div().text_sm().text_color(rgb(0x7d899b)).child(self.feature.subtitle()))
                    )
             );

        if let Some(editor) = &self.editor {
            let heading = format!("Row {} · {}", editor.address.row_id, editor.address.column_key);
            let error = editor.error.clone();
            let mut panel = div().flex().items_center().gap_3().px_4().py_2()
                .bg(rgb(0xeff3ff)).border_b_1().border_color(rgb(0xdbe3f7))
                .child(div().text_sm().font_bold().text_color(rgb(0x425578)).child(heading))
                .child(div().text_sm().text_color(rgb(0x61718a))
                    .child("Editing in cell · Enter/Tab save · Esc cancel"));
            if let Some(error) = error {
                panel = panel.child(div().text_sm().text_color(rgb(0xb83248)).child(error));
            }
            content = content.child(panel);
        } else if let Some(feedback) = &self.feedback {
            content = content.child(div().px_4().py_1().text_sm().text_color(rgb(0x556987)).child(feedback.clone()));
        }

        content.child(div().flex_1().min_h_0().p_4()
            .child(DataTable::new(&self.table).bordered(true).stripe(true)))
    }
}
