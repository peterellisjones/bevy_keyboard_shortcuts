//! A Bevy plugin for handling keyboard shortcuts with modifier support.
//!
//! This crate provides a flexible way to define and check keyboard shortcuts in Bevy applications,
//! with support for modifier keys (Ctrl, Alt, Shift, Super) and both single-press and repeating inputs.
//!
//! # Features
//!
//! - Define keyboard shortcuts with optional modifiers
//! - Support for repeating shortcuts (held keys) and single-press shortcuts
//! - Serialization/deserialization support via serde for easy configuration
//! - Pretty-printing of shortcuts for UI display
//!
//! # Example
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_keyboard_shortcuts::Shortcuts;
//!
//! fn check_shortcuts(keyboard: Res<ButtonInput<KeyCode>>) {
//!     let jump = Shortcuts::single_press(&[KeyCode::Space]);
//!     let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
//!
//!     if jump.pressed(&keyboard) {
//!         println!("Jump!");
//!     }
//!     if save.pressed(&keyboard) {
//!         println!("Saving!");
//!     }
//! }
//! ```
//!
//! # Usage Pattern
//!
//! A typical usage pattern involves creating a Bevy Resource with your shortcuts:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_keyboard_shortcuts::Shortcuts;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Resource)]
//! pub struct ShortcutSettings {
//!     pub move_left: Shortcuts,
//!     pub move_right: Shortcuts,
//!     pub quick_save: Shortcuts,
//! }
//!
//! impl Default for ShortcutSettings {
//!     fn default() -> Self {
//!         Self {
//!             move_left: Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]),
//!             move_right: Shortcuts::repeating(&[KeyCode::KeyD, KeyCode::ArrowRight]),
//!             quick_save: Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl(),
//!         }
//!     }
//! }
//!
//! fn handle_input(
//!     keyboard: Res<ButtonInput<KeyCode>>,
//!     shortcuts: Res<ShortcutSettings>,
//! ) {
//!     if shortcuts.move_left.pressed(&keyboard) {
//!         // Move left
//!     }
//!     if shortcuts.move_right.pressed(&keyboard) {
//!         // Move right
//!     }
//!     if shortcuts.quick_save.pressed(&keyboard) {
//!         // Save game
//!     }
//! }
//! ```
//!
//! # YAML Configuration
//!
//! This crate works well with configuration files. Here's an example YAML configuration:
//!
//! ```yaml
//! # Camera controls - repeating shortcuts (continuous input)
//! move_left:
//!   repeats: true
//!   shortcuts:
//!     - key: "KeyA"
//!     - key: "ArrowLeft"
//!
//! move_right:
//!   repeats: true
//!   shortcuts:
//!     - key: "KeyD"
//!     - key: "ArrowRight"
//!
//! # Example with modifiers
//! save:
//!   repeats: false
//!   shortcuts:
//!     - key: "KeyS"
//!       modifiers:
//!         control: RequirePressed
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

/// A single keyboard shortcut consisting of a key and optional modifiers.
///
/// This type is an internal implementation detail. Users should interact with
/// the `Shortcuts` type instead, which provides a more ergonomic API.
#[derive(Reflect, Debug, Clone, Deserialize, Serialize)]
struct Shortcut {
    /// The main key that must be pressed
    pub key: KeyCode,
    /// Optional modifier keys (Ctrl, Alt, Shift, Super)
    #[serde(default)]
    pub modifiers: Modifiers,
}

/// Specifies how a modifier key should be handled in a shortcut.
///
/// This enum controls whether a modifier key (Ctrl, Alt, Shift, Super) must be pressed
/// or must not be pressed. When wrapped in `Option`, `None` means the modifier is ignored.
#[derive(Reflect, Debug, Clone, PartialEq, Deserialize, Serialize)]
enum ModifierType {
    /// The modifier must be pressed for the shortcut to match
    RequirePressed,
    /// The modifier must NOT be pressed for the shortcut to match
    RequireNotPressed,
}

