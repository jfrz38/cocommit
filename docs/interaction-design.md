# Interaction design

## Layout

The interface remains compact and needs no mouse support.

```text
┌─ cocommit ────────────────────────────────┐
│ Type       feat                           │
│ Scope                                     │
│ Breaking   [ ]                            │
│ Message                                   │
│ Issue                                     │
│ Sign (-S)  [ ]                            │
│                                            │
│               [ Commit ]                  │
├─ Preview ─────────────────────────────────┤
│ feat                                       │
├─ Status ──────────────────────────────────┤
│ Message is required                       │
├───────────────────────────────────────────┤
│ Up/Down Navigate  Ctrl+Enter Commit  Esc  │
└────────────────────────────────────────────┘
```

The focused row must have a visually distinct border, label, or background. The preview is rebuilt from the current draft after every edit. Status is empty until a validation or configuration warning needs to be shown.

The full bordered layout is used when space allows. Smaller terminals use compact one-line rows with automatic vertical scrolling that keeps the focused field visible. For terminals too small to safely draw even the compact layout, render only a clear resize instruction. Do not construct invalid Ratatui layout areas.

## Focus model

Focus advances in this order:

```text
Type -> Scope -> Breaking -> Message -> Issue -> Sign -> Submit
```

`Up` and `Down` move backward and forward through fields, respectively, and wrap. `Tab` and `Shift+Tab` provide the same forward and reverse navigation. Text input is active only for Type's popup search, Scope, Message, and Issue.

## Keyboard behavior

| Key | Form mode | Type picker mode |
|---|---|---|
| `Tab` | Next field | No action |
| `Shift+Tab` | Previous field | No action |
| `Up` / `Down` | Previous / next field | Change highlighted item |
| `Enter` | Open Type picker, advance text field, or submit from Commit | Select highlighted item |
| `Ctrl+Enter` | Submit from any field | No action |
| `F1` | Open or close help | Open or close help |
| `Space` | Toggle Breaking or Sign when focused | Insert a search space |
| `Esc` | Cancel application | Close picker and keep existing type |
| `Ctrl+C` | Cancel application | Cancel application |
| Printable text | Edit focused text field | Filter choices |
| Backspace/Delete/Home/End | Edit focused text field | Edit search query |
| Paste | Insert sanitized one-line text | Insert sanitized search query |

Vim `j` and `k` are intentionally not navigation shortcuts because they must remain searchable type characters.

## Type picker

Pressing `Enter` on Type opens a popup containing the standard types. The popup uses a case-insensitive substring match over its small list; fuzzy-search dependencies are not justified.

- Arrow keys change selection.
- A query filters the list.
- `Enter` selects the highlighted standard type.
- With an empty query, a `custom...` row is available.
- With a non-empty query not exactly equal to a standard type, show `Use "<query>"` as the custom row.
- Selecting a custom row writes the query into Type.
- `Esc` closes the popup without changing Type.
- Selection closes the popup and focuses Scope.

The custom value remains subject to normal Type validation on submission.

## Submission feedback

Commit is an explicit focusable action. There is no confirmation modal: navigating to it and pressing `Enter` already prevents accidental commits while keeping the flow short.

`Ctrl+Enter` submits from any form field without moving focus first. It follows the same validation path as Commit and is ignored while the Type picker is open.

`F1` opens a help popup without changing the current form or picker state. `Esc` closes the popup and restores the previous state.

On validation failure:

1. Keep the UI open.
2. Display a concise field-specific error in Status.
3. Move focus to the first invalid field.

On cancel, restore the terminal and return success without invoking Git. On valid submission, the UI returns the draft and signing choice to `main`; Git runs only after the terminal is restored.

## Event handling constraints

- Process only Crossterm `KeyEventKind::Press` to avoid duplicate actions on platforms that emit release events.
- Handle `Resize` by redrawing.
- Treat paste explicitly and remove or reject line breaks before they reach one-line fields.
- Ignore focus and mouse events in v1.
