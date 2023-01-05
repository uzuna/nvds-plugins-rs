# nvdsmeta example

## Run

```sh
# build custom impl for nvinfer
cd /opt/nvidia/deepstream/deepstream/sources/objectDetector_Yolo/nvdsinfer_custom_impl_Yolo
sudo make

# run
cd <project-dir>
make run
```

## detail

### infer configについて

相対パス指定の基準が2つある

- 実行ディレクトリ
- configファイル基準

field毎の参照先は以下

|ファイル指定名|WorkingDir基準|ファイルが無い場合のエラーメッセージ例|
| --- | --- | --- |
| custom-network-config | 実行dir | `File does not exist : yolov3.cfg` |
| model-file | configファイル | `File does not exist : <model-file>` |
| model-engine-file | configファイル(生成は実行dir) | `WARNING: Deserialize engine failed because file path: /home/fmy/repos/rust/github.com/uzuna/nvds-plugins-rs/scripts/model_b1_gpu0_int8.engine open error` |
| labelfile-path | configファイル | `NvDsInfer Error: NVDSINFER_CUSTOM_LIB_FAILED` |
| custom-lib-path| configファイル | `Could not open labels file:` |