impl ModifierType {
    /// Checks if the modifier state matches this requirement.
    ///
    /// # Arguments
    ///
    /// * `pressed` - Whether the modifier key is currently pressed
    ///
    /// # Returns
    ///
    /// `true` if the current state matches the requirement, `false` otherwise
    pub fn matches(&self, pressed: bool) -> bool {
        match self {
            ModifierType::RequirePressed => pressed,
            ModifierType::RequireNotPressed => !pressed,
        }
    }
}

/// A collection of modifier key requirements for a shortcut.
///
/// This type is an internal implementation detail. Users should use the builder
/// methods on `Shortcuts` instead.
///
/// Each modifier is an `Option<ModifierType>`:
/// - `None` means the modifier is ignored (default)
/// - `Some(RequirePressed)` means the modifier must be pressed
/// - `Some(RequireNotPressed)` means the modifier must NOT be pressed
#[derive(Reflect, Debug, Clone, Default, Deserialize, Serialize)]
struct Modifiers {
    /// Control/Command key requirement (None = ignore)
    #[serde(default)]
    pub control: Option<ModifierType>,
    /// Alt/Option key requirement (None = ignore)
    #[serde(default)]
    pub alt: Option<ModifierType>,
    /// Shift key requirement (None = ignore)
    #[serde(default)]
    pub shift: Option<ModifierType>,
    /// Super/Windows key requirement (None = ignore)
    #[serde(default)]
    pub super_key: Option<ModifierType>,
}

impl Modifiers {
    /// Returns `true` if no modifiers are explicitly set (all are None).
    ///
    /// This checks if all modifiers are `None`, which is the default state meaning
    /// all modifiers are ignored.
    pub fn none(&self) -> bool {
        self.control.is_none()
            && self.alt.is_none()
            && self.shift.is_none()
            && self.super_key.is_none()
    }

    /// Checks if the current keyboard state matches all modifier requirements.
    ///
    /// # Arguments
    ///
    /// * `keys` - The current keyboard input state from Bevy
    ///
    /// # Returns
    ///
    /// `true` if all modifier requirements are satisfied, `false` otherwise
    pub fn pressed(&self, keys: &ButtonInput<KeyCode>) -> bool {
        let control_pressed =
            keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        let alt_pressed = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
        let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let super_pressed = keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight);

        Self::matches_modifier(self.control.as_ref(), control_pressed)
            && Self::matches_modifier(self.alt.as_ref(), alt_pressed)
            && Self::matches_modifier(self.shift.as_ref(), shift_pressed)
            && Self::matches_modifier(self.super_key.as_ref(), super_pressed)
    }

    /// Helper function to check if an optional modifier requirement matches the current state.
    fn matches_modifier(requirement: Option<&ModifierType>, pressed: bool) -> bool {
        match requirement {
            None => true, // None means ignore
            Some(modifier_type) => modifier_type.matches(pressed),
        }
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = Vec::new();

        if self.control == Some(ModifierType::RequirePressed) {
            parts.push("Ctrl");
        }
        if self.alt == Some(ModifierType::RequirePressed) {
            parts.push("Alt");
        }
        if self.shift == Some(ModifierType::RequirePressed) {
            parts.push("Shift");
        }
        if self.super_key == Some(ModifierType::RequirePressed) {
            parts.push("Super");
        }

        write!(f, "{}", parts.join(" + "))
    }
}

impl Shortcut {
    /// Returns a human-readable string representation of the key.
    ///
    /// This converts Bevy's KeyCode debug format into a more user-friendly display string.
    /// For example, `KeyA` becomes `"A"`, `ArrowLeft` becomes `"←"`, etc.
    fn key_str(&self) -> String {
        let debug_str = format!("{:?}", self.key);

        KEY_DISPLAY_MAP
            .get(debug_str.as_str())
            .map(|s| s.to_string())
            .unwrap_or(debug_str)
    }

