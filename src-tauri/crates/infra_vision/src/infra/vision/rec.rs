use crate::infra::vision::rec::paddle_crnn::PaddleRecCrnn;
use domain_vision::RecognizerType;

pub(crate) mod paddle_crnn;
#[derive(Debug)]
pub(crate) enum RuntimeRecognizer {
    PaddleCrnn(PaddleRecCrnn),
}

impl From<RecognizerType> for RuntimeRecognizer {
    fn from(config: RecognizerType) -> Self {
        match config {
            RecognizerType::PaddleCrnn(config) => Self::PaddleCrnn(config.into()),
        }
    }
}
