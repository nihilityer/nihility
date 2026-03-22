use ndarray::{Array2, s};

/// Low Frame Rate（降低帧率处理）
pub struct Lfr {
    lfr_m: usize,
    lfr_n: usize,
}

impl Lfr {
    pub fn init(lfr_m: usize, lfr_n: usize) -> Self {
        Lfr { lfr_m, lfr_n }
    }

    /// 应用 LFR (Low Frame Rate) 处理
    ///
    /// # Arguments
    /// * `inputs` - 输入的二维数组 (T, D)
    /// * `lfr_m` - 窗口大小 (Window size)
    /// * `lfr_n` - 步长 (Stride)
    ///
    /// # Returns
    /// * 返回处理后的二维数组 (T_lfr, lfr_m * D)
    pub fn apply_lfr(&self, inputs: Array2<f32>) -> Array2<f32> {
        if self.lfr_m == 1 && self.lfr_n == 1 {
            return inputs;
        }
        let (t_original, d) = inputs.dim();

        // 计算输出的时间步 T_lfr
        // Python: int(np.ceil(T / lfr_n))
        // Rust 整数向上取整技巧: (numerator + denominator - 1) / denominator
        let t_lfr = t_original.div_ceil(self.lfr_n);

        // 左侧填充 (Left Padding)
        // Python: np.tile(inputs[0], ((lfr_m - 1) // 2, 1))
        let left_pad_count = (self.lfr_m - 1) / 2;

        // 为了方便切片，构建一个新的 padded_inputs
        // 首先预分配空间以提高性能
        let padded_t = t_original + left_pad_count;
        let mut padded_inputs = Array2::<f32>::zeros((padded_t, d));

        // 填充左侧 padding (复制第一行)
        let first_row = inputs.row(0);
        for i in 0..left_pad_count {
            padded_inputs.row_mut(i).assign(&first_row);
        }

        // 填充原始数据
        // slice_mut 对应 Python 的 padded_inputs[left_pad_count:, :] = inputs
        padded_inputs
            .slice_mut(s![left_pad_count.., ..])
            .assign(&inputs);

        // 构建输出 LFR
        // 输出维度: (T_lfr, lfr_m * D)
        let output_feature_dim = self.lfr_m * d;
        let mut lfr_outputs = Array2::<f32>::zeros((t_lfr, output_feature_dim));

        for i in 0..t_lfr {
            let start_idx = i * self.lfr_n;
            let end_idx = start_idx + self.lfr_m;

            // 获取输出矩阵的当前行
            let mut output_row = lfr_outputs.row_mut(i);

            if end_idx <= padded_t {
                // Case A: 窗口完整，直接切片并展平
                let window = padded_inputs.slice(s![start_idx..end_idx, ..]);

                // 将窗口内的元素按顺序填入 output_row
                // ndarray 默认是 row-major，iter() 会按行遍历，符合 Python reshape 行为
                for (j, &val) in window.iter().enumerate() {
                    output_row[j] = val;
                }
            } else {
                // Case B: 处理最后一个 LFR 帧 (需要右侧填充)
                // 获取剩余的真实数据
                let available_rows = padded_t - start_idx;
                let window_part = padded_inputs.slice(s![start_idx..padded_t, ..]);

                let mut write_idx = 0;

                // 写入真实数据
                for &val in window_part.iter() {
                    output_row[write_idx] = val;
                    write_idx += 1;
                }

                // 用最后一帧填充剩余部分
                let num_padding_rows = self.lfr_m - available_rows;
                let last_row = padded_inputs.row(padded_t - 1);

                for _ in 0..num_padding_rows {
                    for &val in last_row.iter() {
                        output_row[write_idx] = val;
                        write_idx += 1;
                    }
                }
            }
        }

        lfr_outputs
    }
}
