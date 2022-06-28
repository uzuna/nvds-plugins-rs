// This example demonstrates how custom GstMeta can be defined and used on buffers.
//
// It simply attaches a GstMeta with a Rust String to buffers that are passed into
// an appsrc and retrieves them again from an appsink.
#![allow(clippy::non_send_fields_in_send_ty)]

use std::str::FromStr;

use anyhow::Error;
use structopt::StructOpt;

use gst::element_error;
use gst::prelude::*;

fn create_source(s: &Source, pipeline: &gst::Pipeline) -> Result<gst::Element, Error> {
    match s {
        Source::ImageFile => {
            let src = gst::ElementFactory::make("filesrc", None)?;
            let dec = gst::ElementFactory::make("jpegdec", None)?;

            src.set_property(
                "location",
                "/opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.jpg",
            );
            pipeline.add_many(&[&src, &dec]);
            gst::Element::link_many(&[&src, &dec])?;
            Ok(dec)
        }
        Source::VideoFile => {
            let src = gst::ElementFactory::make("filesrc", None)?;
            let parse = gst::ElementFactory::make("h264parse", None)?;
            let dec = gst::ElementFactory::make("nvv4l2decoder", None)?;

            src.set_property(
                "location",
                "/opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.h264",
            );

            src.set_property("num-buffers", 10i32);

            pipeline.add_many(&[&src, &parse, &dec]);
            gst::Element::link_many(&[&src, &parse, &dec])?;
            Ok(dec)
        }
    }
}

fn create_pipeline(opt: &Opt) -> Result<gst::Pipeline, Error> {
    gst::init()?;

    // This creates a pipeline with appsrc and appsink.
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

    // Getting data out of the appsink is done by setting callbacks on it.
    // The appsink will then call those handlers, as soon as data is available.
    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            // Add a handler to the "new-sample" signal.
            .new_sample(|appsink| {
                // Pull the sample in question out of the appsink's buffer.
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;

                let buffer = sample.buffer().ok_or_else(|| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to get buffer from appsink")
                    );

                    gst::FlowError::Error
                })?;

                // Retrieve the custom meta from the buffer and print it.
                let meta = buffer
                    .meta::<nvdsmeta_sys::NvDsMeta>()
                    .expect("No custom meta found");
                println!("Got buffer type: {}", meta.meta_type());

                if let Some(meta) = meta.get_batch_meta() {
                    println!(
                        "Got batch meta: {} {}",
                        meta.max_frames_in_batch(),
                        meta.num_frames_in_batch()
                    );
                    let list = meta.frame_meta_list();

                    for (i, meta) in list.enumerate() {
                        println!("frame meta {} {:?}", i, &meta);
                        let objs = meta.object_meta_list();
                        for (j, o) in objs.enumerate() {
                            println!("object meta {} {:?}", j, o.to_object_meta());
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

    // Actually start the pipeline.
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    // And run until EOS or an error happened.
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

    // Finally shut down everything.
    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}

#[derive(Debug, StructOpt)]
enum Source {
    ImageFile,
    VideoFile,
}

impl Default for Source {
    fn default() -> Self {
        Self::ImageFile
    }
}
type ParseError = &'static str;
impl FromStr for Source {
    type Err = ParseError;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        match src {
            "image" => Ok(Self::ImageFile),
            "video" => Ok(Self::VideoFile),
            _ => Err("Could not parse a source"),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mvdsmeta_app",
    about = "test nvdsmeta with deepstremaer sample"
)]
struct Opt {
    /// Select inference source
    #[structopt(short, default_value = "image")]
    source: Source,
}

fn main() {
    // tutorials_common::run is only required to set up the application environment on macOS
    // (but not necessary in normal Cocoa applications where this is set up automatically).
    let opt = Opt::from_args();
    println!("{:?}", opt);
    example_main(&opt);
}
