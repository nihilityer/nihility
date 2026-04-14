# 初始化项目运行环境

## 目前无法正常工作的情况

1. 当使用`hf-mirror.com`作为下载镜像时，`SenseVoice`的`tokenizer.json`会下载失败，原因是镜像站返回的请求头中不包含
   `content-range`