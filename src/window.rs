//! The `Window` struct and associated types.
use std::fmt;

use crate::{
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    error::{ExternalError, NotSupportedError, OsError},
    event_loop::EventLoopWindowTarget,
    monitor::{MonitorHandle, VideoMode},
    platform_impl,
};

pub use crate::icon::{BadIcon, Icon};

/// Represents a window.
///
/// # Example
///
/// ```no_run
/// use winit::{
///     event::{Event, WindowEvent},
///     event_loop::{ControlFlow, EventLoop},
///     window::Window,
/// };
///
/// let mut event_loop = EventLoop::new();
/// let window = Window::new(&event_loop).unwrap();
///
/// event_loop.run(move |event, _, control_flow| {
///     *control_flow = ControlFlow::Wait;
///
///     match event {
///         Event::WindowEvent {
///             event: WindowEvent::CloseRequested,
///             ..
///         } => *control_flow = ControlFlow::Exit,
///         _ => (),
///     }
/// });
/// ```
pub struct Window {
    pub(crate) window: platform_impl::Window,
}

impl fmt::Debug for Window {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmtr.pad("Window { .. }")
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // If the window is in exclusive fullscreen, we must restore the desktop
        // video mode (generally this would be done on application exit, but
        // closing the window doesn't necessarily always mean application exit,
        // such as when there are multiple windows)
        if let Some(Fullscreen::Exclusive(_)) = self.fullscreen() {
            self.set_fullscreen(None);
        }
    }
}

/// Identifier of a window. Unique for each window.
///
/// Can be obtained with `window.id()`.
///
/// Whenever you receive an event specific to a window, this event contains a `WindowId` which you
/// can then compare to the ids of your windows.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub(crate) platform_impl::WindowId);

impl WindowId {
    /// Returns a dummy `WindowId`, useful for unit testing. The only guarantee made about the return
    /// value of this function is that it will always be equal to itself and to future values returned
    /// by this function.  No other guarantees are made. This may be equal to a real `WindowId`.
    ///
    /// **Passing this into a winit function will result in undefined behavior.**
    pub unsafe fn dummy() -> Self {
        WindowId(platform_impl::WindowId::dummy())
    }
}

/// Object that allows you to build windows.
#[derive(Clone, Default)]
pub struct WindowBuilder {
    /// The attributes to use to create the window.
    pub window: WindowAttributes,

    // Platform-specific configuration.
    pub(crate) platform_specific: platform_impl::PlatformSpecificWindowBuilderAttributes,
}

impl fmt::Debug for WindowBuilder {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmtr.debug_struct("WindowBuilder")
            .field("window", &self.window)
            .finish()
    }
}

/// Attributes to use when creating a window.
#[derive(Debug, Clone)]
pub struct WindowAttributes {
    /// The dimensions of the window. If this is `None`, some platform-specific dimensions will be
    /// used.
    ///
    /// The default is `None`.
    pub inner_size: Option<Size>,

    /// The minimum dimensions a window can be, If this is `None`, the window will have no minimum dimensions (aside from reserved).
    ///
    /// The default is `None`.
    pub min_inner_size: Option<Size>,

    /// The maximum dimensions a window can be, If this is `None`, the maximum will have no maximum or will be set to the primary monitor's dimensions by the platform.
    ///
    /// The default is `None`.
    pub max_inner_size: Option<Size>,

    /// The desired position of the window. If this is `None`, some platform-specific position
    /// will be chosen.
    ///
    /// The default is `None`.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS**: The top left corner position of the window content, the window's "inner"
    /// position. The window title bar will be placed above it.
    /// The window will be positioned such that it fits on screen, maintaining
    /// set `inner_size` if any.
    /// If you need to precisely position the top left corner of the whole window you have to
    /// use [`Window::set_outer_position`] after creating the window.
    /// - **Windows**: The top left corner position of the window title bar, the window's "outer"
    /// position.
    /// There may be a small gap between this position and the window due to the specifics of the
    /// Window Manager.
    /// - **X11**: The top left corner of the window, the window's "outer" position.
    /// - **Others**: Ignored.
    ///
    /// See [`Window::set_outer_position`].
    ///
    /// [`Window::set_outer_position`]: crate::window::Window::set_outer_position
    pub position: Option<Position>,

