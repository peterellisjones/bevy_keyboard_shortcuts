# bevy_keyboard_shortcuts

A Bevy plugin for handling keyboard shortcuts with modifier support.

This crate provides a flexible way to define and check keyboard shortcuts in Bevy applications,
with support for modifier keys (Ctrl, Alt, Shift, Super) and both single-press and repeating inputs.

## Features

- Define keyboard shortcuts with optional modifiers
- Support for repeating shortcuts (held keys) and single-press shortcuts
- Serialization/deserialization support via serde for easy configuration
- Pretty-printing of shortcuts for UI display

## Example

```rust
use bevy::prelude::*;
use bevy_keyboard_shortcuts::Shortcuts;

fn check_shortcuts(keyboard: Res<ButtonInput<KeyCode>>) {
    let jump = Shortcuts::single_press(&[KeyCode::Space]);
    let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();

    if jump.pressed(&keyboard) {
        println!("Jump!");
    }
    if save.pressed(&keyboard) {
        println!("Saving!");
    }
}
```

## Usage Pattern

A typical usage pattern involves creating a Bevy Resource with your shortcuts:

```rust
use bevy::prelude::*;
use bevy_keyboard_shortcuts::Shortcuts;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct ShortcutSettings {
    pub move_left: Shortcuts,
    pub move_right: Shortcuts,
    pub quick_save: Shortcuts,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            move_left: Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]),
            move_right: Shortcuts::repeating(&[KeyCode::KeyD, KeyCode::ArrowRight]),
            quick_save: Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl(),
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    shortcuts: Res<ShortcutSettings>,
) {
    if shortcuts.move_left.pressed(&keyboard) {
        // Move left
    }
    if shortcuts.move_right.pressed(&keyboard) {
        // Move right
    }
    if shortcuts.quick_save.pressed(&keyboard) {
        // Save game
    }
}
```

## YAML Configuration

This crate works well with configuration files. Here's an example YAML configuration:

```yaml
# Camera controls - repeating shortcuts (continuous input)
move_left:
  repeats: true
  shortcuts:
    - key: "KeyA"
    - key: "ArrowLeft"

move_right:
  repeats: true
  shortcuts:
    - key: "KeyD"
    - key: "ArrowRight"

# Example with modifiers
save:
  repeats: false
  shortcuts:
    - key: "KeyS"
      modifiers:
        control: RequirePressed
```

## Key Names Reference

Key names in YAML/JSON configuration match Bevy's `KeyCode` enum variants exactly. Remove the `KeyCode::` prefix:

- Letters: `KeyA`, `KeyB`, ..., `KeyZ`
- Numbers: `Digit0`, `Digit1`, ..., `Digit9`
- Function keys: `F1`, `F2`, ..., `F12`
- Arrows: `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`
- Special keys: `Space`, `Enter`, `Escape`, `Tab`, `Backspace`, `Delete`
- Numpad: `Numpad0`, `Numpad1`, ..., `Numpad9`, `NumpadAdd`, `NumpadSubtract`

For example: `KeyCode::KeyA` becomes `"KeyA"`, `KeyCode::Space` becomes `"Space"`.

For the complete list, see [Bevy's KeyCode documentation](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html).

## Modifier Behavior

By default, **all modifiers are ignored** - shortcuts trigger regardless of modifier state.

Each modifier key (Ctrl, Alt, Shift, Super) can be configured:

- **Ignore** (DEFAULT) - Don't check this modifier (works with or without)
  - This is the default if no modifier methods are called
  - The shortcut triggers regardless of the modifier's state
- **RequirePressed** - The modifier MUST be pressed
  - Use `.with_ctrl()`, `.with_alt()`, `.with_shift()`, `.with_super()`
  - In YAML: `control: RequirePressed`, `alt: RequirePressed`, etc.
- **RequireNotPressed** - The modifier must NOT be pressed
  - Use `.without_ctrl()`, `.without_alt()`, `.without_shift()`, `.without_super()`
  - In YAML: `control: RequireNotPressed`, `alt: RequireNotPressed`, etc.

### Examples

```rust
// Default - ignores all modifiers
let flexible = Shortcuts::single_press(&[KeyCode::KeyA]);

// Requires Ctrl
let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();

// Requires Ctrl AND Shift
let redo = Shortcuts::single_press(&[KeyCode::KeyZ]).with_ctrl().with_shift();

// Forbids Ctrl (S without Ctrl)
let action = Shortcuts::single_press(&[KeyCode::KeyS]).without_ctrl();
```
