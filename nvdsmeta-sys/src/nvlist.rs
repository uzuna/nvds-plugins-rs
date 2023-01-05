use std::marker::PhantomData;
use gst::glib;

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
    pub fn from_glib_full(list: *mut glib::ffi::GList) -> GList {
        GList {
            ptr: std::ptr::NonNull::new(list),
        }
    }
}

pub struct GListIter<'a, T> {
    ptr: Option<std::ptr::NonNull<glib::ffi::GList>>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for GListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ptr {
            None => None,
            Some(cur) => unsafe {
                self.ptr = std::ptr::NonNull::new(cur.as_ref().next);
                if let Some(mut next) = self.ptr {
                    next.as_mut().prev = std::ptr::null_mut();
                }
                let item = &*(cur.as_ref().data as *const T);
                Some(item)
            },
        }
    }
}

impl<'a, T> GListIter<'a, T> {
    pub(crate) fn from_glib_full(list: *mut glib::ffi::GList) -> GListIter<'a, T> {
        GListIter {
            ptr: std::ptr::NonNull::new(list),
            phantom: PhantomData,
        }
    }
}