    /// Whether the window is resizable or not.
    ///
    /// The default is `true`.
    pub resizable: bool,

    /// Whether the window should be set as fullscreen upon creation.
    ///
    /// The default is `None`.
    pub fullscreen: Option<Fullscreen>,

    /// The title of the window in the title bar.
    ///
    /// The default is `"winit window"`.
    pub title: String,

    /// Whether the window should be maximized upon creation.
    ///
    /// The default is `false`.
    pub maximized: bool,

    /// Whether the window should be immediately visible upon creation.
    ///
    /// The default is `true`.
    pub visible: bool,

    /// Whether the the window should be transparent. If this is true, writing colors
    /// with alpha values different than `1.0` will produce a transparent window.
    ///
    /// The default is `false`.
    pub transparent: bool,

    /// Whether the window should have borders and bars.
    ///
    /// The default is `true`.
    pub decorations: bool,

    /// Whether the window should always be on top of other windows.
    ///
    /// The default is `false`.
    pub always_on_top: bool,

    /// The window icon.
    ///
    /// The default is `None`.
    pub window_icon: Option<Icon>,
}

impl Default for WindowAttributes {
    #[inline]
    fn default() -> WindowAttributes {
        WindowAttributes {
            inner_size: None,
            min_inner_size: None,
            max_inner_size: None,
            position: None,
            resizable: true,
            title: "winit window".to_owned(),
            maximized: false,
            fullscreen: None,
            visible: true,
            transparent: false,
            decorations: true,
            always_on_top: false,
            window_icon: None,
        }
    }
}

impl WindowBuilder {
    /// Initializes a new `WindowBuilder` with default values.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Requests the window to be of specific dimensions.
    ///
    /// See [`Window::set_inner_size`] for details.
    ///
    /// [`Window::set_inner_size`]: crate::window::Window::set_inner_size
    #[inline]
    pub fn with_inner_size<S: Into<Size>>(mut self, size: S) -> Self {
        self.window.inner_size = Some(size.into());
        self
    }

    /// Sets a minimum dimension size for the window.
    ///
    /// See [`Window::set_min_inner_size`] for details.
    ///
    /// [`Window::set_min_inner_size`]: crate::window::Window::set_min_inner_size
    #[inline]
    pub fn with_min_inner_size<S: Into<Size>>(mut self, min_size: S) -> Self {
        self.window.min_inner_size = Some(min_size.into());
        self
    }

    /// Sets a maximum dimension size for the window.
    ///
    /// See [`Window::set_max_inner_size`] for details.
    ///
    /// [`Window::set_max_inner_size`]: crate::window::Window::set_max_inner_size
    #[inline]
    pub fn with_max_inner_size<S: Into<Size>>(mut self, max_size: S) -> Self {
        self.window.max_inner_size = Some(max_size.into());
        self
    }

    /// Sets a desired initial position for the window.
    ///
    /// See [`WindowAttributes::position`] for details.
    ///
    /// [`WindowAttributes::position`]: crate::window::WindowAttributes::position
    #[inline]
    pub fn with_position<P: Into<Position>>(mut self, position: P) -> Self {
        self.window.position = Some(position.into());
        self
    }

