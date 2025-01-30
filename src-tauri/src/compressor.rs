// 音频压缩器结构体
// 用于控制音频信号的动态范围，防止音量过大
#[derive(Copy, Clone, Debug)]
pub struct Compressor {
  pub peak_at: f32,      // 峰值检测的攻击时间系数
  pub peak_rt: f32,      // 峰值检测的释放时间系数
  pub peak_average: f32, // 峰值的平均值

  pub gain_at: f32,      // 增益控制的攻击时间系数
  pub gain_rt: f32,      // 增益控制的释放时间系数
  pub gain_average: f32, // 增益的平均值

  pub threshold: f32, // 压缩阈值(dB)
}

// 计算时间常数tau
// sample_rate: 采样率
// time_ms: 时间(毫秒)
fn calc_tau(sample_rate: f32, time_ms: f32) -> f32 {
  1.0 - (-2200.0 / (time_ms * sample_rate)).exp()
}

// 限幅器函数
// 当输入信号超过阈值时，计算需要的增益衰减
fn limiter(input: f32, threshold: f32) -> f32 {
  let db = 20.0 * input.abs().log10(); // 将输入转换为分贝值
  let gain = (threshold - db).min(0.0); // 计算需要的增益衰减
  10.0f32.powf(0.05 * gain) // 将分贝值转回线性增益
}

// 计算攻击释放包络
// 使用不同的时间常数进行平滑处理
fn ar_avg(avg: f32, at: f32, rt: f32, input: f32) -> f32 {
  let tau = if input > avg { at } else { rt };

  (1.0 - tau).mul_add(avg, tau * input)
}

impl Compressor {
  // 对输入样本进行压缩处理
  pub fn compress(&mut self, input: f32) -> f32 {
    // 计算输入信号的峰值包络
    self.peak_average = ar_avg(self.peak_average, self.peak_at, self.peak_rt, input.abs());

    // 根据阈值计算需要的增益
    let gain = limiter(self.peak_average, self.threshold);

    // 平滑增益变化
    self.gain_average = ar_avg(self.gain_average, self.gain_rt, self.gain_at, gain);

    // 应用增益到输入信号
    self.gain_average * input
  }

  // 创建新的压缩器实例
  // sample_rate: 采样率
  // threshold: 压缩阈值
  // attack_ms: 攻击时间(毫秒)
  // release_ms: 释放时间(毫秒)
  pub fn new(sample_rate: f32, threshold: f32, attack_ms: f32, release_ms: f32) -> Self {
    Self {
      peak_at: calc_tau(sample_rate, 0.01), // 峰值检测使用固定的快速攻击时间
      peak_rt: calc_tau(sample_rate, 10.0), // 峰值检测使用固定的释放时间
      peak_average: 0.0,
      gain_at: calc_tau(sample_rate, attack_ms),
      gain_rt: calc_tau(sample_rate, release_ms),
      gain_average: 1.0,
      threshold,
    }
  }
}
