# bevy_keyboard_shortcuts

Keyboard shortcut system for Bevy applications, with modifier key support and YAML/JSON configuration support.

## Features

- **Modifier Key Support** - Full support for Ctrl, Alt, Shift, and Super keys with flexible matching modes
- **Repeating vs Single-Press** - Configure shortcuts to trigger continuously while held or just once per press
- **Multiple Alternatives** - Define multiple key combinations for the same action (e.g., WASD + arrow keys)
- **Serde Integration** - Easy configuration via YAML, JSON, or any serde-compatible format
- **Pretty Display** - Human-readable shortcut strings for UI display (e.g., "Ctrl + S")

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_keyboard_shortcuts = { git = "https://github.com/peterellisjones/bevy_keyboard_shortcuts" }
```

## Quick Start

### Basic Usage

```rust
use bevy::prelude::*;
use bevy_keyboard_shortcuts::Shortcuts;

fn my_system(keyboard: Res<ButtonInput<KeyCode>>) {
    // Simple single key shortcut (non-repeating)
    let jump = Shortcuts::single_press(&[KeyCode::Space]);

    if jump.pressed(&keyboard) {
        println!("Space pressed - jumping!");
    }

    // Shortcut with modifier (chainable)
    let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
    if save.pressed(&keyboard) {
        println!("Ctrl+S pressed - saving!");
    }

    // Multiple alternatives (repeating)
    let move_left = Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
    if move_left.pressed(&keyboard) {
        println!("Moving left!");
    }
}
```

### Configuration-Based Approach

Create a settings struct with your shortcuts:

```rust
use bevy::prelude::*;
use bevy_keyboard_shortcuts::Shortcuts;
use serde::{Deserialize, Serialize};

#[derive(Resource, Reflect, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GameShortcuts {
    pub move_left: Shortcuts,
    pub move_right: Shortcuts,
    pub move_up: Shortcuts,
    pub move_down: Shortcuts,
}

fn movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    shortcuts: Res<GameShortcuts>,
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut transform = player.single_mut();
    let speed = 100.0 * time.delta_secs();

    if shortcuts.move_left.pressed(&keyboard) {
        transform.translation.x -= speed;
    }
    if shortcuts.move_right.pressed(&keyboard) {
        transform.translation.x += speed;
    }
    if shortcuts.move_up.pressed(&keyboard) {
        transform.translation.y += speed;
    }
    if shortcuts.move_down.pressed(&keyboard) {
        transform.translation.y -= speed;
    }
}
```

### YAML Configuration

Create a `shortcuts.yaml` file. Key names match Bevy's `KeyCode` enum variants (e.g., `KeyCode::KeyA` → `"KeyA"`, `KeyCode::Space` → `"Space"`):

```yaml
# Movement - repeating shortcuts (continuous input)
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

move_up:
  repeats: true
  shortcuts:
    - key: "KeyW"
    - key: "ArrowUp"

move_down:
  repeats: true
  shortcuts:
    - key: "KeyS"
    - key: "ArrowDown"

# Jump - non-repeating (single press)
jump:
  repeats: false
  shortcuts:
    - key: "Space"

# Save with modifier
save:
  repeats: false
  shortcuts:
    - key: "KeyS"
      modifiers:
        control: RequirePressed
```

#### Modifier Options

The `modifiers` section supports four modifier keys, each with three possible values:

- **`control`** - Ctrl/Command key
- **`alt`** - Alt/Option key
- **`shift`** - Shift key
- **`super`** - Super/Windows key

**Modifier values:**

- **Omitted** (DEFAULT) - If you don't specify a modifier, it's ignored (works with or without)
- **`RequirePressed`** - The modifier key MUST be pressed
- **`RequireNotPressed`** - The modifier key MUST NOT be pressed

**Important:** By default, if you don't specify a modifier in YAML, it is **ignored**, meaning the shortcut works regardless of that modifier's state. Only specify modifiers when you need explicit control.

**Examples:**

```yaml
# Ctrl+S (requires Ctrl, no other modifiers)
save:
  repeats: false
  shortcuts:
    - key: "KeyS"
      modifiers:
        control: RequirePressed

# Ctrl+Shift+Z (requires both Ctrl and Shift)
redo:
  repeats: false
  shortcuts:
    - key: "KeyZ"
      modifiers:
        control: RequirePressed
        shift: RequirePressed

# Alt+F4 (requires Alt)
quit:
  repeats: false
  shortcuts:
    - key: "F4"
      modifiers:
        alt: RequirePressed

# S without Ctrl (explicitly forbid Ctrl)
action:
  repeats: false
  shortcuts:
    - key: "KeyS"
      modifiers:
        control: RequireNotPressed

# A regardless of modifiers (default - omit modifiers section)
flexible:
  repeats: false
  shortcuts:
    - key: "KeyA"
