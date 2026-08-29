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
│ Tab Next  Shift+Tab Previous  Esc Cancel  │
└────────────────────────────────────────────┘
```

The focused row must have a visually distinct border, label, or background. The preview is rebuilt from the current draft after every edit. Status is empty until a validation or configuration warning needs to be shown.

For terminals too small to safely draw the layout, render only a clear resize instruction. Do not construct invalid Ratatui layout areas.

## Focus model

Focus advances in this order:

```text
Type -> Scope -> Breaking -> Message -> Issue -> Sign -> Submit
```

`Tab` moves forward and wraps. `Shift+Tab` moves backward and wraps. Text input is active only for Type's popup search, Scope, Message, and Issue.

## Keyboard behavior

| Key | Form mode | Type picker mode |
|---|---|---|
| `Tab` | Next field | No action |
| `Shift+Tab` | Previous field | No action |
| `Enter` | Open Type picker, advance text field, or submit from Commit | Select highlighted item |
| `Space` | Toggle Breaking or Sign when focused | Insert a search space |
| `Esc` | Cancel application | Close picker and keep existing type |
| `Ctrl+C` | Cancel application | Cancel application |
| `Up` / `Down` | No action | Change highlighted item |
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
