/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate servo;

use self::servo::config::servo_version;
use self::servo::servo_config::opts;
use self::servo::servo_config::resource_files::set_resources_path;
use self::servo::compositing::windowing::{MouseWindowEvent, WindowMethods, WindowEvent};
use self::servo::msg::constellation_msg::{self, Key, TraversalDirection};
use self::servo::servo_geometry::DeviceIndependentPixel;
use self::servo::euclid::{Point2D, ScaleFactor, Size2D, TypedPoint2D, TypedRect, TypedSize2D, TypedVector2D};
use self::servo::ipc_channel::ipc;
use self::servo::script_traits::{LoadData, MouseButton, TouchEventType};
use self::servo::style_traits::DevicePixel;
use self::servo::net_traits::net_error_list::NetError;
use self::servo::webrender_api;
use gleam::gl;
use state::BrowserState;
use platform;

pub use self::servo::BrowserId;
pub use self::servo::compositing::compositor_thread::EventLoopWaker;
pub use self::servo::style_traits::cursor::Cursor as ServoCursor;
pub use self::servo::servo_url::ServoUrl;

use view;
use view::DrawableGeometry;

use std::rc::Rc;
use std::cell::{Cell, RefCell};

pub enum ServoEvent {
    SetWindowInnerSize(u32, u32),
    SetWindowPosition(i32, i32),
    SetFullScreenState(bool),
    TitleChanged(BrowserId, Option<String>),
    StatusChanged(Option<String>),
    LoadStart(BrowserId),
    LoadEnd(BrowserId),
    HeadParsed(BrowserId),
    HistoryChanged(BrowserId, Vec<LoadData>, usize),
    CursorChanged(ServoCursor),
    FaviconChanged(BrowserId, ServoUrl),
    Key(Option<char>, Key, constellation_msg::KeyModifiers),
}

pub struct Servo {
    // FIXME: it's annoying that event for servo are named "WindowEvent"
    events_for_servo: RefCell<Vec<WindowEvent>>,
    servo: RefCell<servo::Servo<ServoCallbacks>>,
    callbacks: Rc<ServoCallbacks>,
}

impl Servo {

    pub fn configure() -> Result<(), &'static str> {

        let path = match platform::get_resources_path() {
            Some(path) => path.join("servo_resources"),
            None => panic!("Can't find resources directory"),
        };

        let path = path.to_str().unwrap().to_string();
        set_resources_path(Some(path));