    /// Sets whether the window is resizable or not.
    ///
    /// See [`Window::set_resizable`] for details.
    ///
    /// [`Window::set_resizable`]: crate::window::Window::set_resizable
    #[inline]
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.window.resizable = resizable;
        self
    }

    /// Requests a specific title for the window.
    ///
    /// See [`Window::set_title`] for details.
    ///
    /// [`Window::set_title`]: crate::window::Window::set_title
    #[inline]
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.window.title = title.into();
        self
    }

    /// Sets the window fullscreen state.
    ///
    /// See [`Window::set_fullscreen`] for details.
    ///
    /// [`Window::set_fullscreen`]: crate::window::Window::set_fullscreen
    #[inline]
    pub fn with_fullscreen(mut self, fullscreen: Option<Fullscreen>) -> Self {
        self.window.fullscreen = fullscreen;
        self
    }

    /// Requests maximized mode.
    ///
    /// See [`Window::set_maximized`] for details.
    ///
    /// [`Window::set_maximized`]: crate::window::Window::set_maximized
    #[inline]
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.window.maximized = maximized;
        self
    }

    /// Sets whether the window will be initially hidden or visible.
    ///
    /// See [`Window::set_visible`] for details.
    ///
    /// [`Window::set_visible`]: crate::window::Window::set_visible
    #[inline]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.window.visible = visible;
        self
    }

    /// Sets whether the background of the window should be transparent.
    #[inline]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.window.transparent = transparent;
        self
    }

    /// Sets whether the window should have a border, a title bar, etc.
    ///
    /// See [`Window::set_decorations`] for details.
    ///
    /// [`Window::set_decorations`]: crate::window::Window::set_decorations
    #[inline]
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.window.decorations = decorations;
        self
    }

    /// Sets whether or not the window will always be on top of other windows.
    ///
    /// See [`Window::set_always_on_top`] for details.
    ///
    /// [`Window::set_always_on_top`]: crate::window::Window::set_always_on_top
    #[inline]
    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.window.always_on_top = always_on_top;
        self
    }

    /// Sets the window icon.
    ///
    /// See [`Window::set_window_icon`] for details.
    ///
    /// [`Window::set_window_icon`]: crate::window::Window::set_window_icon
    #[inline]
    pub fn with_window_icon(mut self, window_icon: Option<Icon>) -> Self {
        self.window.window_icon = window_icon;
        self
    }

    /// Builds the window.
    ///
    /// Possible causes of error include denied permission, incompatible system, and lack of memory.
    ///
    /// Platform-specific behavior:
    /// - **Web**: The window is created but not inserted into the web page automatically. Please
    /// see the web platform module for more information.
    #[inline]
    pub fn build<T: 'static>(
        self,
        window_target: &EventLoopWindowTarget<T>,
    ) -> Result<Window, OsError> {
        platform_impl::Window::new(&window_target.p, self.window, self.platform_specific).map(
            |window| {
                window.request_redraw();
                Window { window }
            },
        )
    }
}

