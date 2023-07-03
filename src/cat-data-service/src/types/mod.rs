pub mod voting_status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerdeType<T>(pub T);

impl<T> From<T> for SerdeType<T> {
    fn from(val: T) -> Self {
        Self(val)
    }
}
