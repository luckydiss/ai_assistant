use thiserror::Error;

#[derive(Debug, Error)]
pub enum DialogueError {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Summary generation failed: {0}")]
    SummaryFailed(String),
}
