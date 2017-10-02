/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{GLAreaExt, WidgetExt, WindowExt};
use super::GtkWindow;
use traits::view::*;

// TODO: remove.
const WINDOW_ID: usize = 0;

pub struct View {
    windows: Rc<RefCell<Vec<GtkWindow>>>,
}

impl View {
    pub fn new(windows: Rc<RefCell<Vec<GtkWindow>>>) -> View {
        View { windows }
    }

    #[cfg(not(target_os = "windows"))]
    fn hidpi_factor(&self) -> f32 {
        let windows = self.windows.borrow();
        let win = &windows[WINDOW_ID];
        win.gtk_window.get_scale_factor() as f32
    }

    #[cfg(target_os = "windows")]
    fn hidpi_factor(&self) -> f32 {
        super::utils::windows_hidpi_factor()
    }
}

impl ViewMethods for View {
    fn get_geometry(&self) -> DrawableGeometry {
        let windows = self.windows.borrow();
        let win = &windows[WINDOW_ID];
        let (width, height) = win.gtk_window.get_size();
        let (mut width, mut height) = (width as u32, height as u32);

        #[cfg(target_os = "windows")]
        let factor = super::utils::windows_hidpi_factor();
        #[cfg(not(target_os = "windows"))]
        let factor = 1.0f32;

        width /= factor as u32;
        height /= factor as u32;

        DrawableGeometry {
            view_size: (width, height),
            margins: (0, 0, 0, 0),
            position: win.gtk_window.get_position(),
            hidpi_factor: self.hidpi_factor(),
        }
    }

    fn update_drawable(&self) {
        let windows = self.windows.borrow();
        let win = &windows[WINDOW_ID];
        let (w, h) = win.gtk_window.get_size();
        win.gtk_window.resize(w, h);
    }

    // FIXME: should be controlled by state
    fn enter_fullscreen(&self) {
    }

    // FIXME: should be controlled by state
    fn exit_fullscreen(&self) {
        // FIXME
        //self.windows.borrow().[WINDOW_ID].gtk_window.swap_buffers().unwrap();
    }

    fn set_live_resize_callback(&self, _callback: &FnMut()) {
        // FIXME
    }

    fn gl(&self) -> Rc<gl::Gl> {
        self.windows.borrow()[WINDOW_ID].gl.clone()
    }

    fn get_events(&self) -> Vec<ViewEvent> {
        let mut windows = self.windows.borrow_mut();
        let win = &mut windows[WINDOW_ID];
        let events = win.view_events.drain(..).collect();
        events
    }

    fn prepare(&self) {
        let mut windows = self.windows.borrow_mut();
        let win = &mut windows[WINDOW_ID];
        win.gl_area.make_current();
    }

    fn swap_buffers(&self) {
        let mut windows = self.windows.borrow_mut();
        let win = &mut windows[WINDOW_ID];
        win.gl_area.queue_render();
    }
}
