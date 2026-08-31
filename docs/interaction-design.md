# Interaction design

## Layout

The interface remains compact and needs no mouse support.

```text
┌─ cocommit ────────────────────────────────┐
│ Type       feat                           │
│ Scope                                     │
│ Breaking   [ ]                            │
│ Subject 0/72; lowercase; no punctuation   │
│ Body                                      │
│ Footers                                   │
│ Issue                                     │
│ Sign commit [ ]                            │
│                                            │
│ Staged changes                              │
│ 3 files  A:1 M:1 D:0 R:1  +12 -4           │
│ > M src/app.rs                             │
│ R docs/guide.md -> docs/usage.md           │
│               [ Commit ]                   │
├─ Preview ─────────────────────────────────┤
│ feat: add endpoint                         │
│                                             │
│ Details                                    │
│                                             │
│ Closes: #42                                │
├─ Error ───────────────────────────────────┤
│ Message is required                       │
├───────────────────────────────────────────┤
│ Up/Down Navigate  Ctrl+Enter Commit  Esc  │
└────────────────────────────────────────────┘
```

The focused row must have a visually distinct border, label, or background. The Subject label shows the character count and any configured subject-length, capitalization, or terminal-punctuation convention as non-blocking guidance. The preview is rebuilt from the current draft after every edit. Its title shows the visible starting line and total line count; when content is outside its viewport it advertises `Enter expand`. The status block is hidden until feedback is needed, uses red for errors, and green for successful index operations.

The full bordered layout is used when space allows. Smaller terminals use compact one-line rows with automatic vertical scrolling that keeps the focused field visible. Configured hidden optional sections consume no row, border, summary, or placeholder in either layout. For terminals too small to safely draw even the compact layout, render only a clear resize instruction. Do not construct invalid Ratatui layout areas.

## Configurable sections

Global `ui.sections` preferences independently control Body, Footers, Issue, and Staged changes. All are visible by default. Type, Scope, Breaking, Subject, Sign commit, Preview, and Commit are always visible. Repository configuration cannot change this preference.

A hidden optional section has no focus target, help entry, key hint, action, picker, or modal. Hidden Body, Footers, and Issue are empty and cannot receive validation focus. Hiding Staged changes retains preflight but removes the staged summary, selection, inclusion checkboxes, and unstage behavior; the resulting commit uses all files in the Git index at submission.

## Focus model

Focus candidates are ordered as follows and filtered to visible sections:

```text
Type -> Scope -> Breaking -> Message -> Body -> Footers -> Issue -> Sign -> Staged changes -> Preview -> Submit
```

`Up` and `Down` move backward and forward through visible fields, respectively, and wrap. When Footers or Staged changes is visible and focused, they select a visibly highlighted item and leave the section at its boundaries. Preview scrolls with these keys while content remains above or below it; at a boundary it moves to the preceding or following visible field. `Home`/`End` jump within its real scroll bounds. `Enter` opens an expanded preview from Preview. `Tab` and `Shift+Tab` always move focus. Typing or pasting while Type is focused opens its popup search. Text input is otherwise active for visible Scope, Message, Body, and Issue.

## Expanded preview

The expanded preview is a read-only overlay over the current form. It displays the exact canonical message, including body and ordered footers. `Up`/`Down` scroll, `Home`/`End` jump to its bounds, and `Enter` or `Esc` closes it without losing form state. `F1` opens help while preserving the overlay and scroll position. `Ctrl+Enter` follows normal validation and submission behavior.

## Body and footers

When Body is visible, it accepts multiline text and preserves paragraph breaks. Its cursor and viewport follow the active line. When Footers is visible, footers are shown in order and keep the selected entry visible. `A` opens a footer-name picker containing `BREAKING CHANGE`, `Closes`, `Fixes`, `Refs`, `Co-authored-by`, and a custom-name choice for any other trailer. `Enter` edits the selected footer, with Footer name, multiline Value, and Save footer controls; `Space` creates or edits the one `BREAKING CHANGE` footer; `Delete` removes the selected footer; `Ctrl+Up` and `Ctrl+Down` reorder it. The modal keeps edits isolated until Save, and help preserves its state. These editors and shortcuts do not exist when their section is hidden.

## Keyboard behavior

| Key | Form mode | Type picker mode |
|---|---|---|
| `Tab` | Next field | No action |
| `Shift+Tab` | Previous field | No action |
| `Up` / `Down` | Previous / next field; select staged file when focused | Change highlighted item |
| `Home` / `End` | First / last staged file when focused | Edit search query |
| `Enter` | Open Type picker, advance text field, or submit from Commit | Select highlighted item |
| `Ctrl+Enter` | Submit from any field | No action |
| `F1` | Open or close help | Open or close help |
| `Space` | Toggle Breaking or Sign; include or exclude the selected staged file; protect the final included file | Insert a search space |
| `Esc` | Cancel application | Close picker and keep existing type |
| `Ctrl+C` | Cancel application | Cancel application |
| Printable text | Open Type picker with the character when Type is focused; otherwise edit focused text field | Filter choices |
| Backspace/Delete/Home/End | Edit focused text field | Edit search query |
| Paste | Open Type picker with sanitized text when Type is focused; otherwise insert sanitized one-line text | Insert sanitized search query |

Vim `j` and `k` are intentionally not navigation shortcuts because they must remain searchable type characters.

## Type picker

Pressing `Enter` on Type opens a popup containing the standard types. Typing or pasting while Type is focused opens the same popup with that value as its query. The popup uses a case-insensitive prefix match over its small list; fuzzy-search dependencies are not justified.

- Arrow keys change selection.
- A query filters the list.
- `Enter` selects the highlighted standard type.
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
2. Display a concise field-specific error in Error.
3. Move focus to the first invalid visible field.

On cancel, restore the terminal and return success without invoking Git. On valid submission, the UI returns the draft and signing choice to `main`; Git runs only after the terminal is restored.

## Event handling constraints

- Process only Crossterm `KeyEventKind::Press` to avoid duplicate actions on platforms that emit release events.
- Handle `Resize` by redrawing.
- Treat paste explicitly and remove or reject line breaks before they reach one-line fields.
- Convert contiguous pasted line breaks to one space. Reject NUL, escape, and other control characters, and keep the current value unchanged when a field or paste limit is exceeded.
- Limit type to 64 characters, scope to 128, message to 512, issue to 20, and a single paste to 4096 characters. Show concise feedback in Error for rejected input.
- Ignore focus and mouse events in v1.

## Staged-change context

Before the form opens, cocommit captures the Git index. The expanded layout shows the staged-file total, `A/M/D/R` counts, insertion/deletion totals, binary-file count when applicable, and a bounded scrollable list with a visible selected file and inclusion checkbox. File names are rendered safely even when they contain unusual characters. `Space` changes that checkbox without invoking Git. On submission, cocommit removes every excluded file from the index without changing its working-tree content, then commits the remaining files. A rename passes both paths. If it is the final included file, the operation is blocked with `Cannot unstage the last staged file`. The compact layout shows the aggregate summary with the selected-file position and places Preview directly below the form rows it renders. When this section is hidden, no staged summary, selection, checkbox, or unstage action exists; every staged file is committed.
