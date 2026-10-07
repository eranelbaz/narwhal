# Editor modes

The SQL editor in the main pane supports three input models.
Switch at runtime with `:mode vim|basic|emacs`, or set
`[editor].mode` in `config.toml`.

| Mode    | Feel                          | Default leader |
|---------|-------------------------------|----------------|
| `vim`   | Normal / Insert / Visual      | `:`            |
| `basic` | Modeless, IDE-style           | `:` (palette)  |
| `emacs` | Ctrl- and Meta- chords        | `C-x`          |

The lower text buffer is shared across modes — switching modes
mid-edit does not lose your text.

## Vim mode (default)

Standard subset: `hjkl`, `w` / `b` / `e`, `0` / `$` / `gg` / `G`,
`i` / `a` / `o`, `dd` / `yy` / `dw` / `cc`, `v` / `V` / `Ctrl-V`,
`u` / `Ctrl-R`, `/` and `?` search, `:` command mode.

Visual mode accepts count prefixes (`3j`). Operators chain with
motions (`d3w`, `c$`). `5G` / `5gg` jump to line 5.

`x` deletes the character under the cursor into the clipboard (never
joins lines). `Delete` deletes forward; at end of line it joins the
next line. Both act on every cursor when multi-cursor is active.

In visual mode, `hjkl` / arrows / `w` / `b` / `0` / `$` / `Home` /
`End` / `G` extend the highlighted selection; `y` copies it, `d` / `x` /
`Delete` cut it, `c` replaces it. A mouse click leaves visual mode.

The mode indicator in the status bar shows `NORMAL` / `INSERT` /
`VISUAL` / `V-LINE` / `V-BLOCK`. Disable with
`[editor].show_mode_indicator = false`.

## Basic mode

Modeless, IDE-style. Typing inserts. Selection extends with
`Shift-Arrow`. `Ctrl-S` runs the buffer.

| Chord         | Action                          |
|---------------|---------------------------------|
| `Ctrl-S`      | Run buffer (same as F6)         |
| `Ctrl-Z`      | Undo                            |
| `Ctrl-Shift-Z`| Redo                            |
| `Ctrl-X` / `Ctrl-C` / `Ctrl-V` | Cut / copy / paste |
| `Ctrl-A`      | Select all                      |
| `Ctrl-F`      | Find                            |
| `Ctrl-/`      | Toggle line comment             |

## Emacs mode

Classic Emacs chords with a `C-x` prefix for two-key sequences.

| Chord       | Action                            |
|-------------|-----------------------------------|
| `C-f` / `C-b` | Forward / backward char         |
| `C-n` / `C-p` | Next / previous line            |
| `C-a` / `C-e` | Beginning / end of line         |
| `M-f` / `M-b` | Forward / backward word         |
| `C-d`       | Delete char forward               |
| `C-k`       | Kill to end of line               |
| `C-w` / `M-w` | Cut / copy selection            |
| `C-y`       | Yank                              |
| `C-x C-s`   | Run buffer                        |
| `C-x u`     | Undo                              |

When the `C-x` prefix is armed, the mode indicator flips to `C-x`.

## Mouse

See [`mouse.md`](./mouse.md).

## Keybinding presets

Layer IDE-style chords on top of the active mode:

```toml
[keybindings]
preset = "vscode"   # default | vscode | datagrip | intellij
```

| Preset    | Adds                                                  |
|-----------|-------------------------------------------------------|
| `vscode`  | `Ctrl-P` (goto), `Ctrl-Shift-P` (command palette)     |
| `datagrip`| `Ctrl-B` (focus sidebar), `Ctrl-Enter` (run statement)|
| `intellij`| Same as `datagrip`                                    |

User `[keymap.*]` overrides always win.

## Migration from v1.x

`[keybindings].vim_mode = false` still works and is interpreted as
`[editor].mode = "basic"`. The field is deprecated; prefer the new
form in new configs.
