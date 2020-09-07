use glium::glutin::event::{self, *};
use vector2math::*;

use crate::Vec2;

pub use event::ElementState as ButtonState;
pub use event::ModifiersState as Modifiers;
pub use event::MouseButton;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    MouseAbsolute(Vec2),
    MouseRelative(Vec2),
    MouseButton {
        button: MouseButton,
        state: ButtonState,
    },
    Key {
        key: Key,
        scancode: u32,
        state: ButtonState,
    },
    Resize(Vec2),
    Move(Vec2),
    Focus(bool),
    Scroll(Vec2),
    CloseRequest,
}

impl Event {
    pub(crate) fn from_glutin(event: event::Event<()>, tracker: &mut StateTracker) -> Two<Self> {
        let window_event = if let event::Event::WindowEvent { event, .. } = event {
            event
        } else {
            return Two::none();
        };
        match window_event {
            WindowEvent::CloseRequested => Event::CloseRequest.into(),
            WindowEvent::Resized(size) => {
                let size = [size.width as f32, size.height as f32];
                tracker.size = size;
                Event::Resize(size).into()
            }
            WindowEvent::Moved(size) => Event::Move([size.x as f32, size.y as f32]).into(),
            WindowEvent::Focused(foc) => Event::Focus(foc).into(),
            WindowEvent::CursorMoved { position, .. } => {
                let pos = [position.x as f32, position.y as f32];
                let two = Two::two(
                    Event::MouseAbsolute(pos),
                    Event::MouseRelative(pos.sub(tracker.mouse_pos)),
                );
                tracker.mouse_pos = pos;
                two
            }
            WindowEvent::MouseInput { button, state, .. } => {
                Event::MouseButton { button, state }.into()
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => Event::Scroll([x, y]).into(),
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(pos),
                ..
            } => Event::Scroll([pos.x as f32, pos.y as f32]).into(),
            WindowEvent::ModifiersChanged(modifiers) => {
                tracker.modifiers = modifiers;
                Two::none()
            }
            WindowEvent::KeyboardInput { input, .. } => {
                let key = input
                    .virtual_keycode
                    .map(Key::from_glutin)
                    .unwrap_or(Key::Unknown);
                match input.state {
                    ButtonState::Pressed => tracker.keys.add(key),
                    ButtonState::Released => tracker.keys.remove(key),
                }
                Event::Key {
                    key,
                    scancode: input.scancode,
                    state: input.state,
                }
                .into()
            }
            _ => Two::none(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateTracker {
    pub mouse_pos: Vec2,
    pub modifiers: Modifiers,
    pub keys: Bits<Key>,
    pub size: Vec2,
}

impl StateTracker {
    pub fn new(size: Vec2) -> Self {
        StateTracker {
            mouse_pos: [0.0; 2],
            modifiers: Modifiers::default(),
            keys: Bits::default(),
            size,
        }
    }
}

pub(crate) struct Two<T>(Option<T>, Option<T>);

impl<T> Two<T> {
    pub const fn none() -> Self {
        Two(None, None)
    }
    pub const fn one(item: T) -> Self {
        Two(Some(item), None)
    }
    pub const fn two(one: T, two: T) -> Self {
        Two(Some(one), Some(two))
    }
}

impl<T> From<T> for Two<T> {
    fn from(item: T) -> Self {
        Two::one(item)
    }
}

impl<T> Iterator for Two<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().or_else(|| self.1.take())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bits<T>(u128, std::marker::PhantomData<T>);

impl<T> Default for Bits<T> {
    fn default() -> Self {
        Bits(0, std::marker::PhantomData)
    }
}

impl<T> Bits<T>
where
    T: From<u128> + Into<u128>,
{
    pub fn add(&mut self, val: T) {
        self.0 |= val.into();
    }
    pub fn remove(&mut self, val: T) {
        self.0 &= !val.into();
    }
    pub fn get(&self, val: T) -> bool {
        (self.0 & val.into()).count_ones() > 0
    }
    pub fn diff(&self, start: T, end: T) -> f32 {
        self.get(end) as i8 as f32 - self.get(start) as i8 as f32
    }
}

macro_rules! keys {
    ($(($key:ident, $glutinkey:ident),)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Key {
            $($key,)*
            Unknown
        }

        impl Key {
            fn from_glutin(key: event::VirtualKeyCode) -> Self {
                match key {
                    $(event::VirtualKeyCode::$glutinkey => Key::$key),*
                }
            }
        }

        impl From<Key> for u128 {
            fn from(key: Key) -> Self {
                1 << key as u128
            }
        }
        impl From<u128> for Key {
            fn from(u: u128) -> Self {
                unsafe { std::mem::transmute(127 - u.leading_zeros() as u8) }
            }
        }
    };
}

keys!(
    (Num1, Key1),
    (Num2, Key2),
    (Num3, Key3),
    (Num4, Key4),
    (Num5, Key5),
    (Num6, Key6),
    (Num7, Key7),
    (Num8, Key8),
    (Num9, Key9),
    (Num0, Key0),
    (A, A),
    (B, B),
    (C, C),
    (D, D),
    (E, E),
    (F, F),
    (G, G),
    (H, H),
    (I, I),
    (J, J),
    (K, K),
    (L, L),
    (M, M),
    (N, N),
    (O, O),
    (P, P),
    (Q, Q),
    (R, R),
    (S, S),
    (T, T),
    (U, U),
    (V, V),
    (W, W),
    (X, X),
    (Y, Y),
    (Z, Z),
    (Escape, Escape),
    (F1, F1),
    (F2, F2),
    (F3, F3),
    (F4, F4),
    (F5, F5),
    (F6, F6),
    (F7, F7),
    (F8, F8),
    (F9, F9),
    (F10, F10),
    (F11, F11),
    (F12, F12),
    (F13, F13),
    (F14, F14),
    (F15, F15),
    (F16, F16),
    (F17, F17),
    (F18, F18),
    (F19, F19),
    (F20, F20),
    (F21, F21),
    (F22, F22),
    (F23, F23),
    (F24, F24),
    (Snapshot, Snapshot),
    (Scroll, Scroll),
    (Pause, Pause),
    (Insert, Insert),
    (Home, Home),
    (Delete, Delete),
    (End, End),
    (PageDown, PageDown),
    (PageUp, PageUp),
    (Left, Left),
    (Up, Up),
    (Right, Right),
    (Down, Down),
    (Back, Back),
    (Enter, Return),
    (Space, Space),
    (Compose, Compose),
    (Caret, Caret),
    (Numlock, Numlock),
    (Numpad0, Numpad0),
    (Numpad1, Numpad1),
    (Numpad2, Numpad2),
    (Numpad3, Numpad3),
    (Numpad4, Numpad4),
    (Numpad5, Numpad5),
    (Numpad6, Numpad6),
    (Numpad7, Numpad7),
    (Numpad8, Numpad8),
    (Numpad9, Numpad9),
    (AbntC1, AbntC1),
    (AbntC2, AbntC2),
    (Add, Add),
    (Apostrophe, Apostrophe),
    (Apps, Apps),
    (At, At),
    (Ax, Ax),
    (Backslash, Backslash),
    (Calculator, Calculator),
    (Capital, Capital),
    (Colon, Colon),
    (Comma, Comma),
    (Convert, Convert),
    (Decimal, Decimal),
    (Divide, Divide),
    (Equals, Equals),
    (Grave, Grave),
    (Kana, Kana),
    (Kanji, Kanji),
    (LAlt, LAlt),
    (LBracket, LBracket),
    (LControl, LControl),
    (LShift, LShift),
    (LWin, LWin),
    (Mail, Mail),
    (MediaSelect, MediaSelect),
    (MediaStop, MediaStop),
    (Minus, Minus),
    (Multiply, Multiply),
    (Mute, Mute),
    (MyComputer, MyComputer),
    (NavigateForward, NavigateForward),
    (NavigateBackward, NavigateBackward),
    (NextTrack, NextTrack),
    (NoConvert, NoConvert),
    (NumpadComma, NumpadComma),
    (NumpadEnter, NumpadEnter),
    (NumpadEquals, NumpadEquals),
    (OEM102, OEM102),
    (Period, Period),
    (PlayPause, PlayPause),
    (Power, Power),
    (PrevTrack, PrevTrack),
    (RAlt, RAlt),
    (RBracket, RBracket),
    (RControl, RControl),
    (RShift, RShift),
    (RWin, RWin),
    (Semicolon, Semicolon),
    (Slash, Slash),
    (Sleep, Sleep),
    (Stop, Stop),
    (Subtract, Subtract),
    (Sysrq, Sysrq),
    (Tab, Tab),
    (Underline, Underline),
    (Unlabeled, Unlabeled),
    (VolumeDown, VolumeDown),
    (VolumeUp, VolumeUp),
    (Wake, Wake),
    (WebBack, WebBack),
    (WebFavorites, WebFavorites),
    (WebForward, WebForward),
    (WebHome, WebHome),
    (WebRefresh, WebRefresh),
    (WebSearch, WebSearch),
    (WebStop, WebStop),
    (Yen, Yen),
    (Copy, Copy),
    (Paste, Paste),
    (Cut, Cut),
);
