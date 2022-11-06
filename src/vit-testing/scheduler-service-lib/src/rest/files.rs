use crate::context::SharedContext;
use crate::{dump_json, FileListerError};
use std::path::PathBuf;
use std::sync::PoisonError;
use warp::reject::Reject;
use warp::{Filter, Rejection, Reply};

impl Reject for Error {}

pub fn filter(
    context: SharedContext,
    path: PathBuf,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let root = warp::path!("files" / ..).boxed();

    let get = warp::path("get").and(warp::fs::dir(path));
    let list = warp::path!("list")
        .and(warp::get())
        .and(with_context)
        .and_then(files_handler);
    root.and(get.or(list)).boxed()
}

pub async fn files_handler(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(dump_json(
        context
            .read()
            .await
            .working_directory()
            .clone()
            .ok_or(Error::NoWorkingDir)?,
    )
    .map(|r| warp::reply::json(&r))?)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] FileListerError),
    #[error("cannot acquire lock on context")]
    Poison,
    #[error("cannot list files dur to configuration issue: no working directory defined")]
    NoWorkingDir,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Self {
        Self::Poison
    }
}
