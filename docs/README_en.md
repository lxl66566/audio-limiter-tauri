# Loudness Normalization tauri app

[简体中文](../README.md) | English

This is a real-time audio loudness equalization program based on the [ebur128](https://github.com/sdroege/ebur128) algorithm, rewritten due to the unsatisfactory UI and algorithms of [dylagit/audio-limiter](https://github.com/dylagit/audio-limiter). It uses Tauri for the interface and can be launched via command line, making it suitable for adding as a startup script.

## Usage

```bash
loudness-normalization list                # List all audio devices
loudness-normalization -i "xxx" -o "xxx"   # Start with the given input and output devices
```

Then left-click the tray icon to open the window for volume control.

Generally, you need to install [vb cable](https://vb-audio.com/Cable/), and then set `CABLE Input (VB-Audio Virtual Cable)` as the default output device in Windows audio settings. The default input for loudness-normalization should be set to `CABLE Output (VB-Audio Virtual Cable)`, and the output to your speakers.

## Development

```bash
pnpm i
pnpm tauri dev -- -- list
# OR
bash testrun.sh
```
