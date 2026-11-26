/// A key, as a Windows virtual-key number from 0x01 to 0xFE. The numbers are an OS standard and do
/// not change with a game build; 0x100 and higher are reserved. Mouse and gamepad buttons of the
/// game are not key codes.
///
/// Left and right modifiers keep their own codes (`LSHIFT`, `RCONTROL`, ...); `generic` folds them.
/// The named constants follow the Windows `VK_` names, with a few common aliases (`CTRL`, `ALT`,
/// `ENTER`, `LEFT_BRACKET`, ...). Codes Windows leaves unassigned or reserved have no constant.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyCode(pub u16);

impl KeyCode {
    pub const LBUTTON: Self = Self(0x01);
    pub const RBUTTON: Self = Self(0x02);
    pub const CANCEL: Self = Self(0x03);
    pub const MBUTTON: Self = Self(0x04);
    pub const XBUTTON1: Self = Self(0x05);
    pub const XBUTTON2: Self = Self(0x06);
    pub const BACK: Self = Self(0x08);
    pub const TAB: Self = Self(0x09);
    pub const CLEAR: Self = Self(0x0C);
    pub const RETURN: Self = Self(0x0D);
    pub const SHIFT: Self = Self(0x10);
    pub const CONTROL: Self = Self(0x11);
    pub const MENU: Self = Self(0x12);
    pub const PAUSE: Self = Self(0x13);
    pub const CAPITAL: Self = Self(0x14);
    pub const KANA: Self = Self(0x15);
    pub const HANGUL: Self = Self(0x15);
    pub const IME_ON: Self = Self(0x16);
    pub const JUNJA: Self = Self(0x17);
    pub const FINAL: Self = Self(0x18);
    pub const HANJA: Self = Self(0x19);
    pub const KANJI: Self = Self(0x19);
    pub const IME_OFF: Self = Self(0x1A);
    pub const ESCAPE: Self = Self(0x1B);
    pub const CONVERT: Self = Self(0x1C);
    pub const NONCONVERT: Self = Self(0x1D);
    pub const ACCEPT: Self = Self(0x1E);
    pub const MODECHANGE: Self = Self(0x1F);
    pub const SPACE: Self = Self(0x20);
    pub const PRIOR: Self = Self(0x21);
    pub const NEXT: Self = Self(0x22);
    pub const END: Self = Self(0x23);
    pub const HOME: Self = Self(0x24);
    pub const LEFT: Self = Self(0x25);
    pub const UP: Self = Self(0x26);
    pub const RIGHT: Self = Self(0x27);
    pub const DOWN: Self = Self(0x28);
    pub const SELECT: Self = Self(0x29);
    pub const PRINT: Self = Self(0x2A);
    pub const EXECUTE: Self = Self(0x2B);
    pub const SNAPSHOT: Self = Self(0x2C);
    pub const INSERT: Self = Self(0x2D);
    pub const DELETE: Self = Self(0x2E);
    pub const HELP: Self = Self(0x2F);
    pub const KEY_0: Self = Self(0x30);
    pub const KEY_1: Self = Self(0x31);
    pub const KEY_2: Self = Self(0x32);
    pub const KEY_3: Self = Self(0x33);
    pub const KEY_4: Self = Self(0x34);
    pub const KEY_5: Self = Self(0x35);
    pub const KEY_6: Self = Self(0x36);
    pub const KEY_7: Self = Self(0x37);
    pub const KEY_8: Self = Self(0x38);
    pub const KEY_9: Self = Self(0x39);
    pub const A: Self = Self(0x41);
    pub const B: Self = Self(0x42);
    pub const C: Self = Self(0x43);
    pub const D: Self = Self(0x44);
    pub const E: Self = Self(0x45);
    pub const F: Self = Self(0x46);
    pub const G: Self = Self(0x47);
    pub const H: Self = Self(0x48);
    pub const I: Self = Self(0x49);
    pub const J: Self = Self(0x4A);
    pub const K: Self = Self(0x4B);
    pub const L: Self = Self(0x4C);
    pub const M: Self = Self(0x4D);
    pub const N: Self = Self(0x4E);
    pub const O: Self = Self(0x4F);
    pub const P: Self = Self(0x50);
    pub const Q: Self = Self(0x51);
    pub const R: Self = Self(0x52);
    pub const S: Self = Self(0x53);
    pub const T: Self = Self(0x54);
    pub const U: Self = Self(0x55);
    pub const V: Self = Self(0x56);
    pub const W: Self = Self(0x57);
    pub const X: Self = Self(0x58);
    pub const Y: Self = Self(0x59);
    pub const Z: Self = Self(0x5A);
    pub const LWIN: Self = Self(0x5B);
    pub const RWIN: Self = Self(0x5C);
    pub const APPS: Self = Self(0x5D);
    pub const SLEEP: Self = Self(0x5F);
    pub const NUMPAD0: Self = Self(0x60);
    pub const NUMPAD1: Self = Self(0x61);
    pub const NUMPAD2: Self = Self(0x62);
    pub const NUMPAD3: Self = Self(0x63);
    pub const NUMPAD4: Self = Self(0x64);
    pub const NUMPAD5: Self = Self(0x65);
    pub const NUMPAD6: Self = Self(0x66);
    pub const NUMPAD7: Self = Self(0x67);
    pub const NUMPAD8: Self = Self(0x68);
    pub const NUMPAD9: Self = Self(0x69);
    pub const MULTIPLY: Self = Self(0x6A);
    pub const ADD: Self = Self(0x6B);
    pub const SEPARATOR: Self = Self(0x6C);
    pub const SUBTRACT: Self = Self(0x6D);
    pub const DECIMAL: Self = Self(0x6E);
    pub const DIVIDE: Self = Self(0x6F);
    pub const F1: Self = Self(0x70);
    pub const F2: Self = Self(0x71);
    pub const F3: Self = Self(0x72);
    pub const F4: Self = Self(0x73);
    pub const F5: Self = Self(0x74);
    pub const F6: Self = Self(0x75);
    pub const F7: Self = Self(0x76);
    pub const F8: Self = Self(0x77);
    pub const F9: Self = Self(0x78);
    pub const F10: Self = Self(0x79);
    pub const F11: Self = Self(0x7A);
    pub const F12: Self = Self(0x7B);
    pub const F13: Self = Self(0x7C);
    pub const F14: Self = Self(0x7D);
    pub const F15: Self = Self(0x7E);
    pub const F16: Self = Self(0x7F);
    pub const F17: Self = Self(0x80);
    pub const F18: Self = Self(0x81);
    pub const F19: Self = Self(0x82);
    pub const F20: Self = Self(0x83);
    pub const F21: Self = Self(0x84);
    pub const F22: Self = Self(0x85);
    pub const F23: Self = Self(0x86);
    pub const F24: Self = Self(0x87);
    pub const NAVIGATION_VIEW: Self = Self(0x88);
    pub const NAVIGATION_MENU: Self = Self(0x89);
    pub const NAVIGATION_UP: Self = Self(0x8A);
    pub const NAVIGATION_DOWN: Self = Self(0x8B);
    pub const NAVIGATION_LEFT: Self = Self(0x8C);
    pub const NAVIGATION_RIGHT: Self = Self(0x8D);
    pub const NAVIGATION_ACCEPT: Self = Self(0x8E);
    pub const NAVIGATION_CANCEL: Self = Self(0x8F);
    pub const NUMLOCK: Self = Self(0x90);
    pub const SCROLL: Self = Self(0x91);
    pub const OEM_FJ_JISHO: Self = Self(0x92);
    pub const OEM_FJ_MASSHOU: Self = Self(0x93);
    pub const OEM_FJ_TOUROKU: Self = Self(0x94);
    pub const OEM_FJ_LOYA: Self = Self(0x95);
    pub const OEM_FJ_ROYA: Self = Self(0x96);
    pub const LSHIFT: Self = Self(0xA0);
    pub const RSHIFT: Self = Self(0xA1);
    pub const LCONTROL: Self = Self(0xA2);
    pub const RCONTROL: Self = Self(0xA3);
    pub const LMENU: Self = Self(0xA4);
    pub const RMENU: Self = Self(0xA5);
    pub const BROWSER_BACK: Self = Self(0xA6);
    pub const BROWSER_FORWARD: Self = Self(0xA7);
    pub const BROWSER_REFRESH: Self = Self(0xA8);
    pub const BROWSER_STOP: Self = Self(0xA9);
    pub const BROWSER_SEARCH: Self = Self(0xAA);
    pub const BROWSER_FAVORITES: Self = Self(0xAB);
    pub const BROWSER_HOME: Self = Self(0xAC);
    pub const VOLUME_MUTE: Self = Self(0xAD);
    pub const VOLUME_DOWN: Self = Self(0xAE);
    pub const VOLUME_UP: Self = Self(0xAF);
    pub const MEDIA_NEXT_TRACK: Self = Self(0xB0);
    pub const MEDIA_PREV_TRACK: Self = Self(0xB1);
    pub const MEDIA_STOP: Self = Self(0xB2);
    pub const MEDIA_PLAY_PAUSE: Self = Self(0xB3);
    pub const LAUNCH_MAIL: Self = Self(0xB4);
    pub const LAUNCH_MEDIA_SELECT: Self = Self(0xB5);
    pub const LAUNCH_APP1: Self = Self(0xB6);
    pub const LAUNCH_APP2: Self = Self(0xB7);
    pub const OEM_1: Self = Self(0xBA);
    pub const OEM_PLUS: Self = Self(0xBB);
    pub const OEM_COMMA: Self = Self(0xBC);
    pub const OEM_MINUS: Self = Self(0xBD);
    pub const OEM_PERIOD: Self = Self(0xBE);
    pub const OEM_2: Self = Self(0xBF);
    pub const OEM_3: Self = Self(0xC0);
    pub const GAMEPAD_A: Self = Self(0xC3);
    pub const GAMEPAD_B: Self = Self(0xC4);
    pub const GAMEPAD_X: Self = Self(0xC5);
    pub const GAMEPAD_Y: Self = Self(0xC6);
    pub const GAMEPAD_RIGHT_SHOULDER: Self = Self(0xC7);
    pub const GAMEPAD_LEFT_SHOULDER: Self = Self(0xC8);
    pub const GAMEPAD_LEFT_TRIGGER: Self = Self(0xC9);
    pub const GAMEPAD_RIGHT_TRIGGER: Self = Self(0xCA);
    pub const GAMEPAD_DPAD_UP: Self = Self(0xCB);
    pub const GAMEPAD_DPAD_DOWN: Self = Self(0xCC);
    pub const GAMEPAD_DPAD_LEFT: Self = Self(0xCD);
    pub const GAMEPAD_DPAD_RIGHT: Self = Self(0xCE);
    pub const GAMEPAD_MENU: Self = Self(0xCF);
    pub const GAMEPAD_VIEW: Self = Self(0xD0);
    pub const GAMEPAD_LEFT_THUMBSTICK_BUTTON: Self = Self(0xD1);
    pub const GAMEPAD_RIGHT_THUMBSTICK_BUTTON: Self = Self(0xD2);
    pub const GAMEPAD_LEFT_THUMBSTICK_UP: Self = Self(0xD3);
    pub const GAMEPAD_LEFT_THUMBSTICK_DOWN: Self = Self(0xD4);
    pub const GAMEPAD_LEFT_THUMBSTICK_RIGHT: Self = Self(0xD5);
    pub const GAMEPAD_LEFT_THUMBSTICK_LEFT: Self = Self(0xD6);
    pub const GAMEPAD_RIGHT_THUMBSTICK_UP: Self = Self(0xD7);
    pub const GAMEPAD_RIGHT_THUMBSTICK_DOWN: Self = Self(0xD8);
    pub const GAMEPAD_RIGHT_THUMBSTICK_RIGHT: Self = Self(0xD9);
    pub const GAMEPAD_RIGHT_THUMBSTICK_LEFT: Self = Self(0xDA);
    pub const OEM_4: Self = Self(0xDB);
    pub const OEM_5: Self = Self(0xDC);
    pub const OEM_6: Self = Self(0xDD);
    pub const OEM_7: Self = Self(0xDE);
    pub const OEM_8: Self = Self(0xDF);
    pub const OEM_AX: Self = Self(0xE1);
    pub const OEM_102: Self = Self(0xE2);
    pub const ICO_HELP: Self = Self(0xE3);
    pub const ICO_00: Self = Self(0xE4);
    pub const PROCESSKEY: Self = Self(0xE5);
    pub const ICO_CLEAR: Self = Self(0xE6);
    pub const PACKET: Self = Self(0xE7);
    pub const OEM_RESET: Self = Self(0xE9);
    pub const OEM_JUMP: Self = Self(0xEA);
    pub const OEM_PA1: Self = Self(0xEB);
    pub const OEM_PA2: Self = Self(0xEC);
    pub const OEM_PA3: Self = Self(0xED);
    pub const OEM_WSCTRL: Self = Self(0xEE);
    pub const OEM_CUSEL: Self = Self(0xEF);
    pub const OEM_ATTN: Self = Self(0xF0);
    pub const OEM_FINISH: Self = Self(0xF1);
    pub const OEM_COPY: Self = Self(0xF2);
    pub const OEM_AUTO: Self = Self(0xF3);
    pub const OEM_ENLW: Self = Self(0xF4);
    pub const OEM_BACKTAB: Self = Self(0xF5);
    pub const ATTN: Self = Self(0xF6);
    pub const CRSEL: Self = Self(0xF7);
    pub const EXSEL: Self = Self(0xF8);
    pub const EREOF: Self = Self(0xF9);
    pub const PLAY: Self = Self(0xFA);
    pub const ZOOM: Self = Self(0xFB);
    pub const NONAME: Self = Self(0xFC);
    pub const PA1: Self = Self(0xFD);
    pub const OEM_CLEAR: Self = Self(0xFE);