    /// Checks if the shortcut is currently being pressed (held down).
    ///
    /// This is useful for continuous actions like camera movement where the action
    /// should continue as long as the key is held.
    ///
    /// # Arguments
    ///
    /// * `keys` - The current keyboard input state from Bevy
    ///
    /// # Returns
    ///
    /// `true` if both the key and all required modifiers are currently pressed
    pub fn pressed(&self, keys: &ButtonInput<KeyCode>) -> bool {
        keys.pressed(self.key) && self.modifiers.pressed(keys)
    }

    /// Checks if the shortcut was just pressed this frame.
    ///
    /// This is useful for single-action events like saving or opening a menu,
    /// where you only want to trigger once per key press.
    ///
    /// # Arguments
    ///
    /// * `keys` - The current keyboard input state from Bevy
    ///
    /// # Returns
    ///
    /// `true` if the key was just pressed this frame and all required modifiers are pressed
    pub fn just_pressed(&self, keys: &ButtonInput<KeyCode>) -> bool {
        keys.just_pressed(self.key) && self.modifiers.pressed(keys)
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.modifiers.none() {
            write!(f, "{} + ", self.modifiers)?
        }
        write!(f, "{}", self.key_str())
    }
}

/// A collection of alternative shortcuts that trigger the same action.
///
/// This allows defining multiple key combinations for the same action (e.g., both
/// arrow keys and WASD for movement). The `repeats` field controls whether the
/// shortcuts trigger continuously while held or only once per press.
///
/// # Creating Shortcuts
///
/// Use the builder methods to create shortcuts programmatically:
///
/// - `Shortcuts::single_press(&[keys])` - Non-repeating shortcuts (single press)
/// - `Shortcuts::repeating(&[keys])` - Repeating shortcuts (held)
/// - `.with_ctrl()` - Add Ctrl modifier (chainable)
/// - `.with_alt()` - Add Alt modifier (chainable)
/// - `.with_shift()` - Add Shift modifier (chainable)
/// - `.with_super()` - Add Super modifier (chainable)
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::KeyCode;
/// use bevy_keyboard_shortcuts::Shortcuts;
///
/// // Single key
/// let jump = Shortcuts::single_press(&[KeyCode::Space]);
///
/// // With modifiers (chainable)
/// let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
/// let redo = Shortcuts::single_press(&[KeyCode::KeyZ]).with_ctrl().with_shift();
///
/// // Multiple alternatives for movement
/// let move_left = Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
/// ```
#[derive(Reflect, Debug, Deserialize, Serialize, Default)]
pub struct Shortcuts {
    /// List of alternative shortcuts that trigger the same action
    /// This field is public for serde deserialization but should not be accessed directly.
    /// Use the builder methods instead.
    #[doc(hidden)]
    #[allow(private_interfaces)]
    pub shortcuts: Vec<Shortcut>,
    /// If `true`, the shortcut triggers continuously while held.
    /// If `false`, it only triggers once when initially pressed.
    #[serde(default)]
    #[doc(hidden)]
    #[allow(private_interfaces)]
    pub repeats: bool,
}

impl Shortcuts {
    /// Creates non-repeating shortcuts from a slice of keys (no modifiers).
    ///
    /// Non-repeating shortcuts only trigger once when initially pressed,
    /// useful for menu toggles, saving, or other single-action events.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// // Single key
    /// let jump = Shortcuts::single_press(&[KeyCode::Space]);
    ///
    /// // Multiple alternatives
    /// let menu = Shortcuts::single_press(&[KeyCode::Escape, KeyCode::KeyM]);
    /// ```
    pub fn single_press(keys: &[KeyCode]) -> Self {
        Self {
            shortcuts: keys
                .iter()
                .map(|&key| Shortcut {
                    key,
                    modifiers: Modifiers::default(),
                })
                .collect(),
            repeats: false,
        }
    }

    /// Creates repeating shortcuts from a slice of keys (no modifiers).
    ///
    /// Repeating shortcuts trigger continuously while held, useful for
    /// continuous actions like movement or camera control.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// // Single key
    /// let zoom_in = Shortcuts::repeating(&[KeyCode::KeyQ]);
    ///
    /// // Multiple alternatives
    /// let move_left = Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
    /// ```
    pub fn repeating(keys: &[KeyCode]) -> Self {
        Self {
            shortcuts: keys
                .iter()
                .map(|&key| Shortcut {
                    key,
                    modifiers: Modifiers::default(),
                })
                .collect(),
            repeats: true,
        }
    }