        opts::set_defaults(opts::default_opts());
        Ok(())
    }

    pub fn version(&self) -> String {
        servo_version()
    }

    pub fn new(geometry: DrawableGeometry, view: Rc<view::View>, waker: Box<EventLoopWaker>) -> Servo {
        // FIXME: url not used here

        let callbacks = Rc::new(ServoCallbacks {
            event_queue: RefCell::new(Vec::new()),
            geometry: Cell::new(geometry),
            waker: waker,
            view: view.clone(),
        });

        let servo = servo::Servo::new(callbacks.clone());

        Servo {
            events_for_servo: RefCell::new(Vec::new()),
            servo: RefCell::new(servo),
            callbacks: callbacks,
        }
    }

    pub fn create_browser(&self, url: &str) -> BrowserState {

        // FIXME
        let url = ServoUrl::parse(url).unwrap();

        let (sender, receiver) = ipc::channel().unwrap();
        self.servo.borrow_mut().handle_events(vec![WindowEvent::NewBrowser(url, sender)]);
        let id = receiver.recv().unwrap();
        self.select_browser(id);

        BrowserState {
            id: id,
            last_mouse_point: (0, 0),
            last_mouse_down_point: (0, 0),
            last_mouse_down_button: None,
            zoom: 1.0,
            url: None,
            title: None,
            user_input: None,
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
            show_fragment_borders: false,
            parallel_display_list_building: false,
            show_parallel_layout: false,
            convert_mouse_to_touch: false,
            show_webrender_stats: false,
            show_tiles_borders: false,
        }
    }

    pub fn select_browser(&self, id: BrowserId) {
        self.servo.borrow_mut().handle_events(vec![WindowEvent::SelectBrowser(id)]);
    }

    pub fn get_events(&self) -> Vec<ServoEvent> {
        self.callbacks.get_events()
    }

    pub fn reload(&self, id: BrowserId) {
        let event = WindowEvent::Reload(id);
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn go_back(&self, id: BrowserId) {
        let event = WindowEvent::Navigation(id, TraversalDirection::Back(1));
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn go_forward(&self, id: BrowserId) {
        let event = WindowEvent::Navigation(id, TraversalDirection::Forward(1));
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn load_url(&self, id: BrowserId, url: ServoUrl) {
        let event = WindowEvent::LoadUrl(id, url);
        self.events_for_servo.borrow_mut().push(event);
    }

    fn substract_margins(&self, x: i32, y: i32) -> (i32, i32) {
        let geometry = self.callbacks.geometry.get();
        let (top, _, _, left) = geometry.margins;
        let top = top as f32 * geometry.hidpi_factor;
        let left = left as f32 * geometry.hidpi_factor;
        let x = x - left as i32;
        let y = y - top as i32;
        (x, y)
    }

    pub fn perform_mouse_move(&self, x: i32, y: i32) {
        let (x, y) = self.substract_margins(x, y);
        let event = WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32));
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn perform_scroll(&self, x: i32, y: i32, dx: f32, dy: f32, phase: view::TouchPhase) {
        let (x, y) = self.substract_margins(x, y);

        let delta = TypedVector2D::new(dx, dy);
        let scroll_location = webrender_api::ScrollLocation::Delta(delta);
        let phase = match phase {
            view::TouchPhase::Started => TouchEventType::Down,
            view::TouchPhase::Moved => TouchEventType::Move,
            view::TouchPhase::Ended => TouchEventType::Up,
        };
        let event = WindowEvent::Scroll(scroll_location, TypedPoint2D::new(x, y), phase);
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn update_geometry(&self, geometry: DrawableGeometry) {
        self.callbacks.geometry.set(geometry);
        let event = WindowEvent::Resize(self.callbacks.framebuffer_size());
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn perform_click(&self,
                 x: i32,
                 y: i32,
                 org_x: i32,
                 org_y: i32,
                 element_state: view::ElementState,
                 mouse_button: view::MouseButton,
                 mouse_down_button: Option<view::MouseButton>) {
        let (x, y) = self.substract_margins(x, y);
        let (org_x, org_y) = self.substract_margins(org_x, org_y);
        let max_pixel_dist = 10f64;
        let button = match mouse_button {
            view::MouseButton::Left => MouseButton::Left,
            view::MouseButton::Right => MouseButton::Right,
            view::MouseButton::Middle => MouseButton::Middle,
        };
        let event = match element_state {
            view::ElementState::Pressed => {
                MouseWindowEvent::MouseDown(button, TypedPoint2D::new(x as f32, y as f32))
            }
            view::ElementState::Released => {
                let mouse_up_event = MouseWindowEvent::MouseUp(button, TypedPoint2D::new(x as f32, y as f32));
                match mouse_down_button {
                    None => mouse_up_event,
                    Some(but) if mouse_button == but => {
                        // Same button
                        let pixel_dist = Point2D::new(org_x, org_y) - Point2D::new(x, y);
                        let pixel_dist =
                            ((pixel_dist.x * pixel_dist.x + pixel_dist.y * pixel_dist.y) as f64)
                                .sqrt();
                        if pixel_dist < max_pixel_dist {
                            self.events_for_servo.borrow_mut().push(WindowEvent::MouseWindowEventClass(mouse_up_event));
                            MouseWindowEvent::Click(button, TypedPoint2D::new(x as f32, y as f32))
                        } else {
                            mouse_up_event
                        }
                    }
                    Some(_) => mouse_up_event,
                }
            }
        };
        self.events_for_servo.borrow_mut().push(WindowEvent::MouseWindowEventClass(event));
    }

    pub fn zoom(&self, zoom: f32) {
        self.events_for_servo.borrow_mut().push(WindowEvent::Zoom(zoom));

    }

    pub fn reset_zoom(&self) {
        self.events_for_servo.borrow_mut().push(WindowEvent::ResetZoom);
    }

    pub fn set_webrender_profiler_enabled(&self, _enabled: bool) {
        // FIXME
    }

    pub fn sync(&self, force: bool) {
        // FIXME: ports/glutin/window.rs uses mem::replace. Should we too?
        // See: https://doc.rust-lang.org/core/mem/fn.replace.html
        if !self.events_for_servo.borrow().is_empty() || force {
            let mut events = self.events_for_servo.borrow_mut();
            let clone = events.drain(..).collect();
            self.servo.borrow_mut().handle_events(clone);
        }
    }
}

struct ServoCallbacks {
    pub geometry: Cell<DrawableGeometry>,
    event_queue: RefCell<Vec<ServoEvent>>,
    waker: Box<EventLoopWaker>,
    view: Rc<view::View>,
}

impl ServoCallbacks {
    fn get_events(&self) -> Vec<ServoEvent> {
        // FIXME: ports/glutin/window.rs uses mem::replace. Should we too?
        // See: https://doc.rust-lang.org/core/mem/fn.replace.html
        let mut events = self.event_queue.borrow_mut();
        let copy = events.drain(..).collect();
        copy
    }
}

impl WindowMethods for ServoCallbacks {
    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    fn supports_clipboard(&self) -> bool {
        // FIXME
        false
    }

    fn allow_navigation(&self, _id: BrowserId, _url: ServoUrl, chan: ipc::IpcSender<bool>) {
        chan.send(true).ok();
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        self.waker.clone()
    }

    fn gl(&self) -> Rc<gl::Gl> {
        self.view.gl()
    }

    fn hidpi_factor(&self) -> ScaleFactor<f32, DeviceIndependentPixel, DevicePixel> {
        let scale_factor = self.geometry.get().hidpi_factor;
        ScaleFactor::new(scale_factor)
    }

    fn framebuffer_size(&self) -> TypedSize2D<u32, DevicePixel> {
        let scale_factor = self.geometry.get().hidpi_factor as u32;
        let (width, height) = self.geometry.get().view_size;
        TypedSize2D::new(scale_factor * width, scale_factor * height)
    }

    fn window_rect(&self) -> TypedRect<u32, DevicePixel> {
        let scale_factor = self.geometry.get().hidpi_factor as u32;
        let mut size = self.framebuffer_size();

        let (top, right, bottom, left) = self.geometry.get().margins;
        let top = top * scale_factor;
        let right = right * scale_factor;
        let bottom = bottom * scale_factor;
        let left = left * scale_factor;

        size.height = size.height - top - bottom;
        size.width = size.width - left - right;
        
        TypedRect::new(TypedPoint2D::new(left, top), size)
    }

    fn size(&self) -> TypedSize2D<f32, DeviceIndependentPixel> {
        let (width, height) = self.geometry.get().view_size;
        TypedSize2D::new(width as f32, height as f32)
    }

    fn client_window(&self, _id: BrowserId) -> (Size2D<u32>, Point2D<i32>) {
        let (width, height) = self.geometry.get().view_size;
        let (x, y) = self.geometry.get().position;
        (Size2D::new(width, height), Point2D::new(x as i32, y as i32))
    }

    // Events

    fn set_inner_size(&self, _id: BrowserId, size: Size2D<u32>) {
        self.event_queue
            .borrow_mut()
            .push(ServoEvent::SetWindowInnerSize(size.width as u32, size.height as u32));
    }

    fn set_position(&self, _id: BrowserId, point: Point2D<i32>) {
        self.event_queue.borrow_mut().push(ServoEvent::SetWindowPosition(point.x, point.y));
    }

    fn set_fullscreen_state(&self, _id: BrowserId, state: bool) {
        self.event_queue.borrow_mut().push(ServoEvent::SetFullScreenState(state))
    }

    fn present(&self) {
        self.view.swap_buffers();
    }

    fn set_page_title(&self, id: BrowserId, title: Option<String>) {
        self.event_queue.borrow_mut().push(ServoEvent::TitleChanged(id, title));
    }

    fn status(&self, _id: BrowserId, status: Option<String>) {
        self.event_queue.borrow_mut().push(ServoEvent::StatusChanged(status));
    }

    fn load_start(&self, id: BrowserId) {
        self.event_queue.borrow_mut().push(ServoEvent::LoadStart(id));
    }

    fn load_end(&self, id: BrowserId) {
        self.event_queue.borrow_mut().push(ServoEvent::LoadEnd(id));
    }

    fn load_error(&self, _id: BrowserId, _: NetError, _url: String) {
        // FIXME: never called by servo
    }

    fn head_parsed(&self, id: BrowserId) {
        self.event_queue.borrow_mut().push(ServoEvent::HeadParsed(id));
    }

    fn history_changed(&self, id: BrowserId, entries: Vec<LoadData>, current: usize) {
        self.event_queue.borrow_mut().push(ServoEvent::HistoryChanged(id, entries, current));
    }

    fn set_cursor(&self, cursor: ServoCursor) {
        self.event_queue.borrow_mut().push(ServoEvent::CursorChanged(cursor));
    }

    fn set_favicon(&self, id: BrowserId, url: ServoUrl) {
        self.event_queue.borrow_mut().push(ServoEvent::FaviconChanged(id, url));
    }

    fn handle_key(&self, _id: Option<BrowserId>, ch: Option<char>, key: Key, mods: constellation_msg::KeyModifiers) {
        self.event_queue.borrow_mut().push(ServoEvent::Key(ch, key, mods));
    }
}
