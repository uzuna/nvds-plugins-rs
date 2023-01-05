use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, NaiveDateTime, Utc};
use nvdsmeta_sys::{NvBbox_Coords, NvDsFrameMeta, NvDsObjectMeta};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBoxCorrds {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl From<&NvBbox_Coords> for BBoxCorrds {
    fn from(c: &NvBbox_Coords) -> Self {
        Self {
            left: c.left,
            top: c.top,
            width: c.width,
            height: c.height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub detection_index: u32,
    pub class_id: i32,
    pub object_id: u64,
    pub detector_bbox_info: BBoxCorrds,
    pub confidence: f32,
    pub label: String,
}

impl From<&NvDsObjectMeta> for ObjectMeta {
    fn from(x: &NvDsObjectMeta) -> Self {
        Self {
            detection_index: 0,
            class_id: x.class_id(),
            object_id: x.object_id(),
            detector_bbox_info: BBoxCorrds::from(x.detector_bbox()),
            confidence: x.confidence(),
            label: x.label().to_str().unwrap().to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferFrameInfo {
    source_id: u32,
    width: u32,
    height: u32,
    frame_num: i32,
    pts: u64,
    #[serde(with = "ts_nanoseconds")]
    infer_ts: DateTime<Utc>,
}

impl BufferFrameInfo {
    pub fn new(pts: u64, meta: &NvDsFrameMeta) -> Self {
        let infer_ts = meta.ntp_timestamp();
        let naive = NaiveDateTime::from_timestamp_opt(
            infer_ts as i64 / 1_000_000_000,
            (infer_ts % 1_000_000_000) as u32,
        )
        .unwrap();
        Self {
            source_id: meta.source_id(),
            width: meta.source_frame_width(),
            height: meta.source_frame_height(),
            frame_num: meta.frame_num(),
            pts,
            infer_ts: DateTime::<Utc>::from_utc(naive, Utc),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameObjects {
    frame: BufferFrameInfo,
    objects: Vec<ObjectMeta>,
}

impl FrameObjects {
    pub fn new(frame: BufferFrameInfo, objects: Vec<ObjectMeta>) -> Self {
        Self { frame, objects }
    }
}
