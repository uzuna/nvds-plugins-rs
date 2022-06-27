use gst::prelude::*;
use std::fmt;

mod imp;

#[link(name = "nvdsgst_meta")]
extern "C" {
    pub fn nvds_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub fn nvds_meta_api_get_type() -> glib::Type;
}

mod nvgst {
    #[allow(non_camel_case_types)]
    pub(crate) const NvDsMetaType_NVDS_GST_BATCH_META: crate::imp::NvDsMetaType =
        crate::imp::NvDsMetaType_NVDS_GST_CUSTOM_META + 1;
    #[repr(C)]
    pub struct NvDsMeta {
        meta: gst::ffi::GstMeta,
        pub(super) meta_data: glib::ffi::gpointer,
        user_data: glib::ffi::gpointer,
        pub(super) meta_type: crate::imp::NvDsMetaType,
        copyfunc: glib::ffi::gpointer,
        freefunc: glib::ffi::gpointer,
        gst_to_nvds_meta_transform_func: glib::ffi::gpointer,
        gst_to_nvds_meta_release_func: glib::ffi::gpointer,
    }
}
#[repr(transparent)]
pub struct NvDsMeta(nvgst::NvDsMeta);

// Metas must be Send+Sync.
unsafe impl Send for NvDsMeta {}
unsafe impl Sync for NvDsMeta {}

impl NvDsMeta {
    #[doc(alias = "get_meta_type")]
    pub fn meta_type(&self) -> crate::imp::NvDsMetaType {
        self.0.meta_type
    }

    pub fn get_batch_meta(&self) -> Option<&NvDsBatchMeta> {
        if self.meta_type() == nvgst::NvDsMetaType_NVDS_GST_BATCH_META {
            unsafe { Some(&*(self.0.meta_data as *const NvDsBatchMeta)) }
        } else {
            None
        }
    }
}

unsafe impl MetaAPI for NvDsMeta {
    type GstType = nvgst::NvDsMeta;

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

#[repr(transparent)]
pub struct NvDsBatchMeta(imp::NvDsBatchMeta);

impl NvDsBatchMeta {
    pub fn max_frames_in_batch(&self) -> u32 {
        self.0.max_frames_in_batch
    }
    pub fn num_frames_in_batch(&self) -> u32 {
        self.0.num_frames_in_batch
    }
}

impl fmt::Debug for NvDsBatchMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NvDsBatchMeta")
            .field("max_frames_in_batch", &self.max_frames_in_batch())
            .field("num_frames_in_batch", &self.num_frames_in_batch())
            .finish()
    }
}
