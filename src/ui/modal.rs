use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use tui_input::Input;

use crate::app::{
    App, FooterEditorFocus, FooterEditorState, FooterNamePickerState, Mode, TypeChoice,
    TypePickerState,
};

use super::{
    components::{focused_style, render_action_row, render_text_row, type_picker_title},
    help,
    preview::{clamp_scroll, render_preview_expanded},
};

pub(super) fn render(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    mut cursor: Option<Position>,
) -> (Option<Position>, Option<u16>) {
    if let Mode::TypePicker(picker) = app.mode() {
        cursor = render_picker(frame, area, app, picker);
    }
    if let Mode::ScopePicker(picker) = app.mode() {
        cursor = render_scope_picker(frame, area, app, picker);
    }
    if let Mode::FooterNamePicker(picker) = app.mode() {
        cursor = render_footer_name_picker(frame, area, picker);
    }
    if let Mode::FooterEditor(editor) = app.mode() {
        cursor = render_footer_editor(frame, area, editor);
    }
    let expanded_preview_limit = if matches!(app.mode(), Mode::PreviewExpanded) {
        Some(render_preview_expanded(
            frame,
            area,
            &app.preview(),
            app.preview_state().scroll,
        ))
    } else {
        None
    };
    if expanded_preview_limit.is_some() {
        cursor = None;
    }
    if matches!(app.mode(), Mode::Help(_)) {
        help::render(frame, area, app);
        cursor = None;
    }
    (cursor, expanded_preview_limit)
}

fn render_picker(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    picker: &TypePickerState,
) -> Option<Position> {
    render_choice_picker(
        frame,
        area,
        type_picker_title(app.types_are_restricted()),
        &picker.query,
        picker.highlighted,
        app.type_choices(picker),
    )
}

fn render_scope_picker(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    picker: &TypePickerState,
) -> Option<Position> {
    render_choice_picker(
        frame,
        area,
        " Select scope suggestion ",
        &picker.query,
        picker.highlighted,
        app.scope_choices(picker),
    )
}

fn render_footer_name_picker(
    frame: &mut Frame,
    area: Rect,
    picker: &FooterNamePickerState,
) -> Option<Position> {
    render_choice_picker(
        frame,
        area,
        " Select footer name ",
        &picker.query,
        picker.highlighted,
        picker.choices(),
    )
}

fn render_choice_picker(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    query: &Input,
    highlighted: usize,
    choices: Vec<TypeChoice>,
) -> Option<Position> {
    let width = area.width.saturating_sub(4).min(60);
    let height = area.height.saturating_sub(2).min(18);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(focused_style(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let width = usize::from(chunks[0].width.max(1));
    let scroll = query.visual_scroll(width);
    frame.render_widget(
        Paragraph::new(query.to_string()).scroll((0, clamp_scroll(scroll))),
        chunks[0],
    );

    let items = choices
        .iter()
        .map(|choice| ListItem::new(choice_label(choice)))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select((!choices.is_empty()).then_some(highlighted));
    frame.render_stateful_widget(
        List::new(items).highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[1],
        &mut state,
    );

    let column = query
        .visual_cursor()
        .saturating_sub(scroll)
        .min(width.saturating_sub(1)) as u16;
    Some(Position::new(chunks[0].x + column, chunks[0].y))
}

fn render_footer_editor(
    frame: &mut Frame,
    area: Rect,
    editor: &FooterEditorState,
) -> Option<Position> {
    let width = area.width.saturating_sub(4).min(70);
    let height = area.height.saturating_sub(2).min(18);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Footer editor ")
        .border_style(focused_style(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);
    let mut cursor = render_text_row(
        frame,
        rows[0],
        "Footer name",
        &editor.token,
        editor.focus == FooterEditorFocus::Name,
    );
    cursor = cursor.or(render_text_row(
        frame,
        rows[1],
        "Value (Enter newline)",
        &editor.value,
        editor.focus == FooterEditorFocus::Value,
    ));
    render_action_row(
        frame,
        rows[2],
        "Save footer",
        editor.focus == FooterEditorFocus::Save,
    );
    cursor
}

fn choice_label(choice: &TypeChoice) -> String {
    match choice {
        TypeChoice::Standard(value) => value.clone(),
        TypeChoice::CustomQuery(value) => format!("Use \"{value}\""),
    }
}