    pub const BACKSPACE: Self = Self::BACK;
    pub const ENTER: Self = Self::RETURN;
    pub const CTRL: Self = Self::CONTROL;
    pub const ALT: Self = Self::MENU;
    pub const LCTRL: Self = Self::LCONTROL;
    pub const RCTRL: Self = Self::RCONTROL;
    pub const LALT: Self = Self::LMENU;
    pub const RALT: Self = Self::RMENU;
    /// The generic Windows key: `generic` folds `RWIN` to it.
    pub const WIN: Self = Self::LWIN;
    pub const CAPS_LOCK: Self = Self::CAPITAL;
    pub const NUM_LOCK: Self = Self::NUMLOCK;
    pub const SCROLL_LOCK: Self = Self::SCROLL;
    pub const PAGE_UP: Self = Self::PRIOR;
    pub const PAGE_DOWN: Self = Self::NEXT;
    pub const PRINT_SCREEN: Self = Self::SNAPSHOT;
    pub const SEMICOLON: Self = Self::OEM_1;
    pub const EQUALS: Self = Self::OEM_PLUS;
    pub const COMMA: Self = Self::OEM_COMMA;
    pub const MINUS: Self = Self::OEM_MINUS;
    pub const PERIOD: Self = Self::OEM_PERIOD;
    pub const SLASH: Self = Self::OEM_2;
    pub const GRAVE: Self = Self::OEM_3;
    pub const LEFT_BRACKET: Self = Self::OEM_4;
    pub const BACKSLASH: Self = Self::OEM_5;
    pub const RIGHT_BRACKET: Self = Self::OEM_6;
    pub const QUOTE: Self = Self::OEM_7;

