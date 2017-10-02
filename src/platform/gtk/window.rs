/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use gtk::WindowExt;
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
        let text = state.browsers.iter().enumerate().fold("|".to_owned(), |f, (idx, b)| {
            let title = b.title.as_ref().and_then(|t| {
                if t.is_empty() { None } else { Some(t) }
            }).map_or("No Title", |t| t.as_str());
            let selected = if Some(idx) == state.current_browser_index { '>' } else { ' ' };
            let loading = if b.is_loading { '*' } else { ' ' };
            format!("{} {} {:15.15} {}|", f, selected, title, loading)
        });

        let mut windows = self.windows.borrow_mut();
        windows[WINDOW_ID].gtk_window.set_title(&text);

        if state.urlbar_focused {
            let url = format!("{}", state.browsers[state.current_browser_index.unwrap()]
                              .url.as_ref().map_or("", |t| t.as_str()));
            /*match tinyfiledialogs::input_box("Search or type URL", "Search or type URL", &url) {
                Some(input) => {
                    let win = windows.get_mut(&WINDOW_ID).unwrap();
                    win.window_events.push(WindowEvent::DoCommand(WindowCommand::Load(input)));
                }
                None => { },
            }*/
            windows[WINDOW_ID].window_events.push(WindowEvent::UrlbarFocusChanged(false));
        }
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


