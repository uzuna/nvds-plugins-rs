gst-launch-1.0 \
filesrc location=/opt/nvidia/deepstream/deepstream/samples/streams/sample_720p.jpg ! \
jpegdec ! \
videoconvert ! \
nvvideoconvert ! \
m.sink_0 nvstreammux name=m batch-size=1 width=1280 height=720 batched-push-timeout=40000 nvbuf-memory-type=0 ! \
nvinfer config-file-path=config_infer_yolov3.txt ! \
queue ! \
nvdsosd ! \
nvstreamdemux name=d d.src_0 ! \
nvvideoconvert ! \
jpegenc ! \
filesink location=_infer_image.jpg