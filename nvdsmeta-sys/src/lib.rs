use gst::prelude::*;
use std::{fmt, ffi::CStr};

mod imp;
pub mod nvlist;

#[link(name = "nvdsgst_meta")]
extern "C" {
    // pub(crate) fn nvds_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub(crate) fn nvds_meta_api_get_type() -> glib::Type;
}

mod nvgst {
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

#[repr(transparent)]
#[derive(Debug)]
pub struct NvDsObjectMeta(imp::NvDsObjectMeta);

impl NvDsObjectMeta {
    pub fn to_object_meta(&self) -> ObjectMeta {
        ObjectMeta::from(&self.0)
    }
}

#[derive(Debug)]
pub struct BBoxCorrds {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl From<&imp::NvDsComp_BboxInfo> for BBoxCorrds {
    fn from(x: &imp::NvDsComp_BboxInfo) -> Self {
        let c = x.org_bbox_coords;
        Self { left: c.left, top: c.top, width: c.width, height: c.height }
    }
}

#[derive(Debug)]
pub struct ObjectMeta {
    pub class_id: i32,
    pub object_id: u64,
    pub detector_bbox_info: BBoxCorrds,
    pub confidence: f32,
    pub label: String,
}

impl From<&imp::NvDsObjectMeta> for ObjectMeta {
    fn from(x: &imp::NvDsObjectMeta) -> Self {
        let label = unsafe {CStr::from_ptr(&x.obj_label as *const std::os::raw::c_char)};
        Self { class_id: x.class_id, 
            object_id: x.object_id, 
            detector_bbox_info: BBoxCorrds::from(&x.detector_bbox_info), 
            confidence: x.confidence, 
            label: label.to_str().unwrap().to_owned(),
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct NvDsFrameMeta(imp::NvDsFrameMeta);

impl NvDsFrameMeta {
    pub fn object_meta_list(&self) -> nvlist::GListIter<NvDsObjectMeta> {
        nvlist::GListIter::from_glib_full(self.0.obj_meta_list as *mut glib::ffi::GList)
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