/// Base Window functions.
impl Window {
    /// Creates a new Window for platforms where this is appropriate.
    ///
    /// This function is equivalent to [`WindowBuilder::new().build(event_loop)`].
    ///
    /// Error should be very rare and only occur in case of permission denied, incompatible system,
    ///  out of memory, etc.
    ///
    /// Platform-specific behavior:
    /// - **Web**: The window is created but not inserted into the web page automatically. Please
    /// see the web platform module for more information.
    ///
    /// [`WindowBuilder::new().build(event_loop)`]: crate::window::WindowBuilder::build
    #[inline]
    pub fn new<T: 'static>(event_loop: &EventLoopWindowTarget<T>) -> Result<Window, OsError> {
        let builder = WindowBuilder::new();
        builder.build(event_loop)
    }

    /// Returns an identifier unique to the window.
    #[inline]
    pub fn id(&self) -> WindowId {
        WindowId(self.window.id())
    }

    /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
    ///
    /// See the [`dpi`](crate::dpi) module for more information.
    ///
    /// Note that this value can change depending on user action (for example if the window is
    /// moved to another screen); as such, tracking `WindowEvent::ScaleFactorChanged` events is
    /// the most robust way to track the DPI you need to use to draw.
    ///
    /// ## Platform-specific
    ///
    /// - **X11:** This respects Xft.dpi, and can be overridden using the `WINIT_X11_SCALE_FACTOR` environment variable.
    /// - **Android:** Always returns 1.0.
    /// - **iOS:** Can only be called on the main thread. Returns the underlying `UIView`'s
    ///   [`contentScaleFactor`].
    ///
    /// [`contentScaleFactor`]: https://developer.apple.com/documentation/uikit/uiview/1622657-contentscalefactor?language=objc
    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    /// Emits a `WindowEvent::RedrawRequested` event in the associated event loop after all OS
    /// events have been processed by the event loop.
    ///
    /// This is the **strongly encouraged** method of redrawing windows, as it can integrate with
    /// OS-requested redraws (e.g. when a window gets resized).
    ///
    /// This function can cause `RedrawRequested` events to be emitted after `Event::MainEventsCleared`
    /// but before `Event::NewEvents` if called in the following circumstances:
    /// * While processing `MainEventsCleared`.
    /// * While processing a `RedrawRequested` event that was sent during `MainEventsCleared` or any
    ///   directly subsequent `RedrawRequested` event.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread.
    /// - **Android:** Unsupported.
    #[inline]
    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }

    /// Reset the dead key state of the keyboard.
    ///
    /// This is useful when a dead key is bound to trigger an action. Then
    /// this function can be called to reset the dead key state so that
    /// follow-up text input won't be affected by the dead key.
    ///
    /// ## Platform-specific
    /// - **Web:** Does nothing
    // ---------------------------
    // Developers' Note: If this cannot be implemented on every desktop platform
    // at least, then this function should be provided through a platform specific
    // extension trait
    pub fn reset_dead_keys(&self) {
        self.window.reset_dead_keys();
    }
}

/// Position and size functions.
impl Window {
    /// Returns the position of the top-left hand corner of the window's client area relative to the
    /// top-left hand corner of the desktop.
    ///
    /// The same conditions that apply to `outer_position` apply to this method.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Returns the top left coordinates of the
    ///   window's [safe area] in the screen space coordinate system.
    /// - **Web:** Returns the top-left coordinates relative to the viewport. _Note: this returns the
    ///    same value as `outer_position`._
    /// - **Android / Wayland:** Always returns [`NotSupportedError`].
    ///
    /// [safe area]: https://developer.apple.com/documentation/uikit/uiview/2891103-safeareainsets?language=objc
    #[inline]
    pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, NotSupportedError> {
        self.window.inner_position()
    }

    /// Returns the position of the top-left hand corner of the window relative to the
    ///  top-left hand corner of the desktop.
    ///
    /// Note that the top-left hand corner of the desktop is not necessarily the same as
    ///  the screen. If the user uses a desktop with multiple monitors, the top-left hand corner
    ///  of the desktop is the top-left hand corner of the monitor at the top-left of the desktop.
    ///
    /// The coordinates can be negative if the top-left hand corner of the window is outside
    ///  of the visible screen region.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Returns the top left coordinates of the
    ///   window in the screen space coordinate system.
    /// - **Web:** Returns the top-left coordinates relative to the viewport.
    /// - **Android / Wayland:** Always returns [`NotSupportedError`].
    #[inline]
    pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, NotSupportedError> {
        self.window.outer_position()
    }

    /// Modifies the position of the window.
    ///
    /// See `outer_position` for more information about the coordinates. This automatically un-maximizes the
    /// window if it's maximized.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Sets the top left coordinates of the
    ///   window in the screen space coordinate system.
    /// - **Web:** Sets the top-left coordinates relative to the viewport.
    /// - **Android / Wayland:** Unsupported.
    #[inline]
    pub fn set_outer_position<P: Into<Position>>(&self, position: P) {
        self.window.set_outer_position(position.into())
    }

