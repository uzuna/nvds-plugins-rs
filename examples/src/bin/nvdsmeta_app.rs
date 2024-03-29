//! Example using nvdsmeta-sys with Appsink
//!
//! and Use to check the operation of nvdsmeta-sys.
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};

use anyhow::Error;

use examples::ObjectMeta;
use examples::{BufferFrameInfo, FrameObjects};
use structopt::StructOpt;

use gst::element_error;
use gst::prelude::*;

fn create_source(s: &Source, pipeline: &gst::Pipeline) -> Result<gst::Element, Error> {
    match s {
        Source::ImageFile { location } => {
            let src = gst::ElementFactory::make("filesrc").build()?;
            let dec = gst::ElementFactory::make("jpegdec").build()?;
            let vidconv = gst::ElementFactory::make("videoconvert").build()?;

            src.set_property("location", location);
            pipeline.add_many(&[&src, &dec, &vidconv])?;
            gst::Element::link_many(&[&src, &dec, &vidconv])?;
            Ok(vidconv)
        }
        Source::VideoFile {
            location,
            num_buffers,
        } => {
            let src = gst::ElementFactory::make("filesrc").build()?;
            let parse = gst::ElementFactory::make("h264parse").build()?;
            let dec = gst::ElementFactory::make("nvv4l2decoder").build()?;

            src.set_property("location", location);

            src.set_property("num-buffers", num_buffers);

            pipeline.add_many(&[&src, &parse, &dec])?;
            gst::Element::link_many(&[&src, &parse, &dec])?;
            Ok(dec)
        }
        Source::V4l2Src {
            device,
            num_buffers,
            width,
            height,
        } => {
            let src = gst::ElementFactory::make("v4l2src").build()?;
            let vidconv = gst::ElementFactory::make("videoconvert").build()?;

            src.set_property("device", device);
            src.set_property("num-buffers", num_buffers);

            let caps = gst::Caps::builder("video/x-raw")
                .field("width", width)
                .field("height", height)
                .build();

            pipeline.add_many(&[&src, &vidconv])?;
            src.link_filtered(&vidconv, &caps)?;

            Ok(vidconv)
        }
    }
}

fn create_pipeline(opt: &Opt, sender: Sender<FrameObjects>) -> Result<gst::Pipeline, Error> {
    gst::init()?;

    let pipeline = gst::Pipeline::new(None);
    let srcbin = create_source(&opt.source, &pipeline)?;

    let nvvidconv = gst::ElementFactory::make("nvvideoconvert").build()?;
    let nvstreammux = gst::ElementFactory::make("nvstreammux").build()?;
    let nvinfer = gst::ElementFactory::make("nvinfer").build()?;
    let appsink = gst::ElementFactory::make("appsink").build()?;

    nvstreammux.set_property("batch-size", 1u32);
    nvstreammux.set_property("width", 1280u32);
    nvstreammux.set_property("height", 720u32);
    nvstreammux.set_property("batched-push-timeout", 40000i32);
    // FIXME we can use GstNvBufMemoryType?
    // nvstreammux.set_property("nvbuf-memory-type", "0");

    nvinfer.set_property("config-file-path", opt.config_infer_file.to_str().unwrap());

    pipeline.add_many(&[&nvvidconv, &nvstreammux, &nvinfer, &appsink])?;
    gst::Element::link_many(&[&srcbin, &nvvidconv])?;
    let src_pad = nvvidconv.static_pad("src").expect("has not src pad");
    let sink_pad = nvstreammux
        .request_pad_simple("sink_0")
        .expect("has not sink pad");
    src_pad.link(&sink_pad)?;
    gst::Element::link_many(&[&nvstreammux, &nvinfer, &appsink])?;

    let appsink = appsink.downcast::<gst_app::AppSink>().unwrap();

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
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
                        let frame_info = BufferFrameInfo::new(*buffer.pts().unwrap(), meta);

                        let objs = meta.object_meta_list();
                        let objects = objs
                            .enumerate()
                            .map(|(j, o)| {
                                let mut obj = ObjectMeta::from(o);
                                obj.detection_index = j as u32;
                                obj
                            })
                            .collect();
                        sender.send(FrameObjects::new(frame_info, objects)).unwrap();
                    }
                }

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    Ok(pipeline)
}

fn example_main(opt: &Opt) {
    use std::io::Write;
    let (sender, receiver) = channel();
    let pipeline = create_pipeline(opt, sender).unwrap();

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");
    let mut f = std::io::BufWriter::new(std::fs::File::create(&opt.export_json).unwrap());
    'outer: loop {
        if let Ok(v) = receiver.try_recv() {
            serde_json::to_writer(&mut f, &v).unwrap();
            writeln!(&mut f).unwrap();
        }

        for msg in bus.iter_timed(gst::ClockTime::MSECOND * 10) {
            use gst::MessageView;
            match msg.view() {
                MessageView::Eos(..) => break 'outer,
                MessageView::Error(err) => {
                    println!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    break 'outer;
                }
                _ => (),
            }
        }
    }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}

#[derive(Debug, StructOpt)]
enum Source {
    /// inference image file
    ImageFile {
        #[structopt(
            short,
            long,
            default_value = "/opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.jpg"
        )]
        location: String,
    },
    /// inference video file
    VideoFile {
        #[structopt(
            short,
            long,
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

        #[structopt(long, default_value = "1280")]
        width: i32,
        #[structopt(long, default_value = "720")]
        height: i32,
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

    #[structopt(long, parse(from_os_str), default_value = "config_infer_yolov3.txt")]
    config_infer_file: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "detect.json")]
    export_json: PathBuf,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let opt = Opt::from_args();
    log::debug!("{:?}", opt);
    example_main(&opt);
}
