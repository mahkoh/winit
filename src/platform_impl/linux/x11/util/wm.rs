use super::*;
use xcb_dl_util::property::XcbGetPropertyError;

impl XConnection {
    pub fn update_cached_wm_info(&self) {
        for screen in &self.screens {
            *screen.supported_hints.lock() = self.get_supported_hints(screen.root);
            *screen.wm_name.lock() = self.get_wm_name(screen.root);
        }
    }

    fn get_supported_hints(&self, root: ffi::xcb_window_t) -> Vec<ffi::xcb_atom_t> {
        let supported_atom = self.get_atom("_NET_SUPPORTED");
        self.get_property(root, supported_atom, ffi::XCB_ATOM_ATOM)
            .unwrap_or_else(|_| Vec::with_capacity(0))
    }

    fn get_wm_name(&self, root: ffi::xcb_window_t) -> Option<String> {
        let check_atom = self.get_atom("_NET_SUPPORTING_WM_CHECK");
        let wm_name_atom = self.get_atom("_NET_WM_NAME");

        // Mutter/Muffin/Budgie doesn't have _NET_SUPPORTING_WM_CHECK in its _NET_SUPPORTED, despite
        // it working and being supported. This has been reported upstream, but due to the
        // inavailability of time machines, we'll just try to get _NET_SUPPORTING_WM_CHECK
        // regardless of whether or not the WM claims to support it.
        //
        // Blackbox 0.70 also incorrectly reports not supporting this, though that appears to be fixed
        // in 0.72.
        /*if !supported_hints.contains(&check_atom) {
            return None;
        }*/

        // IceWM (1.3.x and earlier) doesn't report supporting _NET_WM_NAME, but will nonetheless
        // provide us with a value for it. Note that the unofficial 1.4 fork of IceWM works fine.
        /*if !supported_hints.contains(&wm_name_atom) {
            return None;
        }*/

        // Of the WMs tested, only xmonad and dwm fail to provide a WM name.

        // Querying this property on the root window will give us the ID of a child window created by
        // the WM.
        let root_window_wm_check = {
            let result = self.get_property(root, check_atom, ffi::XCB_ATOM_WINDOW);

            let wm_check = result.ok().and_then(|wm_check| wm_check.get(0).cloned());

            if let Some(wm_check) = wm_check {
                wm_check
            } else {
                return None;
            }
        };

        // Querying the same property on the child window we were given, we should get this child
        // window's ID again.
        let child_window_wm_check = {
            let result = self.get_property(root_window_wm_check, check_atom, ffi::XCB_ATOM_WINDOW);

            let wm_check = result.ok().and_then(|wm_check| wm_check.get(0).cloned());

            if let Some(wm_check) = wm_check {
                wm_check
            } else {
                return None;
            }
        };

        // These values should be the same.
        if root_window_wm_check != child_window_wm_check {
            return None;
        }

        // All of that work gives us a window ID that we can get the WM name from.
        let wm_name = {
            let utf8_string_atom = self.get_atom("UTF8_STRING");

            let result = self.get_property(root_window_wm_check, wm_name_atom, utf8_string_atom);

            // IceWM requires this. IceWM was also the only WM tested that returns a null-terminated
            // string. For more fun trivia, IceWM is also unique in including version and uname
            // information in this string (this means you'll have to be careful if you want to match
            // against it, though).
            // The unofficial 1.4 fork of IceWM still includes the extra details, but properly
            // returns a UTF8 string that isn't null-terminated.
            let no_utf8 = match result {
                Err(XcbGetPropertyError::InvalidPropertyType {
                    actual: ffi::XCB_ATOM_STRING,
                    ..
                }) => true,
                _ => false,
            };

            if no_utf8 {
                self.get_property(root_window_wm_check, wm_name_atom, ffi::XCB_ATOM_STRING)
            } else {
                result
            }
        }
        .ok();

        wm_name.and_then(|wm_name| String::from_utf8(wm_name).ok())
    }
}