    // The key names of the settings text, the same table and spelling as modmanager's chord text.
    const NAMES: [(&'static str, u16); 89] = [
        ("A", 0x41),
        ("B", 0x42),
        ("C", 0x43),
        ("D", 0x44),
        ("E", 0x45),
        ("F", 0x46),
        ("G", 0x47),
        ("H", 0x48),
        ("I", 0x49),
        ("J", 0x4A),
        ("K", 0x4B),
        ("L", 0x4C),
        ("M", 0x4D),
        ("N", 0x4E),
        ("O", 0x4F),
        ("P", 0x50),
        ("Q", 0x51),
        ("R", 0x52),
        ("S", 0x53),
        ("T", 0x54),
        ("U", 0x55),
        ("V", 0x56),
        ("W", 0x57),
        ("X", 0x58),
        ("Y", 0x59),
        ("Z", 0x5A),
        ("0", 0x30),
        ("1", 0x31),
        ("2", 0x32),
        ("3", 0x33),
        ("4", 0x34),
        ("5", 0x35),
        ("6", 0x36),
        ("7", 0x37),
        ("8", 0x38),
        ("9", 0x39),
        ("F1", 0x70),
        ("F2", 0x71),
        ("F3", 0x72),
        ("F4", 0x73),
        ("F5", 0x74),
        ("F6", 0x75),
        ("F7", 0x76),
        ("F8", 0x77),
        ("F9", 0x78),
        ("F10", 0x79),
        ("F11", 0x7A),
        ("F12", 0x7B),
        ("F13", 0x7C),
        ("F14", 0x7D),
        ("F15", 0x7E),
        ("F16", 0x7F),
        ("F17", 0x80),
        ("F18", 0x81),
        ("F19", 0x82),
        ("F20", 0x83),
        ("F21", 0x84),
        ("F22", 0x85),
        ("F23", 0x86),
        ("F24", 0x87),
        ("Space", 0x20),
        ("Enter", 0x0D),
        ("Escape", 0x1B),
        ("Tab", 0x09),
        ("Backspace", 0x08),
        ("Delete", 0x2E),
        ("Insert", 0x2D),
        ("Home", 0x24),
        ("End", 0x23),
        ("PageUp", 0x21),
        ("PageDown", 0x22),
        ("Left", 0x25),
        ("Right", 0x27),
        ("Up", 0x26),
        ("Down", 0x28),
        ("LeftBracket", 0xDB),
        ("RightBracket", 0xDD),
        ("Equals", 0xBB),
        ("Minus", 0xBD),
        ("Comma", 0xBC),
        ("Period", 0xBE),
        ("Slash", 0xBF),
        ("Backslash", 0xDC),
        ("Semicolon", 0xBA),
        ("Quote", 0xDE),
        ("Grave", 0xC0),
        ("Ctrl", 0x11),
        ("Shift", 0x10),
        ("Alt", 0x12),
    ];

    // The settings text of a key with no name: `VK_0x` and the code in hex.
    pub(crate) const HEX_PREFIX: &'static str = "VK_0x";

    /// The generic code of a side key: the left and right Shift, Ctrl and Alt give `SHIFT`, `CTRL`
    /// and `ALT`, and `RWIN` gives `WIN` (`LWIN`). Any other key is itself.
    pub fn generic(self) -> KeyCode {
        match self.0 {
            0xA0 | 0xA1 => Self::SHIFT,
            0xA2 | 0xA3 => Self::CONTROL,
            0xA4 | 0xA5 => Self::MENU,
            0x5C => Self::LWIN,
            _ => self,
        }
    }

    /// The name of the key in the settings text (`"W"`, `"F3"`, `"LeftBracket"`), or `None` for a key
    /// the text writes as `VK_0x..`.
    pub fn name(self) -> Option<&'static str> {
        Self::NAMES.iter().find(|(_, code)| *code == self.0).map(|(name, _)| *name)
    }

    /// The key of a settings-text name, ignoring case. Also reads `VK_0x..` for a code from 0x01 to
    /// 0xFE.
    pub fn from_name(name: &str) -> Option<KeyCode> {
        if let Some((_, code)) = Self::NAMES.iter().find(|(known, _)| known.eq_ignore_ascii_case(name)) {
            return Some(Self(*code));
        }
        let prefix = name.get(..Self::HEX_PREFIX.len())?;
        if !prefix.eq_ignore_ascii_case(Self::HEX_PREFIX) {
            return None;
        }
        let digits = &name[Self::HEX_PREFIX.len()..];
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let code = u16::from_str_radix(digits, 16).ok()?;
        (0x01..=0xFE).contains(&code).then_some(Self(code))
    }
}
