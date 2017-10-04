/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod app;
mod utils;
mod view;
mod window;

use std::cell::Cell;
use std::rc::Rc;

use gtk;
use gtk::GLArea;
use servo::EventLoopWaker;
use traits::view::*;
use traits::window::{WindowCommand, WindowEvent};

pub use self::app::App;
pub use self::view::View;
pub use self::window::Window;

pub struct GtkWindow {
    gl: Rc<gl::Gl>,
    gtk_window: gtk::Window,
    gl_area: GLArea,
    event_loop_waker: Box<EventLoopWaker>,
    key_modifiers: Cell<KeyModifiers>,
    last_pressed_key: Cell<Option<Key>>,
    mouse_coordinate: (i32, i32),
    tabs: gtk::Notebook,
    view_events: Vec<ViewEvent>,
    window_events: Vec<WindowEvent>,
}

impl GtkWindow {

    pub fn glutin_event_to_command(&self/*, event: &glutin::WindowEvent*/) -> Option<WindowCommand> {
        None
        /*match *event {
            glutin::WindowEvent::KeyboardInput{ input: glutin::KeyboardInput {
                state: glutin::ElementState::Pressed,
                virtual_keycode,
                modifiers,
                ..
            }, ..} => {
                match (virtual_keycode, utils::cmd_or_ctrl(modifiers), modifiers.ctrl, modifiers.shift) {
                    (Some(glutin::VirtualKeyCode::R), true, _, _) => Some(WindowCommand::Reload),
                    (Some(glutin::VirtualKeyCode::Left), true, _, _) => Some(WindowCommand::NavigateBack),
                    (Some(glutin::VirtualKeyCode::Right), true, _, _) => Some(WindowCommand::NavigateForward),
                    (Some(glutin::VirtualKeyCode::L), true, _, _) => Some(WindowCommand::OpenLocation),
                    (Some(glutin::VirtualKeyCode::Equals), true, _, _) => Some(WindowCommand::ZoomIn),
                    (Some(glutin::VirtualKeyCode::Minus), true, _, _) => Some(WindowCommand::ZoomOut),
                    (Some(glutin::VirtualKeyCode::Key0), true, _, _) => Some(WindowCommand::ZoomToActualSize),
                    (Some(glutin::VirtualKeyCode::T), true, _, _) => Some(WindowCommand::NewTab),
                    (Some(glutin::VirtualKeyCode::W), true, _, _) => Some(WindowCommand::CloseTab),
                    (Some(glutin::VirtualKeyCode::Tab), _, true, false) => Some(WindowCommand::NextTab),
                    (Some(glutin::VirtualKeyCode::Tab), _, true, true) => Some(WindowCommand::PrevTab),
                    (Some(glutin::VirtualKeyCode::Key1), true, _, _) => Some(WindowCommand::SelectTab(0)),
                    (Some(glutin::VirtualKeyCode::Key2), true, _, _) => Some(WindowCommand::SelectTab(1)),
                    (Some(glutin::VirtualKeyCode::Key3), true, _, _) => Some(WindowCommand::SelectTab(2)),
                    (Some(glutin::VirtualKeyCode::Key4), true, _, _) => Some(WindowCommand::SelectTab(3)),
                    (Some(glutin::VirtualKeyCode::Key5), true, _, _) => Some(WindowCommand::SelectTab(4)),
                    (Some(glutin::VirtualKeyCode::Key6), true, _, _) => Some(WindowCommand::SelectTab(5)),
                    (Some(glutin::VirtualKeyCode::Key7), true, _, _) => Some(WindowCommand::SelectTab(6)),
                    (Some(glutin::VirtualKeyCode::Key8), true, _, _) => Some(WindowCommand::SelectTab(7)),
                    (Some(glutin::VirtualKeyCode::Key9), true, _, _) => Some(WindowCommand::SelectTab(8)),
                    _ => None
                }
            }
            _ => None
        }*/
    }

    pub fn glutin_event_to_view_event(&mut self/*, event: &glutin::WindowEvent*/) -> Option<ViewEvent> {
        /*match *event {
            glutin::WindowEvent::MouseInput{state, button: glutin::MouseButton::Left, ..} => {
                let state = match state {
                    glutin::ElementState::Released => ElementState::Released,
                    glutin::ElementState::Pressed => ElementState::Pressed,
                };
                Some(ViewEvent::MouseInput(state, MouseButton::Left, self.mouse_coordinate.0, self.mouse_coordinate.1))
            }
            glutin::WindowEvent::ReceivedCharacter(ch) => {

                let mods = self.key_modifiers.get();

                // FIXME: cleanup
                let event = if let Some(last_pressed_key) = self.last_pressed_key.get() {
                    Some(ViewEvent::KeyEvent(Some(ch), last_pressed_key, KeyState::Pressed, mods))
                } else {
                    if !ch.is_control() {
                        match utils::char_to_script_key(ch) {
                            Some(key) => {
                                Some(ViewEvent::KeyEvent(Some(ch),
                                    key,
                                    KeyState::Pressed,
                                    mods))
                            }
                            None => None
                        }
                    } else {
                        None
                    }
                };
                self.last_pressed_key.set(None);
                event
            }
            glutin::WindowEvent::KeyboardInput{ input: glutin::KeyboardInput {
                state, virtual_keycode: Some(virtual_keycode), modifiers, ..}, ..
            } => {

                let mut servo_mods = KeyModifiers::empty();
                if modifiers.shift { servo_mods.insert(SHIFT); }
                if modifiers.ctrl { servo_mods.insert(CONTROL); }
                if modifiers.alt { servo_mods.insert(ALT); }
                if modifiers.logo { servo_mods.insert(SUPER); }

                self.key_modifiers.set(servo_mods);

                if let Ok(key) = utils::gtk_key_to_script_key(virtual_keycode) {
                    let state = match state {
                        glutin::ElementState::Pressed => KeyState::Pressed,
                        glutin::ElementState::Released => KeyState::Released,
                    };
                    if state == KeyState::Pressed {
                        if utils::is_printable(virtual_keycode) {
                            self.last_pressed_key.set(Some(key));
                        }
                    }
                    Some(ViewEvent::KeyEvent(None, key, state, self.key_modifiers.get()))
                } else {
                    None
                }
            }
        }*/
        None
    }
}

