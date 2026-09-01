# Terminal support

## Automated coverage

CI runs formatting, Clippy, the complete test suite, and a release build on
Ubuntu, Windows, and macOS. Every runner verifies that Git is available before
running the suite, so real-Git integration coverage cannot be skipped in CI.

Unix runners also run the PTY lifecycle tests. They verify cancellation and
that `SIGTERM` restores the alternate screen, bracketed paste mode, and cursor
state. Windows relies on deterministic non-interactive tests because those PTY
and signal semantics are Unix-specific.

## Release-candidate terminal matrix

Before a release candidate is accepted, record the operating-system version,
terminal version, Git version, result, and any observed limitation for every
row below. A failed row blocks the candidate until it is fixed, explicitly
removed from the supported set, or accepted as a documented limitation.

| Environment | Required checks |
|---|---|
| Windows Terminal with PowerShell | Cancel, unsigned commit, hook rejection, compact layout, paste rejection, and `Ctrl+C`. |
| Windows Terminal with Git Bash and zsh | Cancel, unsigned commit, compact layout, paste rejection, and `Ctrl+C`. |
| macOS Terminal.app | Cancel, unsigned commit, compact layout, paste rejection, and `Ctrl+C`. |
| iTerm2 | Cancel, unsigned commit, compact layout, paste rejection, and `Ctrl+C`. |
| Linux graphical terminal | Cancel, unsigned commit, hook rejection, compact layout, paste rejection, `Ctrl+C`, and `SIGTERM` restoration. |
| SSH to a Linux host | The Linux checks through a remote interactive session. |
| tmux on Linux | The Linux checks inside a tmux pane. |

The current known limitation is that some terminals send `Ctrl+Enter` as
`Enter`. In those terminals, focus `Commit` and press `Enter` instead. Record
whether each tested terminal distinguishes the key combination.

## Manual procedure

Run the following from Git Bash or zsh at the project root. It creates an
isolated repository and runs the current local binary against it:

```bash
project_root="$PWD"
sandbox="$(mktemp -d)"
git -C "$sandbox" init
git -C "$sandbox" config user.name "Cocommit Test"
git -C "$sandbox" config user.email "cocommit@example.com"
printf 'visual test\n' > "$sandbox/demo.txt"
git -C "$sandbox" add demo.txt
(cd "$sandbox" && cargo run --release --quiet --locked --manifest-path "$project_root/Cargo.toml")
git -C "$sandbox" log -1 --format='%B'
git -C "$sandbox" show --stat --oneline HEAD
```

In the form, verify focus visibility, the staged-file summary, the preview,
and a valid unsigned commit. Run it again and cancel with `Esc` or `Ctrl+C`.
For a compact-layout check, reduce the terminal before starting the command.
For Unix signal restoration, leave the form open and send `SIGTERM` from a
second terminal, then verify the shell remains usable.
