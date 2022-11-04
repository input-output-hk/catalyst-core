pub fn rem_first(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.as_str()
}

pub trait StringExtension {
    fn remove_quotas(self) -> Self;
}

impl StringExtension for String {
    fn remove_quotas(self) -> Self {
        #[allow(clippy::single_char_pattern)]
        self.replace("\"", "")
    }
}
