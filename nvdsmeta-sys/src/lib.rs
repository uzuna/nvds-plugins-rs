use gst::{glib, prelude::*, ClockTime};
use std::{ffi::CStr, fmt};

mod imp;
pub mod nvlist;

#[link(name = "nvdsgst_meta")]
extern "C" {
    // pub(crate) fn nvds_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub(crate) fn nvds_meta_api_get_type() -> glib::Type;
}

mod nvgst {
    use gst::glib;
    #[allow(non_upper_case_globals)]
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

pub use imp::NvBbox_Coords;
#[repr(transparent)]
#[derive(Debug)]
pub struct NvDsObjectMeta(imp::NvDsObjectMeta);

impl NvDsObjectMeta {
    #[inline]
    pub fn class_id(&self) -> i32 {
        self.0.class_id
    }
    #[inline]
    pub fn object_id(&self) -> u64 {
        self.0.object_id
    }
    #[inline]
    pub fn confidence(&self) -> f32 {
        self.0.confidence
    }
    #[inline]
    pub fn tracker_confidence(&self) -> f32 {
        self.0.tracker_confidence
    }
    #[inline]
    pub fn label(&self) -> &CStr {
        unsafe { CStr::from_ptr(&self.0.obj_label as *const std::os::raw::c_char) }
    }
    #[inline]
    pub fn detector_bbox(&self) -> &NvBbox_Coords {
        &self.0.detector_bbox_info.org_bbox_coords
    }
    #[inline]
    pub fn tracker_bbox(&self) -> &NvBbox_Coords {
        &self.0.tracker_bbox_info.org_bbox_coords
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct NvDsFrameMeta(imp::NvDsFrameMeta);

impl NvDsFrameMeta {
    pub fn object_meta_list(&self) -> nvlist::GListIter<NvDsObjectMeta> {
        nvlist::GListIter::from_glib_full(self.0.obj_meta_list as *mut glib::ffi::GList)
    }
    #[inline]
    pub fn source_id(&self) -> u32 {
        self.0.source_id
    }
    #[inline]
    pub fn frame_num(&self) -> i32 {
        self.0.frame_num
    }
    #[inline]
    pub fn buf_pts(&self) -> ClockTime {
        ClockTime::from_nseconds(self.0.buf_pts)
    }
    #[inline]
    pub fn ntp_timestamp(&self) -> u64 {
        self.0.ntp_timestamp
    }
    #[inline]
    pub fn source_frame_width(&self) -> u32 {
        self.0.source_frame_width
    }
    #[inline]
    pub fn source_frame_height(&self) -> u32 {
        self.0.source_frame_height
    }
    #[inline]
    pub fn num_obj_meta(&self) -> u32 {
        self.0.num_obj_meta
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
    pub fn frame_meta_list(&self) -> nvlist::GListIter<NvDsFrameMeta> {
        nvlist::GListIter::from_glib_full(self.0.frame_meta_list as *mut glib::ffi::GList)
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
