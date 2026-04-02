use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use ndarray::{ArrayView2, ArrayViewD, Axis, Ix2};

pub fn squeeze_singleton_axes_to_2d<'a>(
    mut output: ArrayViewD<'a, f32>,
    method: &str,
) -> VisionResult<ArrayView2<'a, f32>> {
    let output_shape = output.shape().to_vec();
    for axis in (0..output.ndim()).rev() {
        if output.ndim() > 2 && output.shape()[axis] == 1 {
            output = output.index_axis_move(Axis(axis), 0);
        }
    }

    output
        .into_dimensionality::<Ix2>()
        .map_err(|e| VisionError::DataProcessingErr {
            method: method.to_string(),
            e: format!("输出布局不符合预期: shape={:?}, error={}", output_shape, e),
        })
}

pub fn select_batch_and_squeeze_to_2d<'a>(
    output: ArrayViewD<'a, f32>,
    batch_index: usize,
    method: &str,
) -> VisionResult<ArrayView2<'a, f32>> {
    let output_shape = output.shape().to_vec();
    let sample = match output.ndim() {
        0 | 1 => {
            return Err(VisionError::DataProcessingErr {
                method: method.to_string(),
                e: format!("输出维度过低: {:?}", output_shape),
            });
        }
        2 => output,
        _ => {
            if batch_index >= output_shape[0] {
                return Err(VisionError::BatchMatchDetSizeFailed {
                    batch: output_shape[0],
                    det_num: batch_index + 1,
                });
            }
            output.clone().index_axis_move(Axis(0), batch_index)
        }
    };

    squeeze_singleton_axes_to_2d(sample, method)
}
