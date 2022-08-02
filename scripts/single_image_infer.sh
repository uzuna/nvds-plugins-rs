if [ $# -ne 1 ]; then
  echo "must set <config-file-path>" 1>&2
  exit 1
fi

gst-launch-1.0 \
v4l2src device=/dev/video0 num-buffers=10 ! \
decodebin ! \
videoconvert ! \
nvvideoconvert ! \
m.sink_0 nvstreammux name=m batch-size=1 width=1280 height=720 batched-push-timeout=40000 nvbuf-memory-type=0 ! \
nvinfer config-file-path=$1 ! \
queue ! \
nvdsosd ! \
nvstreamdemux name=d d.src_0 ! fakesink