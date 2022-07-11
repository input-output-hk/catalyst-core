use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;

pub fn build_path_for_challenge(path: &Path, challenge_name: &str) -> PathBuf {
    let challenge_name = sanitize_name(challenge_name);
    let ext = path.extension();

    let mut path = path.with_extension("").as_os_str().to_owned();
    path.push("_");
    path.push(&*challenge_name);
    let path = PathBuf::from(path);

    match ext {
        Some(ext) => path.with_extension(ext),
        None => path,
    }
}

fn sanitize_name(name: &str) -> Cow<'_, str> {
    static REMOVE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[^-\w.]"#).unwrap());
    static REPLACE_UNDERSCORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#" |:"#).unwrap()); // space or colon
                                                                                                //
    let name = REPLACE_UNDERSCORE_REGEX.replace_all(name, "_");
    match name {
        Cow::Borrowed(borrow) => REMOVE_REGEX.replace_all(borrow, ""),
        Cow::Owned(owned) => {
            let result = REMOVE_REGEX.replace_all(&owned, "");
            Cow::Owned(result.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_correctly() {
        assert_eq!(sanitize_name("asdf"), "asdf");
        // colons and spaces replaced with underscores
        assert_eq!(sanitize_name("a b:c"), "a_b_c");
        // other symbols removed
        assert_eq!(sanitize_name("a£$%^&*()bc"), "abc");
        // . and - are allowed
        assert_eq!(sanitize_name("a.b-c"), "a.b-c");
        // all together
        assert_eq!(sanitize_name("foo$%. bar:baz"), "foo._bar_baz");
    }

    #[test]
    fn test_build_path() {
        let path = "/some/path.ext";
        let challenge = "challenge";
        let built_path = build_path_for_challenge(Path::new(path), challenge);
        assert_eq!(built_path, PathBuf::from("/some/path_challenge.ext"));
    }

    #[test]
    fn test_build_path_hidden_file() {
        let path = "/some/.path.ext";
        let challenge = "challenge";
        let built_path = build_path_for_challenge(Path::new(path), challenge);
        assert_eq!(built_path, PathBuf::from("/some/.path_challenge.ext"));
    }
}
