# Audio-Limiter-tauri

核心代码使用 [dylagit/audio-limiter](https://github.com/dylagit/audio-limiter) 的，但是 UI 换成了 tauri。

## Usage

```bash
audio-limiter list                # 列出所有音频设备
audio-limiter -i "xxx" -o "xxx"   # 设置输入输出设备
```

然后左键托盘图标，进行音量控制。

一般来说需要安装 [vb cable](https://vb-audio.com/Cable/)，然后在 windows 系统音频设置里将 `CABLE Input (VB-Audio Virtual Cable)` 设为默认输出设备。audio-limiter 的默认 input 需要设为 `CABLE Output (VB-Audio Virtual Cable)`，output 为扬声器。

## Development

```bash
pnpm i
pnpm tauri dev
```
