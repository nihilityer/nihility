use crate::error::*;
use ndarray::{Array2, s};
use ndarray_npy::read_npy;
use std::path::Path;

/// 倒谱均值方差归一化
pub struct Cmvn {
    cmvn: Array2<f32>,
}

impl Cmvn {
    pub fn init<P: AsRef<Path>>(data_path: P) -> Result<Self> {
        Ok(Self {
            cmvn: read_npy(data_path)?,
        })
    }

    pub fn apply_cmvn(&self, inputs: Array2<f32>) -> Array2<f32> {
        let (_frame, dim) = inputs.dim();

        // 这里我们需要取出 means 和 vars，并确保切片长度与 input 的 dim 一致
        // 注意：如果 cmvn 的维度小于输入 dim，这里会 panic
        let row_0 = self.cmvn.row(0);
        let row_1 = self.cmvn.row(1);
        let means = row_0.slice(s![..dim]);
        let vars = row_1.slice(s![..dim]);

        // 复制 inputs 以便进行修改
        // inputs shape: (T, D)
        // means shape:  (D,)  <- ndarray 会自动广播这个一维数组到每一行
        let mut outputs = inputs.to_owned();

        // 执行 (inputs + means) * vars
        outputs += &means;
        outputs *= &vars;

        outputs
    }
}
