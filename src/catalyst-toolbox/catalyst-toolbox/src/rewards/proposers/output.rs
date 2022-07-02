use std::{
    borrow::Cow,
    fs::File,
    io::BufWriter,
    ops::Deref,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use super::types::Calculation;

pub fn build_path_for_challenge(path: &Path, challenge_name: &str) -> PathBuf {
    let challenge_name = REPLACE_UNDERSCORE_REGEX.replace(challenge_name, "_");
    let challenge_name = REMOVE_REGEX.replace(&challenge_name, "");

    let ext = path.extension();
    let path = path.with_extension("");
    let path = path.join(format!("_{challenge_name}"));
    match ext {
        Some(ext) => path.with_extension(ext),
        None => path,
    }
}

fn sanitize_name<'a>(name: &'a str) -> Cow<'a, str> {
    
    let name = REPLACE_UNDERSCORE_REGEX.replace(name, "_");
    name.make_ascii_uppercas
    let name = REMOVE_REGEX.replace(&name, "");
    name
}

/// Utility method to perform two cow-returning functions in a row
///
/// Usually, the temporary cow is freed when the function returns, but the returned cow references
/// it, causing a "dropped while borrowed" error.
///
/// This function maps
fn double_cow<T: Clone + ?Sized, S: Clone + ?Sized>(
    input: &T,
    first: impl FnOnce(&T) -> Cow<'_, T>,
    second: impl FnOnce(&T) -> Cow<'_, S>,
) -> Cow<'_, S> {
    match first(input) {
        Cow::Borrowed(borrowed) => second(borrowed),
        Cow::Owned(owned) => {
            let result = second(&owned);
            Cow::Owned(S::clone(&result))
        }
    }
}

trait CowExt<'a, T: Clone + ?Sized + 'a> {
        fn map<S: Clone + ?Sized + 'a>(self, f: impl FnOnce(&T) -> Cow<'a, S>) -> Cow<'a, S>;
}

impl<'a, T: Clone + ?Sized + 'a> CowExt<'a, T> for Cow<'a, T> {
        fn map<S: Clone + ?Sized + 'a>(self, f: impl FnOnce(&T) -> Cow<'a, S>) -> Cow<'a, S> {
        match self {
            Cow::Borrowed(borrowed) => f(borrowed),
            Cow::Owned(owned) => {
                let result = f(&owned);
                Cow::Owned(S::clone(&result))
            }
        }
    }
}

static REMOVE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[^-\w.]"#).unwrap());
static REPLACE_UNDERSCORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#" |:"#).unwrap()); // space or colon

pub fn write_json(path: &Path, results: &[Calculation]) -> Result<()> {
    let writer = BufWriter::new(File::options().write(true).open(path)?);
    serde_json::to_writer(writer, &results)?;

    Ok(())
}

pub fn write_csv(path: &Path, results: &[Calculation]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in results {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_simple(before: &str, after: &str) {
        let built_path = build_path_for_challenge(Path::new(""), before);
        assert_eq!(built_path, PathBuf::from(after));
    }

    #[test]
    fn replaces_spaces_and_underscores() {
        check_simple("foo bar:baz", "foo_bar_baz");
    }

    #[test]
    fn test_build_path() {
        let path = "/some/path.ext";
        let challenge = "challenge";
        let built_path = build_path_for_challenge(Path::new(path), challenge);
        assert_eq!(built_path, PathBuf::from("/some/path_challenge.ext"));
    }
}
