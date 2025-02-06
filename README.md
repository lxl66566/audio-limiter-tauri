# Loudness Normalization tauri app

简体中文 | [English](docs/README_en.md)

由于 [dylagit/audio-limiter](https://github.com/dylagit/audio-limiter) 的 UI 和算法都不是很好用，我重新写了一个基于 [ebur128](https://github.com/sdroege/ebur128) 算法的实时音频响度均衡程序。使用 tauri 作为界面，并且结合了命令行启动方式，可以作为开机脚本添加到系统。

## Usage

```bash
loudness-normalization list                # 列出所有音频设备
loudness-normalization -i "xxx" -o "xxx"   # 以给定的输入输出设备启动
```

然后左键托盘图标，打开窗口，进行音量控制。

一般来说需要安装 [vb cable](https://vb-audio.com/Cable/)，然后在 windows 系统音频设置里将 `CABLE Input (VB-Audio Virtual Cable)` 设为默认输出设备。loudness-normalization 的默认 input 需要设为 `CABLE Output (VB-Audio Virtual Cable)`，output 为扬声器。

## Development

```bash
pnpm i
pnpm tauri dev -- -- list
# OR
bash testrun.sh
```