    /// Adds Ctrl modifier to the first shortcut.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// let save = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Ctrl modifier is already set.
    pub fn with_ctrl(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(
                shortcut.modifiers.control.is_none(),
                "Ctrl modifier already set"
            );
            shortcut.modifiers.control = Some(ModifierType::RequirePressed);
        }
        self
    }

    /// Adds Alt modifier to the first shortcut.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// let shortcut = Shortcuts::single_press(&[KeyCode::F4]).with_alt();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Alt modifier is already set.
    pub fn with_alt(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(shortcut.modifiers.alt.is_none(), "Alt modifier already set");
            shortcut.modifiers.alt = Some(ModifierType::RequirePressed);
        }
        self
    }

    /// Adds Shift modifier to the first shortcut.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// let shortcut = Shortcuts::single_press(&[KeyCode::Tab]).with_shift();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Shift modifier is already set.
    pub fn with_shift(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(
                shortcut.modifiers.shift.is_none(),
                "Shift modifier already set"
            );
            shortcut.modifiers.shift = Some(ModifierType::RequirePressed);
        }
        self
    }

    /// Adds Super/Windows key modifier to the first shortcut.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// let shortcut = Shortcuts::single_press(&[KeyCode::KeyD]).with_super();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Super modifier is already set.
    pub fn with_super(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(
                shortcut.modifiers.super_key.is_none(),
                "Super modifier already set"
            );
            shortcut.modifiers.super_key = Some(ModifierType::RequirePressed);
        }
        self
    }

    /// Requires that Ctrl is NOT pressed for the first shortcut.
    ///
    /// This is useful when you want to explicitly forbid a modifier key,
    /// for example to have "S" work differently from "Ctrl+S".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// // Only trigger when S is pressed WITHOUT Ctrl
    /// let action = Shortcuts::single_press(&[KeyCode::KeyS]).without_ctrl();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Ctrl modifier is already set.
    pub fn without_ctrl(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(
                shortcut.modifiers.control.is_none(),
                "Ctrl modifier already set"
            );
            shortcut.modifiers.control = Some(ModifierType::RequireNotPressed);
        }
        self
    }

    /// Requires that Alt is NOT pressed for the first shortcut.
    ///
    /// This is useful when you want to explicitly forbid a modifier key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// // Only trigger when F4 is pressed WITHOUT Alt
    /// let action = Shortcuts::single_press(&[KeyCode::F4]).without_alt();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Alt modifier is already set.
    pub fn without_alt(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(shortcut.modifiers.alt.is_none(), "Alt modifier already set");
            shortcut.modifiers.alt = Some(ModifierType::RequireNotPressed);
        }
        self
    }

    /// Requires that Shift is NOT pressed for the first shortcut.
    ///
    /// This is useful when you want to explicitly forbid a modifier key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// // Only trigger when Tab is pressed WITHOUT Shift
    /// let action = Shortcuts::single_press(&[KeyCode::Tab]).without_shift();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Shift modifier is already set.
    pub fn without_shift(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(
                shortcut.modifiers.shift.is_none(),
                "Shift modifier already set"
            );
            shortcut.modifiers.shift = Some(ModifierType::RequireNotPressed);
        }
        self
    }

    /// Requires that Super/Windows key is NOT pressed for the first shortcut.
    ///
    /// This is useful when you want to explicitly forbid a modifier key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::KeyCode;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// // Only trigger when D is pressed WITHOUT Super
    /// let action = Shortcuts::single_press(&[KeyCode::KeyD]).without_super();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics in debug mode if Super modifier is already set.
    pub fn without_super(mut self) -> Self {
        if let Some(shortcut) = self.shortcuts.first_mut() {
            debug_assert!(
                shortcut.modifiers.super_key.is_none(),
                "Super modifier already set"
            );
            shortcut.modifiers.super_key = Some(ModifierType::RequireNotPressed);
        }
        self
    }

    /// Checks if any of the shortcuts in this collection are activated.
    ///
    /// The behavior depends on the `repeats` field:
    /// - If `repeats` is `true`, checks if any shortcut is currently pressed (held)
    /// - If `repeats` is `false`, checks if any shortcut was just pressed this frame
    ///
    /// # Arguments
    ///
    /// * `keys` - The current keyboard input state from Bevy
    ///
    /// # Returns
    ///
    /// `true` if any of the shortcuts are activated according to the repeat setting
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use bevy_keyboard_shortcuts::Shortcuts;
    ///
    /// fn movement_system(keyboard: Res<ButtonInput<KeyCode>>) {
    ///     let shortcuts = Shortcuts::repeating(&[KeyCode::KeyA]);
    ///
    ///     if shortcuts.pressed(&keyboard) {
    ///         // Move the character
    ///     }
    /// }
    /// ```
    pub fn pressed(&self, keys: &ButtonInput<KeyCode>) -> bool {
        if self.repeats {
            self.shortcuts.iter().any(|s| s.pressed(keys))
        } else {
            self.shortcuts.iter().any(|s| s.just_pressed(keys))
        }
    }
}

