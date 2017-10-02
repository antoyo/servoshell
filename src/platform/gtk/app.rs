/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::env;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use epoxy;
use gdk;
use gdk::{POINTER_MOTION_MASK, SCROLL_MASK};
use glib_itc::{Receiver, Sender, channel};
use gtk;
use gtk::{
    ContainerExt,
    Entry,
    EntryExt,
    GLArea,
    GLAreaExt,
    Image,
    Inhibit,
    SeparatorToolItem,
    Toolbar,
    ToolButton,
    ToolItem,
    ToolItemExt,
    WidgetExt,
    WindowType,
};
use gtk::Orientation::Vertical;
use platform::{GtkWindow, Window};
use servo:: EventLoopWaker;
use shared_library::dynamic_library::DynamicLibrary;
use state::AppState;
use super::utils;
use traits::app::{AppEvent, AppMethods};
use traits::view::{gl, KeyModifiers, MouseScrollDelta, TouchPhase, ViewEvent};
use traits::window::{WindowCommand, WindowEvent, WindowMethods};

// TODO: remove.
const WINDOW_ID: usize = 0;

pub struct GtkEventLoopWaker {
    tx: Arc<Mutex<Sender>>,
}

impl EventLoopWaker for GtkEventLoopWaker {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(GtkEventLoopWaker {
            tx: self.tx.clone(),
        })
    }

    fn wake(&self) {
        self.tx.lock().unwrap().send();
    }
}

pub struct App {
    event_loop_waker: Box<EventLoopWaker>,
    rx: Option<Receiver>,
    windows: Rc<RefCell<Vec<GtkWindow>>>,
}

impl App {

    fn should_exit(&self/*, event: &glutin::WindowEvent*/) -> bool {
        // Exit if window is closed or if Cmd/Ctrl Q
        /*match *event {
            glutin::WindowEvent::Closed => {
                return true
            },
            _ => { }
        }

        if let glutin::WindowEvent::KeyboardInput {
            device_id: _,
            input: glutin::KeyboardInput {
                state: glutin::ElementState::Pressed,
                scancode: _,
                virtual_keycode: Some(glutin::VirtualKeyCode::Q),
                modifiers,
            }
        } = *event {
            if utils::cmd_or_ctrl(modifiers) {
                return true
            }
        }*/
        false
    }

    pub fn take_receiver(&mut self) -> Option<Receiver> {
        self.rx.take()
    }
}

impl AppMethods for App {
    fn new<'a>() -> Result<App, &'a str> {
        let (tx, rx) = channel();
        let event_loop_waker = Box::new(GtkEventLoopWaker {
            tx: Arc::new(Mutex::new(tx)),
        });
        let windows = Rc::new(RefCell::new(vec![]));
        Ok(App {
            windows,
            event_loop_waker,
            rx: Some(rx),
        })
    }

    fn get_resources_path() -> Option<PathBuf> {
        // Try current directory. Used for example with "cargo run"
        let p = env::current_dir().unwrap();
        if p.join("servo_resources/").exists() {
            return Some(p.join("servo_resources/"));
        }

        // Maybe in /resources/
        let p = p.join("resources").join("servo_resources");
        if p.exists() {
            return Some(p);
        }

        // Maybe we run from an app bundle
        let p = env::current_exe().unwrap();
        let p = p.parent().unwrap();
        let p = p.parent().unwrap().join("Resources");

        if p.join("servo_resources/").exists() {
            return Some(p.join("servo_resources/"));
        }

        None
    }

    fn render(&self, state: &AppState) {
        let cursor = utils::servo_cursor_to_gtk_cursor(state.cursor);
        let windows = self.windows.borrow();
        for window in windows.iter() {
            if let Some(window) = window.gtk_window.get_window() {
                gdk::WindowExt::set_cursor(&window, &cursor);
            }
        };
    }

    fn get_events(&self) -> Vec<AppEvent> {
        vec![]
    }

