//! Example using nvdsmeta-sys with Appsink
//! 
//! and Use to check the operation of nvdsmeta-sys.
#![allow(clippy::non_send_fields_in_send_ty)]

use anyhow::Error;
use chrono::serde::ts_nanoseconds;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use nvdsmeta_sys::NvBbox_Coords;

use nvdsmeta_sys::NvDsObjectMeta;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use gst::element_error;
use gst::prelude::*;

fn create_source(s: &Source, pipeline: &gst::Pipeline) -> Result<gst::Element, Error> {
    match s {
        Source::ImageFile { location } => {
            let src = gst::ElementFactory::make("filesrc", None)?;
            let dec = gst::ElementFactory::make("jpegdec", None)?;

            src.set_property("location", location);
            pipeline.add_many(&[&src, &dec])?;
            gst::Element::link_many(&[&src, &dec])?;
            Ok(dec)
        }
        Source::VideoFile {
            location,
            num_buffers,
        } => {
            let src = gst::ElementFactory::make("filesrc", None)?;
            let parse = gst::ElementFactory::make("h264parse", None)?;
            let dec = gst::ElementFactory::make("nvv4l2decoder", None)?;

            src.set_property("location", location);

            src.set_property("num-buffers", num_buffers);

            pipeline.add_many(&[&src, &parse, &dec])?;
            gst::Element::link_many(&[&src, &parse, &dec])?;
            Ok(dec)
        }
        Source::V4l2Src {
            device,
            num_buffers,
        } => {
            let src = gst::ElementFactory::make("v4l2src", None)?;
            let dec = gst::ElementFactory::make("nvv4l2decoder", None)?;

            src.set_property("device", device);
            src.set_property("num-buffers", num_buffers);

            pipeline.add_many(&[&src, &dec])?;
            gst::Element::link_many(&[&src, &dec])?;
            Ok(dec)
        }
    }
}

fn create_pipeline(opt: &Opt) -> Result<gst::Pipeline, Error> {
    gst::init()?;

    let pipeline = gst::Pipeline::new(None);
    let srcbin = create_source(&opt.source, &pipeline)?;

    let vidconv = gst::ElementFactory::make("videoconvert", None)?;
    let nvvidconv = gst::ElementFactory::make("nvvideoconvert", None)?;
    let nvstreammux = gst::ElementFactory::make("nvstreammux", None)?;
    let nvinfer = gst::ElementFactory::make("nvinfer", None)?;
    let appsink = gst::ElementFactory::make("appsink", None)?;

    nvstreammux.set_property("batch-size", 1u32);
    nvstreammux.set_property("width", 1280u32);
    nvstreammux.set_property("height", 720u32);
    nvstreammux.set_property("batched-push-timeout", 40000i32);
    // FIXME we can use GstNvBufMemoryType? 
    // nvstreammux.set_property("nvbuf-memory-type", "0");

    nvinfer.set_property("config-file-path", "../scripts/config_infer_yolov3.txt");

    pipeline.add_many(&[&vidconv, &nvvidconv, &nvstreammux, &nvinfer, &appsink])?;
    gst::Element::link_many(&[&srcbin, &vidconv, &nvvidconv])?;
    let src_pad = nvvidconv.static_pad("src").expect("has not src pad");
    let sink_pad = nvstreammux
        .request_pad_simple("sink_0")
        .expect("has not sink pad");
    src_pad.link(&sink_pad)?;
    gst::Element::link_many(&[&nvstreammux, &nvinfer, &appsink])?;

    let appsink = appsink.downcast::<gst_app::AppSink>().unwrap();

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(|appsink| {
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or_else(|| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to get buffer from appsink")
                    );
                    gst::FlowError::Error
                })?;

                let meta = buffer
                    .meta::<nvdsmeta_sys::NvDsMeta>()
                    .expect("No custom meta found");

                if let Some(meta) = meta.get_batch_meta() {
                    let list = meta.frame_meta_list();

                    for meta in list {
                        let frame_info = BufferFrameInfo::new(
                            meta.frame_num() as i64,
                            *buffer.pts().unwrap(),
                            meta.ntp_timestamp(),
                        );
                        let objs = meta.object_meta_list();
                        for (j, o) in objs.enumerate() {
                            let mut obj = ObjectMeta::from(o);
                            obj.detection_index = j as u32;
                            let msg = ObjectMessage::new(frame_info.clone(), obj);
                            println!("msg {:?}", msg);
                        }
                    }
                }

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    Ok(pipeline)
}

fn example_main(opt: &Opt) {
    let pipeline = create_pipeline(opt).unwrap();

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        }
    }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BufferFrameInfo {
    frame_no: i64,
    pts: u64,
    #[serde(with = "ts_nanoseconds")]
    infer_ts: DateTime<Utc>,
}

impl BufferFrameInfo {
    fn new(frame_no: i64, pts: u64, infer_ts: u64) -> Self {
        let naive = NaiveDateTime::from_timestamp_opt(
            infer_ts as i64 / 1000_000_000,
            (infer_ts % 1000_000_000) as u32,
        )
        .unwrap();
        Self {
            frame_no,
            pts,
            infer_ts: DateTime::<Utc>::from_utc(naive, Utc),
        }
    }
}

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
struct ObjectMessage {
    frame: BufferFrameInfo,
    object: ObjectMeta,
}

impl ObjectMessage {
    fn new(frame: BufferFrameInfo, object: ObjectMeta) -> Self{
        Self{ frame, object }
    }
}

#[derive(Debug, StructOpt)]
enum Source {
    /// inference image file
    ImageFile {
        #[structopt(
            short, long,
            default_value = "/opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.jpg"
        )]
        location: String,
    },
    /// inference video file
    VideoFile {
        #[structopt(
            short, long,
            default_value = "/opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.h264"
        )]
        location: String,
        /// Number of buffers to flow in the pipeline
        #[structopt(long, default_value = "30")]
        num_buffers: i32,
    },
    /// inference v4l2 camera source
    V4l2Src {
        #[structopt(short, long, default_value = "/dev/video0")]
        device: String,
        /// Number of buffers to flow in the pipeline
        #[structopt(long, default_value = "30")]
        num_buffers: i32,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mvdsmeta_app",
    about = "test nvdsmeta with deepstremaer sample"
)]
struct Opt {
    #[structopt(subcommand)]
    source: Source,
}

fn main() {
    let opt = Opt::from_args();
    log::debug!("{:?}", opt);
    example_main(&opt);
}