impl fmt::Display for Shortcuts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keys = self
            .shortcuts
            .iter()
            .map(|shortcut| shortcut.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}", keys)
    }
}

/// Internal mapping of KeyCode debug strings to human-readable display strings.
///
/// This is used by `Shortcut::key_str()` to convert Bevy's KeyCode variants into
/// user-friendly strings for display in UI. For example:
/// - `KeyA` → `"A"`
/// - `ArrowLeft` → `"←"`
/// - `Backspace` → `"⌫"`
static KEY_DISPLAY_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Punctuation and symbols - show actual character
    m.insert("Backquote", "`");
    m.insert("Backslash", "\\");
    m.insert("BracketLeft", "[");
    m.insert("BracketRight", "]");
    m.insert("Comma", ",");
    m.insert("Equal", "=");
    m.insert("Minus", "-");
    m.insert("Period", ".");
    m.insert("Quote", "'");
    m.insert("Semicolon", ";");
    m.insert("Slash", "/");

    // Special keys with symbols
    m.insert("Backspace", "⌫");
    m.insert("Delete", "⌦");
    m.insert("Enter", "↵");
    m.insert("Escape", "Esc");
    m.insert("Tab", "⇥");
    m.insert("Space", "Space");

    // Navigation keys
    m.insert("ArrowUp", "↑");
    m.insert("ArrowDown", "↓");
    m.insert("ArrowLeft", "←");
    m.insert("ArrowRight", "→");
    m.insert("Home", "Home");
    m.insert("End", "End");
    m.insert("PageUp", "PgUp");
    m.insert("PageDown", "PgDn");

    // Lock keys
    m.insert("CapsLock", "CapsLock");
    m.insert("NumLock", "NumLock");
    m.insert("ScrollLock", "ScrollLock");

    // System keys
    m.insert("PrintScreen", "PrtScr");
    m.insert("Pause", "Pause");
    m.insert("ContextMenu", "Menu");
    m.insert("Insert", "Insert");

    // Letter keys
    for letter in 'A'..='Z' {
        m.insert(
            Box::leak(format!("Key{}", letter).into_boxed_str()),
            Box::leak(letter.to_string().into_boxed_str()),
        );
    }

    // Digit keys
    for digit in 0..=9 {
        m.insert(
            Box::leak(format!("Digit{}", digit).into_boxed_str()),
            Box::leak(digit.to_string().into_boxed_str()),
        );
    }

    // Numpad digit keys
    for digit in 0..=9 {
        m.insert(
            Box::leak(format!("Numpad{}", digit).into_boxed_str()),
            Box::leak(format!("Num {}", digit).into_boxed_str()),
        );
    }

    // Numpad operators
    m.insert("NumpadAdd", "Num +");
    m.insert("NumpadSubtract", "Num -");
    m.insert("NumpadMultiply", "Num *");
    m.insert("NumpadDivide", "Num /");
    m.insert("NumpadDecimal", "Num .");
    m.insert("NumpadEqual", "Num =");
    m.insert("NumpadEnter", "Num Enter");
    m.insert("NumpadComma", "Num ,");
    m.insert("NumpadBackspace", "Num ⌫");
    m.insert("NumpadClear", "Num Clear");
    m.insert("NumpadClearEntry", "Num CE");
    m.insert("NumpadHash", "Num #");
    m.insert("NumpadParenLeft", "Num (");
    m.insert("NumpadParenRight", "Num )");
    m.insert("NumpadStar", "Num *");
    m.insert("NumpadMemoryAdd", "Num M+");
    m.insert("NumpadMemoryClear", "Num MC");
    m.insert("NumpadMemoryRecall", "Num MR");
    m.insert("NumpadMemoryStore", "Num MS");
    m.insert("NumpadMemorySubtract", "Num M-");

    // Function keys
    for i in 1..=35 {
        m.insert(
            Box::leak(format!("F{}", i).into_boxed_str()),
            Box::leak(format!("F{}", i).into_boxed_str()),
        );
    }

    // Media keys
    m.insert("MediaPlayPause", "Play/Pause");
    m.insert("MediaStop", "Stop");
    m.insert("MediaTrackNext", "Next Track");
    m.insert("MediaTrackPrevious", "Prev Track");
    m.insert("MediaSelect", "Media Select");
    m.insert("AudioVolumeUp", "Vol+");
    m.insert("AudioVolumeDown", "Vol-");
    m.insert("AudioVolumeMute", "Mute");

    // Browser keys
    m.insert("BrowserBack", "Browser Back");
    m.insert("BrowserForward", "Browser Forward");
    m.insert("BrowserRefresh", "Refresh");
    m.insert("BrowserStop", "Browser Stop");
    m.insert("BrowserSearch", "Browser Search");
    m.insert("BrowserFavorites", "Favorites");
    m.insert("BrowserHome", "Browser Home");

    // Application keys
    m.insert("LaunchMail", "Mail");
    m.insert("LaunchApp1", "App1");
    m.insert("LaunchApp2", "App2");
    m.insert("Copy", "Copy");
    m.insert("Cut", "Cut");
    m.insert("Paste", "Paste");
    m.insert("Undo", "Undo");
    m.insert("Find", "Find");
    m.insert("Open", "Open");
    m.insert("Select", "Select");

    // Modifier keys (for completeness)
    m.insert("ControlLeft", "Ctrl");
    m.insert("ControlRight", "Ctrl");
    m.insert("AltLeft", "Alt");
    m.insert("AltRight", "Alt");
    m.insert("ShiftLeft", "Shift");
    m.insert("ShiftRight", "Shift");
    m.insert("SuperLeft", "Super");
    m.insert("SuperRight", "Super");

    // International keys
    m.insert("IntlBackslash", "Intl \\");
    m.insert("IntlRo", "Ro");
    m.insert("IntlYen", "¥");

    // Language keys
    m.insert("Lang1", "Lang1");
    m.insert("Lang2", "Lang2");
    m.insert("Lang3", "Lang3");
    m.insert("Lang4", "Lang4");
    m.insert("Lang5", "Lang5");
    m.insert("KanaMode", "Kana");
    m.insert("Hiragana", "Hiragana");
    m.insert("Katakana", "Katakana");
    m.insert("Convert", "Convert");
    m.insert("NonConvert", "NonConvert");

    // Additional application/system keys
    m.insert("Again", "Again");
    m.insert("Resume", "Resume");
    m.insert("Suspend", "Suspend");
    m.insert("Abort", "Abort");
    m.insert("Props", "Props");
    m.insert("Help", "Help");

    // Power keys
    m.insert("Power", "Power");
    m.insert("Sleep", "Sleep");
    m.insert("WakeUp", "WakeUp");
    m.insert("Eject", "⏏");

    // Function lock and other special keys
    m.insert("Fn", "Fn");
    m.insert("FnLock", "FnLock");
    m.insert("Turbo", "Turbo");
    m.insert("Meta", "Meta");
    m.insert("Hyper", "Hyper");

    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_display_single() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyA]);
        assert_eq!(shortcuts.to_string(), "A");
    }

    #[test]
    fn test_shortcuts_display_multiple() {
        let shortcuts = Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
        assert_eq!(shortcuts.to_string(), "A, ←");
    }

    #[test]
    fn test_shortcuts_display_with_modifiers() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
        assert_eq!(shortcuts.to_string(), "Ctrl + S");
    }

    #[test]
    fn test_shortcuts_display_multiple_modifiers() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyZ])
            .with_ctrl()
            .with_shift();
        assert_eq!(shortcuts.to_string(), "Ctrl + Shift + Z");
    }

    #[test]
    fn test_shortcuts_display_mixed() {
        // Test displaying shortcuts with different modifiers
        let ctrl_s = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
        let super_s = Shortcuts::single_press(&[KeyCode::KeyS]).with_super();

        assert_eq!(ctrl_s.to_string(), "Ctrl + S");
        assert_eq!(super_s.to_string(), "Super + S");
    }

    #[test]
    fn test_shortcuts_display_special_keys() {
        let shortcuts =
            Shortcuts::single_press(&[KeyCode::ArrowUp, KeyCode::Space, KeyCode::Enter]);

        assert_eq!(shortcuts.to_string(), "↑, Space, ↵");
    }

    #[test]
    fn test_shortcuts_pressed_repeating_when_held() {
        let shortcuts = Shortcuts::repeating(&[KeyCode::KeyA]);
        let mut keys = ButtonInput::<KeyCode>::default();

        // Not pressed initially
        assert!(!shortcuts.pressed(&keys));

        // Press the key
        keys.press(KeyCode::KeyA);
        assert!(shortcuts.pressed(&keys));

        // Still pressed (simulating held key)
        assert!(shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_pressed_non_repeating_just_pressed() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyA]);
        let mut keys = ButtonInput::<KeyCode>::default();

        // Not pressed initially
        assert!(!shortcuts.pressed(&keys));

        // Press the key
        keys.press(KeyCode::KeyA);
        assert!(shortcuts.pressed(&keys));

        // Clear just_pressed state (simulating next frame)
        keys.clear_just_pressed(KeyCode::KeyA);
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_pressed_multiple_alternatives() {
        let shortcuts = Shortcuts::repeating(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press first alternative
        keys.press(KeyCode::KeyA);
        assert!(shortcuts.pressed(&keys));

        keys.release(KeyCode::KeyA);
        keys.clear();

        // Press second alternative
        keys.press(KeyCode::ArrowLeft);
        assert!(shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_pressed_with_modifiers() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyS]).with_ctrl();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press S without Ctrl - should not match
        keys.press(KeyCode::KeyS);
        assert!(!shortcuts.pressed(&keys));

        // Press Ctrl as well - should match
        keys.press(KeyCode::ControlLeft);
        assert!(shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_pressed_default_ignore_modifiers() {
        // This tests the default behavior - modifiers are ignored
        let shortcuts = Shortcuts::repeating(&[KeyCode::KeyA]);
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press A without Ctrl - should match
        keys.press(KeyCode::KeyA);
        assert!(shortcuts.pressed(&keys));

        // Press Ctrl as well - should still match (default is Ignore)
        keys.press(KeyCode::ControlLeft);
        assert!(shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_pressed_empty_shortcuts() {
        let shortcuts = Shortcuts::default();
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyA);

        // No shortcuts defined, should never match
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_pressed_multiple_modifiers() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyZ])
            .with_ctrl()
            .with_shift();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press Z with only Ctrl - should not match
        keys.press(KeyCode::KeyZ);
        keys.press(KeyCode::ControlLeft);
        assert!(!shortcuts.pressed(&keys));

        // Add Shift - should now match
        keys.press(KeyCode::ShiftLeft);
        assert!(shortcuts.pressed(&keys));
    }

    #[test]
    #[should_panic(expected = "Ctrl modifier already set")]
    #[cfg(debug_assertions)]
    fn test_shortcuts_duplicate_modifier_panics() {
        let _shortcuts = Shortcuts::single_press(&[KeyCode::KeyS])
            .with_ctrl()
            .with_ctrl();
    }

    #[test]
    fn test_shortcuts_press_and_repeating() {
        let press_shortcuts = Shortcuts::single_press(&[KeyCode::KeyW]);
        assert!(!press_shortcuts.repeats);

        let repeating_shortcuts = Shortcuts::repeating(&[KeyCode::KeyW]);
        assert!(repeating_shortcuts.repeats);
    }

    #[test]
    fn test_shortcuts_without_ctrl() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyS]).without_ctrl();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press S without Ctrl - should match
        keys.press(KeyCode::KeyS);
        assert!(shortcuts.pressed(&keys));

        // Press Ctrl as well - should NOT match
        keys.press(KeyCode::ControlLeft);
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_without_alt() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::F4]).without_alt();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press F4 without Alt - should match
        keys.press(KeyCode::F4);
        assert!(shortcuts.pressed(&keys));

        // Press Alt as well - should NOT match
        keys.press(KeyCode::AltLeft);
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_without_shift() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::Tab]).without_shift();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press Tab without Shift - should match
        keys.press(KeyCode::Tab);
        assert!(shortcuts.pressed(&keys));

        // Press Shift as well - should NOT match
        keys.press(KeyCode::ShiftLeft);
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_without_super() {
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyD]).without_super();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press D without Super - should match
        keys.press(KeyCode::KeyD);
        assert!(shortcuts.pressed(&keys));

        // Press Super as well - should NOT match
        keys.press(KeyCode::SuperLeft);
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_with_and_without_modifiers() {
        // Require Ctrl but forbid Alt
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyZ])
            .with_ctrl()
            .without_alt();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press Z with Ctrl but no Alt - should match
        keys.press(KeyCode::KeyZ);
        keys.press(KeyCode::ControlLeft);
        assert!(shortcuts.pressed(&keys));

        // Add Alt - should NOT match
        keys.press(KeyCode::AltLeft);
        assert!(!shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_default_ignores_modifiers() {
        // Default behavior should ignore all modifiers
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyA]);
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press A without modifiers - should match
        keys.press(KeyCode::KeyA);
        assert!(shortcuts.pressed(&keys));

        // Press A with Ctrl - should still match (default is Ignore)
        keys.press(KeyCode::ControlLeft);
        assert!(shortcuts.pressed(&keys));

        // Add Alt - should still match
        keys.press(KeyCode::AltLeft);
        assert!(shortcuts.pressed(&keys));

        // Add Shift - should still match
        keys.press(KeyCode::ShiftLeft);
        assert!(shortcuts.pressed(&keys));

        // Add Super - should still match
        keys.press(KeyCode::SuperLeft);
        assert!(shortcuts.pressed(&keys));
    }

    #[test]
    fn test_shortcuts_mixed_modifier_modes() {
        // Require Ctrl, forbid Shift, Alt defaults to Ignore
        let shortcuts = Shortcuts::single_press(&[KeyCode::KeyZ])
            .with_ctrl()
            .without_shift();
        let mut keys = ButtonInput::<KeyCode>::default();

        // Press Z with Ctrl, no Shift - should match
        keys.press(KeyCode::KeyZ);
        keys.press(KeyCode::ControlLeft);
        assert!(shortcuts.pressed(&keys));

        // Add Alt - should still match (Alt defaults to Ignore)
        keys.press(KeyCode::AltLeft);
        assert!(shortcuts.pressed(&keys));

        // Add Shift - should NOT match (forbidden)
        keys.press(KeyCode::ShiftLeft);
        assert!(!shortcuts.pressed(&keys));
    }
}