```

## Core Concepts

### Builder Methods

Create shortcuts using the ergonomic builder API:

```rust
// Non-repeating shortcuts (single press)
let jump = Shortcuts::single_press(&[KeyCode::Space]);
let menu = Shortcuts::single_press(&[KeyCode::Escape, KeyCode::KeyM]);

// Repeating shortcuts (held continuously)
let move_left = Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
let zoom_in = Shortcuts::repeating(&[KeyCode::KeyQ]);

// With modifiers (require pressed)
let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
let redo = Shortcuts::single_press(&[KeyCode::KeyZ]).with_ctrl().with_shift();
let quit = Shortcuts::single_press(&[KeyCode::F4]).with_alt();

// Without modifiers (explicitly forbid)
let action = Shortcuts::single_press(&[KeyCode::KeyS]).without_ctrl();  // S without Ctrl
```

### Modifier Behavior

By default, **all modifiers are ignored** - shortcuts trigger regardless of modifier state.

Each modifier key (Ctrl, Alt, Shift, Super) can be configured with two states:

- **Ignore** (DEFAULT) - Don't check this modifier (works with or without)
  - This is the default if no modifier methods are called
  - The shortcut triggers regardless of the modifier's state
- **RequirePressed** - The modifier MUST be pressed
  - Use `.with_ctrl()`, `.with_alt()`, `.with_shift()`, `.with_super()`
- **RequireNotPressed** - The modifier must NOT be pressed
  - Use `.without_ctrl()`, `.without_alt()`, `.without_shift()`, `.without_super()`

### Repeating vs Non-Repeating

- **`repeats: true`** - Use for continuous actions (movement, camera control)
  - Triggers every frame while the key is held
  - Uses `pressed()` internally

- **`repeats: false`** - Use for single actions (menu toggles, saving)
  - Triggers only once when the key is first pressed
  - Uses `just_pressed()` internally

## Display Formatting

Shortcuts automatically format nicely for display in UI:

```rust
use bevy_keyboard_shortcuts::{Shortcut, Modifiers, ModifierType};

let shortcut = Shortcut {
    key: KeyCode::KeyS,
    modifiers: Modifiers {
        control: ModifierType::RequirePressed,
        ..Default::default()
    },
};

println!("{}", shortcut);  // Output: "Ctrl + S"
```

Multiple alternatives are comma-separated:

```rust
let shortcuts = Shortcuts {
    shortcuts: vec![
        Shortcut { key: KeyCode::KeyA, modifiers: Modifiers::default() },
        Shortcut { key: KeyCode::ArrowLeft, modifiers: Modifiers::default() },
    ],
    repeats: true,
};

println!("{}", shortcuts);  // Output: "A, ←"
```

## Key Names Reference

### Finding KeyCode Names

Key names in YAML/JSON configuration match Bevy's `KeyCode` enum variants exactly. Remove the `KeyCode::` prefix:

| KeyCode Variant        | YAML/JSON String | Display Name |
| ---------------------- | ---------------- | ------------ |
| `KeyCode::KeyA`        | `"KeyA"`         | `"A"`        |
| `KeyCode::Space`       | `"Space"`        | `"Space"`    |
| `KeyCode::Enter`       | `"Enter"`        | `"↵"`        |
| `KeyCode::ArrowLeft`   | `"ArrowLeft"`    | `"←"`        |
| `KeyCode::Digit1`      | `"Digit1"`       | `"1"`        |
| `KeyCode::F1`          | `"F1"`           | `"F1"`       |
| `KeyCode::ControlLeft` | `"ControlLeft"`  | `"Ctrl"`     |
| `KeyCode::Backspace`   | `"Backspace"`    | `"⌫"`        |

**Common key categories:**

- **Letters**: `KeyA`, `KeyB`, `KeyC`, ... `KeyZ`
- **Numbers**: `Digit0`, `Digit1`, ... `Digit9`
- **Function keys**: `F1`, `F2`, ... `F12`
- **Arrows**: `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`
- **Modifiers**: `ControlLeft`, `ControlRight`, `AltLeft`, `AltRight`, `ShiftLeft`, `ShiftRight`, `SuperLeft`, `SuperRight`
- **Special**: `Space`, `Enter`, `Escape`, `Tab`, `Backspace`, `Delete`
- **Numpad**: `Numpad0`, `Numpad1`, ... `Numpad9`, `NumpadAdd`, `NumpadSubtract`, etc.

For the complete list, see [Bevy's KeyCode documentation](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html).

### Display Names

The crate automatically converts key codes to user-friendly display strings for UI:

- Letters: `KeyA` → `"A"`
- Arrows: `ArrowLeft` → `"←"`
- Special: `Backspace` → `"⌫"`, `Enter` → `"↵"`
- Numpad: `Numpad5` → `"Num 5"`
