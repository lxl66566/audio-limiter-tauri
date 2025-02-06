use ebur128::{EbuR128, Mode};
use plotters::prelude::*;
use std::collections::VecDeque;

const PLOT_WINDOW: usize = 4800; // 100ms @ 48kHz

// 音频压缩器结构体
// 用于控制音频信号的动态范围，防止音量过大
#[derive(Debug)]
pub struct Compressor {
    // ebur128 状态
    ebu: EbuR128,
    // 目标响度值(LUFS)
    target_loudness: f64,
    // 当前增益
    current_gain: f32,
    // 平滑系数
    smoothing: f32,
    // 通道数
    channels: usize,
    // 临时缓冲区
    buffer: Vec<f32>,
    // 用于绘图的样本历史
    input_history: VecDeque<f32>,
    output_history: VecDeque<f32>,
}

impl Compressor {
    // 创建新的压缩器实例
    // sample_rate: 采样率
    // channels: 通道数
    // target_loudness: 目标响度值(LUFS)
    pub fn new(sample_rate: u32, channels: u32, target_loudness: f64) -> Self {
        // 创建 ebur128 实例,启用所有模式
        let mode = Mode::I | Mode::S | Mode::M;
        let ebu = EbuR128::new(channels, sample_rate, mode).unwrap();

        Self {
            ebu,
            target_loudness,
            current_gain: 1.0,
            smoothing: 0.95,
            channels: channels as usize,
            buffer: Vec::with_capacity(channels as usize * 32), // 增大缓冲区以存储更多帧
            input_history: VecDeque::with_capacity(PLOT_WINDOW),
            output_history: VecDeque::with_capacity(PLOT_WINDOW),
        }
    }

    // 对输入样本进行压缩处理
    pub fn compress_frame(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());

        // 添加整个帧到 ebur128
        if let Ok(()) = self.ebu.add_frames_f32(input) {
            // 获取当前响度
            if let Ok(loudness) = self.ebu.loudness_momentary() {
                if loudness.is_finite() {
                    // 计算需要的增益
                    let target_gain = if loudness < -70.0 {
                        1.0
                    } else {
                        let gain_db = self.target_loudness - loudness;
                        // 限制增益范围在更合理的范围内
                        10.0f64.powf(gain_db / 20.0).clamp(0.5, 2.0) as f32
                    };

                    // 平滑增益变化
                    self.current_gain =
                        self.smoothing * self.current_gain + (1.0 - self.smoothing) * target_gain;
                }
            }
        }

        // 对所有通道应用相同的增益
        for &sample in input {
            let out = sample * self.current_gain;
            output.push(out);

            // 只记录第一个通道的数据用于绘图
            if output.len() % self.channels == 1 {
                if self.input_history.len() >= PLOT_WINDOW {
                    self.input_history.pop_front();
                    self.output_history.pop_front();
                }
                self.input_history.push_back(sample);
                self.output_history.push_back(out);
            }
        }

        output
    }

    pub fn plot_waveforms(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Audio Waveform", ("sans-serif", 30))
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(0..PLOT_WINDOW, -1.5f32..1.5f32)?;

        chart
            .configure_mesh()
            .y_desc("Amplitude")
            .x_desc("Sample")
            .draw()?;

        // 绘制输入波形
        chart
            .draw_series(LineSeries::new(
                self.input_history.iter().enumerate().map(|(i, &v)| (i, v)),
                &BLUE,
            ))?
            .label("Input")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        // 绘制输出波形
        chart
            .draw_series(LineSeries::new(
                self.output_history.iter().enumerate().map(|(i, &v)| (i, v)),
                &RED,
            ))?
            .label("Output")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        Ok(())
    }

    // 更新目标响度
    pub fn set_target_loudness(&mut self, target: f64) {
        self.target_loudness = target;
    }

    // 重置状态
    pub fn reset(&mut self) {
        self.ebu.reset();
        self.current_gain = 1.0;
        self.buffer.clear();
        self.input_history.clear();
        self.output_history.clear();
    }
}