    /// Returns the physical size of the window's client area.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Returns the `PhysicalSize` of the window's
    ///   [safe area] in screen space coordinates.
    /// - **Web:** Returns the size of the canvas element.
    ///
    /// [safe area]: https://developer.apple.com/documentation/uikit/uiview/2891103-safeareainsets?language=objc
    #[inline]
    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    /// Modifies the inner size of the window.
    ///
    /// See `inner_size` for more information about the values. This automatically un-maximizes the
    /// window if it's maximized.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android:** Unsupported.
    /// - **Web:** Sets the size of the canvas element.
    #[inline]
    pub fn set_inner_size<S: Into<Size>>(&self, size: S) {
        self.window.set_inner_size(size.into())
    }

    /// Returns the physical size of the entire window.
    ///
    /// These dimensions include the title bar and borders. If you don't want that (and you usually don't),
    /// use `inner_size` instead.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Returns the `PhysicalSize` of the window in
    ///   screen space coordinates.
    /// - **Web:** Returns the size of the canvas element. _Note: this returns the same value as
    ///   `inner_size`._
    #[inline]
    pub fn outer_size(&self) -> PhysicalSize<u32> {
        self.window.outer_size()
    }

    /// Sets a minimum dimension size for the window.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    #[inline]
    pub fn set_min_inner_size<S: Into<Size>>(&self, min_size: Option<S>) {
        self.window.set_min_inner_size(min_size.map(|s| s.into()))
    }

    /// Sets a maximum dimension size for the window.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    #[inline]
    pub fn set_max_inner_size<S: Into<Size>>(&self, max_size: Option<S>) {
        self.window.set_max_inner_size(max_size.map(|s| s.into()))
    }
}

