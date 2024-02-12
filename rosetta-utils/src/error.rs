/// Maximum length of error messages to log.
const ERROR_MSG_MAX_LENGTH: usize = 100;

/// Helper type that truncates the error message to `ERROR_MSG_MAX_LENGTH` before logging.
pub struct SafeLogError<'a, T>(&'a T);

impl<T> std::fmt::Display for SafeLogError<'_, T>
where
    T: ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = <T as ToString>::to_string(self.0);
        let msg_str = msg.trim();
        if msg_str.chars().count() > ERROR_MSG_MAX_LENGTH {
            let msg = msg_str.chars().take(ERROR_MSG_MAX_LENGTH).collect::<String>();
            let msg_str = msg.trim_end();
            write!(f, "{msg_str}...")
        } else {
            write!(f, "{msg_str}")
        }
    }
}

pub trait LogErrorExt: Sized {
    fn truncate(&self) -> SafeLogError<'_, Self>;
}

#[cfg(feature = "jsonrpsee")]
impl LogErrorExt for jsonrpsee_core::ClientError {
    fn truncate(&self) -> SafeLogError<'_, Self> {
        SafeLogError(self)
    }
}
