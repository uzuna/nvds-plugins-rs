pub struct GList {
    ptr: Option<std::ptr::NonNull<glib::ffi::GList>>,
}

impl Iterator for GList {
    type Item = std::ptr::NonNull<glib::ffi::GList>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ptr {
            None => None,
            Some(cur) => unsafe {
                self.ptr = std::ptr::NonNull::new(cur.as_ref().next);
                if let Some(mut next) = self.ptr {
                    next.as_mut().prev = std::ptr::null_mut();
                }
                Some(cur)
            },
        }
    }
}

impl GList {
    pub unsafe fn from_glib_full(list: *mut glib::ffi::GList) -> GList {
        GList {
            ptr: std::ptr::NonNull::new(list),
        }
    }
}