/// Misc. attribute functions.
impl Window {
    /// Modifies the title of the window.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android:** Unsupported.
    #[inline]
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title)
    }

    /// Modifies the window's visibility.
    ///
    /// If `false`, this will hide the window. If `true`, this will show the window.
    /// ## Platform-specific
    ///
    /// - **Android / Wayland / Web:** Unsupported.
    /// - **iOS:** Can only be called on the main thread.
    #[inline]
    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible)
    }

    /// Sets whether the window is resizable or not.
    ///
    /// Note that making the window unresizable doesn't exempt you from handling `Resized`, as that event can still be
    /// triggered by DPI scaling, entering fullscreen mode, etc.
    ///
    /// ## Platform-specific
    ///
    /// This only has an effect on desktop platforms.
    ///
    /// Due to a bug in XFCE, this has no effect on Xfwm.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    #[inline]
    pub fn set_resizable(&self, resizable: bool) {
        self.window.set_resizable(resizable)
    }

    /// Sets the window to minimized or back
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    /// - **Wayland:** Un-minimize is unsupported.
    #[inline]
    pub fn set_minimized(&self, minimized: bool) {
        self.window.set_minimized(minimized);
    }

    /// Sets the window to maximized or back.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    #[inline]
    pub fn set_maximized(&self, maximized: bool) {
        self.window.set_maximized(maximized)
    }

    /// Gets the window's current maximized state.
    ///
    /// ## Platform-specific
    ///
    /// - **Wayland / X11:** Not implemented.
    /// - **iOS / Android / Web:** Unsupported.
    #[inline]
    pub fn is_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    /// Sets the window to fullscreen or back.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** `Fullscreen::Exclusive` provides true exclusive mode with a
    ///   video mode change. *Caveat!* macOS doesn't provide task switching (or
    ///   spaces!) while in exclusive fullscreen mode. This mode should be used
    ///   when a video mode change is desired, but for a better user experience,
    ///   borderless fullscreen might be preferred.
    ///
    ///   `Fullscreen::Borderless` provides a borderless fullscreen window on a
    ///   separate space. This is the idiomatic way for fullscreen games to work
    ///   on macOS. See `WindowExtMacOs::set_simple_fullscreen` if
    ///   separate spaces are not preferred.
    ///
    ///   The dock and the menu bar are always disabled in fullscreen mode.
    /// - **iOS:** Can only be called on the main thread.
    /// - **Wayland:** Does not support exclusive fullscreen mode and will no-op a request.
    /// - **Windows:** Screen saver is disabled in fullscreen mode.
    /// - **Android:** Unsupported.
    #[inline]
    pub fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        self.window.set_fullscreen(fullscreen)
    }

    /// Gets the window's current fullscreen state.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread.
    /// - **Android:** Will always return `None`.
    /// - **Wayland:** Can return `Borderless(None)` when there are no monitors.
    #[inline]
    pub fn fullscreen(&self) -> Option<Fullscreen> {
        self.window.fullscreen()
    }

    /// Turn window decorations on or off.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    ///
    /// [`setPrefersStatusBarHidden`]: https://developer.apple.com/documentation/uikit/uiviewcontroller/1621440-prefersstatusbarhidden?language=objc
    #[inline]
    pub fn set_decorations(&self, decorations: bool) {
        self.window.set_decorations(decorations)
    }

    /// Change whether or not the window will always be on top of other windows.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web / Wayland:** Unsupported.
    #[inline]
    pub fn set_always_on_top(&self, always_on_top: bool) {
        self.window.set_always_on_top(always_on_top)
    }

    /// Sets the window icon. On Windows and X11, this is typically the small icon in the top-left
    /// corner of the titlebar.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web / Wayland / macOS:** Unsupported.
    ///
    /// On Windows, this sets `ICON_SMALL`. The base size for a window icon is 16x16, but it's
    /// recommended to account for screen scaling and pick a multiple of that, i.e. 32x32.
    ///
    /// X11 has no universal guidelines for icon sizes, so you're at the whims of the WM. That
    /// said, it's usually in the same ballpark as on Windows.
    #[inline]
    pub fn set_window_icon(&self, window_icon: Option<Icon>) {
        self.window.set_window_icon(window_icon)
    }

    /// Sets location of IME candidate box in client area coordinates relative to the top left.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    #[inline]
    pub fn set_ime_position<P: Into<Position>>(&self, position: P) {
        self.window.set_ime_position(position.into())
    }

    /// Requests user attention to the window, this has no effect if the application
    /// is already focused. How requesting for user attention manifests is platform dependent,
    /// see `UserAttentionType` for details.
    ///
    /// Providing `None` will unset the request for user attention. Unsetting the request for
    /// user attention might not be done automatically by the WM when the window receives input.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web / Wayland:** Unsupported.
    /// - **macOS:** `None` has no effect.
    /// - **X11:** Requests for user attention must be manually cleared.
    #[inline]
    pub fn request_user_attention(&self, request_type: Option<UserAttentionType>) {
        self.window.request_user_attention(request_type)
    }
}

/// Cursor functions.
impl Window {
    /// Modifies the cursor icon of the window.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android:** Unsupported.
    #[inline]
    pub fn set_cursor_icon(&self, cursor: CursorIcon) {
        self.window.set_cursor_icon(cursor);
    }

    /// Changes the position of the cursor in window coordinates.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web / Wayland:** Always returns an [`ExternalError::NotSupported`].
    #[inline]
    pub fn set_cursor_position<P: Into<Position>>(&self, position: P) -> Result<(), ExternalError> {
        self.window.set_cursor_position(position.into())
    }

