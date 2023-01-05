

.PHONY: run
run: yolov3.cfg yolov3.weights
	LD_LIBRARY_PATH=/opt/nvidia/deepstream/deepstream/lib cargo run video-file

yolov3.cfg:
	wget https://raw.githubusercontent.com/pjreddie/darknet/master/cfg/yolov3.cfg -q --show-progress

yolov3.weights:
	wget https://pjreddie.com/media/files/yolov3.weights -q --show-progress