    fn new_window<'a>(&self) -> Result<Box<WindowMethods>, &'a str> {

        #[cfg(target_os = "windows")]
        let factor = utils::windows_hidpi_factor();
        #[cfg(not(target_os = "windows"))]
        let factor = 1.0f32;

        let gtk_window = gtk::Window::new(WindowType::Toplevel);
        gtk_window.set_size_request(1024 * factor as i32, 768 * factor as i32);

        let windows = self.windows.clone();
        gtk_window.connect_scroll_event(move |_, event| {
            let (dx, dy) = event.get_delta();
            let dy = dy /* * -38.0 */; // TODO: remove multiplication.
            let delta = MouseScrollDelta::PixelDelta(dx as f32, dy as f32);
            let phase = TouchPhase::Moved;
            let mut windows = windows.borrow_mut();
            let window: &mut GtkWindow = windows.get_mut(WINDOW_ID).unwrap();
            window.view_events.push(ViewEvent::MouseWheel(delta, phase));
            Inhibit(false)
        });

        let vbox = gtk::Box::new(Vertical, 0);
        gtk_window.add(&vbox);

        let toolbar = Toolbar::new();
        vbox.add(&toolbar);

        let previous_button = ToolButton::new(&icon("go-previous"), None);
        toolbar.add(&previous_button);

        let next_button = ToolButton::new(&icon("go-next"), None);
        toolbar.add(&next_button);

        toolbar.add(&SeparatorToolItem::new());

        let reload_button = ToolButton::new(&icon("view-refresh"), None);
        toolbar.add(&reload_button);

        toolbar.add(&SeparatorToolItem::new());

        let url_entry = Entry::new();
        let url_tool_item = ToolItem::new();
        url_tool_item.set_expand(true);
        url_tool_item.add(&url_entry);
        toolbar.add(&url_tool_item);

        let windows = self.windows.clone();
        url_entry.connect_activate(move |entry| {
            let mut windows = windows.borrow_mut();
            let win: &mut GtkWindow = windows.get_mut(WINDOW_ID).unwrap();
            println!("Loading: {}", entry.get_text().unwrap());
            win.window_events.push(WindowEvent::DoCommand(WindowCommand::Load(entry.get_text().unwrap())));
            //callback()
        });

        let gl_area = GLArea::new();
        gl_area.set_auto_render(false);
        gl_area.set_has_depth_buffer(true);
        gl_area.add_events((POINTER_MOTION_MASK | SCROLL_MASK).bits() as i32);
        gl_area.set_vexpand(true);
        vbox.add(&gl_area);

        let windows = self.windows.clone();
        gl_area.connect_resize(move |_, _, _| {
            let mut windows = windows.borrow_mut();
            let window: &mut GtkWindow = windows.get_mut(WINDOW_ID).unwrap();
            window.view_events.push(ViewEvent::GeometryDidChange);
        });

        gtk_window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        gtk_window.show_all();

        gl_area.make_current();

        epoxy::load_with(|s| {
            unsafe {
                match DynamicLibrary::open(None).unwrap().symbol(s) {
                    Ok(v) => v,
                    Err(_) => ptr::null(),
                }
            }
        });
        let gl = unsafe {
            gl::GlFns::load_with(epoxy::get_proc_addr)
        };

        gl.clear_color(1.0, 1.0, 1.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();

        self.windows.borrow_mut().push(GtkWindow {
            gl,
            gtk_window,
            gl_area,
            event_loop_waker: self.event_loop_waker.clone(),
            key_modifiers: Cell::new(KeyModifiers::empty()),
            last_pressed_key: Cell::new(None),
            view_events: vec![],
            window_events: vec![],
            mouse_coordinate: (0, 0),
        });

        Ok(Box::new(Window::new(self.windows.clone())))
    }

    fn run<T>(&self, mut callback: T) where T: FnMut() {
        let windows = self.windows.clone();
        gtk::main();
        /*self.event_loop.borrow_mut().run_forever(|e| {
            let mut call_callback = false;
            match e {
                glutin::Event::WindowEvent {event, window_id} => {
                    if self.should_exit(&event) {
                        return glutin::ControlFlow::Break;
                    }
                    let mut windows = self.windows.borrow_mut();
                    match windows.get_mut(&window_id) {
                        Some(window) => {
                            match (*window).glutin_event_to_command(&event) {
                                Some(command) => {
                                    window.window_events.push(WindowEvent::DoCommand(command));
                                    call_callback = true;
                                }
                                None => {
                                    match (*window).glutin_event_to_view_event(&event) {
                                        Some(event) => {
                                            window.view_events.push(event);
                                            call_callback = true;
                                        }
                                        None => {
                                            warn!("Got unknown glutin event: {:?}", event);
                                        }
                                    }
                                }
                            }
                        },
                        None => {
                            warn!("Unexpected event ({:?} for unknown Windows ({:?})", event, window_id);
                        }
                    }
                },
                glutin::Event::Awakened => {
                    let mut windows = self.windows.borrow_mut();
                    for (_, window) in windows.iter_mut() {
                        window.window_events.push(WindowEvent::EventLoopAwaken);
                    };
                    call_callback = true;
                }
                _ => { }
            }
            if call_callback {
                callback();
            }
            glutin::ControlFlow::Continue
        });*/
    }
}

fn icon(name: &str) -> Image {
    Image::new_from_file(format!("images/{}.png", name))
}