    /// Grabs the cursor, preventing it from leaving the window.
    ///
    /// There's no guarantee that the cursor will be hidden. You should
    /// hide it by yourself if you want so.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** This locks the cursor in a fixed location, which looks visually awkward.
    /// - **iOS / Android / Web:** Always returns an [`ExternalError::NotSupported`].
    #[inline]
    pub fn set_cursor_grab(&self, grab: bool) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(grab)
    }

    /// Modifies the cursor's visibility.
    ///
    /// If `false`, this will hide the cursor. If `true`, this will show the cursor.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows:** The cursor is only hidden within the confines of the window.
    /// - **X11:** The cursor is only hidden within the confines of the window.
    /// - **Wayland:** The cursor is only hidden within the confines of the window.
    /// - **macOS:** The cursor is hidden as long as the window has input focus, even if the cursor is
    ///   outside of the window.
    /// - **iOS / Android:** Unsupported.
    #[inline]
    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }

    /// Moves the window with the left mouse button until the button is released.
    ///
    /// There's no guarantee that this will work unless the left mouse button was pressed
    /// immediately before this function is called.
    ///
    /// ## Platform-specific
    ///
    /// - **X11:** Un-grabs the cursor.
    /// - **Wayland:** Requires the cursor to be inside the window to be dragged.
    /// - **macOS:** May prevent the button release event to be triggered.
    /// - **iOS / Android / Web:** Always returns an [`ExternalError::NotSupported`].
    #[inline]
    pub fn drag_window(&self) -> Result<(), ExternalError> {
        self.window.drag_window()
    }
}

/// Monitor info functions.
impl Window {
    /// Returns the monitor on which the window currently resides.
    ///
    /// Returns `None` if current monitor can't be detected.
    ///
    /// ## Platform-specific
    ///
    /// **iOS:** Can only be called on the main thread.
    #[inline]
    pub fn current_monitor(&self) -> Option<MonitorHandle> {
        self.window.current_monitor()
    }

    /// Returns the list of all the monitors available on the system.
    ///
    /// This is the same as `EventLoopWindowTarget::available_monitors`, and is provided for convenience.
    ///
    /// ## Platform-specific
    ///
    /// **iOS:** Can only be called on the main thread.
    #[inline]
    pub fn available_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
        self.window
            .available_monitors()
            .into_iter()
            .map(|inner| MonitorHandle { inner })
    }

    /// Returns the primary monitor of the system.
    ///
    /// Returns `None` if it can't identify any monitor as a primary one.
    ///
    /// This is the same as `EventLoopWindowTarget::primary_monitor`, and is provided for convenience.
    ///
    /// ## Platform-specific
    ///
    /// **iOS:** Can only be called on the main thread.
    /// **Wayland:** Always returns `None`.
    #[inline]
    pub fn primary_monitor(&self) -> Option<MonitorHandle> {
        self.window.primary_monitor()
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for Window {
    /// Returns a `raw_window_handle::RawWindowHandle` for the Window
    ///
    /// ## Platform-specific
    ///
    /// - **Android:** Only available after receiving the Resumed event and before Suspended. *If you*
    /// *try to get the handle outside of that period, this function will panic*!
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }
}

/// Describes the appearance of the mouse cursor.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CursorIcon {
    /// The platform-dependent default cursor.
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    /// Self explanatory.
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done. But in contrast
    /// with "Wait" the user may still interact with the program. Often rendered
    /// as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,

    /// Cursor showing that something cannot be done.
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    /// Indicates something can be grabbed.
    Grab,
    /// Indicates something is grabbed.
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,

    /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
    /// is used when the movement starts from the south-east corner of the box.
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

impl Default for CursorIcon {
    fn default() -> Self {
        CursorIcon::Default
    }
}

/// Fullscreen modes.
#[derive(Clone, Debug, PartialEq)]
pub enum Fullscreen {
    Exclusive(VideoMode),

    /// Providing `None` to `Borderless` will fullscreen on the current monitor.
    Borderless(Option<MonitorHandle>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

/// ## Platform-specific
///
/// - **X11:** Sets the WM's `XUrgencyHint`. No distinction between `Critical` and `Informational`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserAttentionType {
    /// ## Platform-specific
    /// - **macOS:** Bounces the dock icon until the application is in focus.
    /// - **Windows:** Flashes both the window and the taskbar button until the application is in focus.
    Critical,
    /// ## Platform-specific
    /// - **macOS:** Bounces the dock icon once.
    /// - **Windows:** Flashes the taskbar button until the application is in focus.
    Informational,
}

impl Default for UserAttentionType {
    fn default() -> Self {
        UserAttentionType::Informational
    }
}
