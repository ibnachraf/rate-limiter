pub enum EngineError {
    BodyAnalyzer(BodyAnalyzerError),
}

pub enum BodyAnalyzerError {
    BodySizeExceeded,
}

