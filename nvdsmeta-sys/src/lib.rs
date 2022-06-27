use gst::prelude::*;
use std::fmt;

#[link(name = "nvdsgst_meta")]
extern "C" {
    pub fn nvds_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub fn nvds_meta_api_get_type() -> glib::Type;
}

mod imp {
    #[repr(C)]
    pub struct NvDsMeta {
        meta: gst::ffi::GstMeta,
        pub(super) meta_data: glib::ffi::gpointer,
        user_data: glib::ffi::gpointer,
        pub(super) meta_type: i32,
        copyfunc: glib::ffi::gpointer,
        freefunc: glib::ffi::gpointer,
        gst_to_nvds_meta_transform_func: glib::ffi::gpointer,
        gst_to_nvds_meta_release_func: glib::ffi::gpointer,
    }
}
#[repr(transparent)]
pub struct NvDsMeta(imp::NvDsMeta);

// Metas must be Send+Sync.
unsafe impl Send for NvDsMeta {}
unsafe impl Sync for NvDsMeta {}

impl NvDsMeta {
    #[doc(alias = "get_label")]
    pub fn meta_type(&self) -> i32 {
        self.0.meta_type
    }
}

unsafe impl MetaAPI for NvDsMeta {
    type GstType = imp::NvDsMeta;

    fn meta_api() -> glib::Type {
        unsafe { crate::nvds_meta_api_get_type() }
    }
}

impl fmt::Debug for NvDsMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NvDsMeta")
            .field("meta_type", &self.meta_type())
            .finish()
    }
}
