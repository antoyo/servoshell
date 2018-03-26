/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use gdk::{POINTER_MOTION_MASK, SCROLL_MASK};
use gtk;
use gtk::{
    ContainerExt,
    GLArea,
    GLAreaExt,
    Label,
    NotebookExt,
    NotebookExtManual,
    WidgetExt,
    WindowExt,
};
use logs::ShellLog;
use platform::View;
use servo::EventLoopWaker;
use state::WindowState;
use super::GtkWindow;
use traits::view::ViewMethods;
use traits::window::{WindowCommand, WindowEvent, WindowMethods};

// TODO: remove.
const WINDOW_ID: usize = 0;

pub struct Window {
    windows: Rc<RefCell<Vec<GtkWindow>>>,
}

impl Window {
    pub fn new(windows: Rc<RefCell<Vec<GtkWindow>>>) -> Window {
        Window { windows }
    }
}

impl WindowMethods for Window {
    fn render(&self, state: &WindowState) {
        // FIXME: mut WindowState
        /*let text = state.browsers.iter().enumerate().fold("|".to_owned(), |f, (idx, b)| {
            let title = b.title.as_ref().and_then(|t| {
                if t.is_empty() { None } else { Some(t) }
            }).map_or("No Title", |t| t.as_str());
            let selected = if Some(idx) == state.current_browser_index { '>' } else { ' ' };
            let loading = if b.is_loading { '*' } else { ' ' };
            format!("{} {} {:15.15} {}|", f, selected, title, loading)
        });*/

        let tabs;
        let visual_count;

        {
            let mut windows = self.windows.borrow_mut();

            let state_count = state.browsers.len();
            {
                let window = &windows[WINDOW_ID];

                if state_count > 1 {
                    window.tabs.set_show_tabs(true);
                }
                else {
                    //window.tabs.set_show_tabs(false); // TODO: uncomment.
                }
                for i in 0..state_count {
                    if let Some(ref title) = state.browsers[i].title {
                        if Some(i) == state.current_browser_index {
                            window.gtk_window.set_title(title);
                        }
                        if let Some(tab) = window.tabs.get_nth_page(Some(i as u32)) {
                            window.tabs.set_tab_label_text(&tab, title);
                        }
                    }
                }

                tabs = window.tabs.clone();
                visual_count = window.tabs.get_n_pages() as usize;
                if state_count == visual_count + 1 {
                    let gl_area = GLArea::new();
                    gl_area.set_auto_render(false);
                    gl_area.set_has_depth_buffer(true);
                    gl_area.add_events((POINTER_MOTION_MASK | SCROLL_MASK).bits() as i32);
                    gl_area.set_vexpand(true);
                    window.tabs.add(&gl_area);
                    gl_area.show();
                    gl_area.make_current();
                    mem::forget(gl_area);
                }
            }

            if state.urlbar_focused {
                let url = format!("{}", state.browsers[state.current_browser_index.unwrap()]
                                  .url.as_ref().map_or("", |t| t.as_str()));
                windows[WINDOW_ID].window_events.push(WindowEvent::UrlbarFocusChanged(false));
            }
        }

        tabs.set_current_page(Some(visual_count as u32));
    }

    fn new_view(&self) -> Result<Rc<ViewMethods>, &'static str> {
        Ok(Rc::new(View::new(self.windows.clone())))
    }

    fn new_event_loop_waker(&self) -> Box<EventLoopWaker> {
        let windows = self.windows.borrow();
        windows[WINDOW_ID].event_loop_waker.clone()
    }

    fn get_events(&self) -> Vec<WindowEvent> {
        let mut windows = self.windows.borrow_mut();
        let win = &mut windows[WINDOW_ID];
        let events = win.window_events.drain(..).collect();
        events
    }

    fn append_logs(&self, _logs: &Vec<ShellLog>) {
    }
}


