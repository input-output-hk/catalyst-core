use crate::request::Request;
use bytes::BufMut;
use futures::TryStreamExt;
use thiserror::Error;
use warp::multipart::Part;
use warp::Buf;

const PIN: &str = "pin";
const THRESHOLD: &str = "threshold";
const SLOT_NO: &str = "slot-no";
const FUNDS: &str = "funds";
const QR: &str = "qr";

pub async fn parse_multipart(form: warp::multipart::FormData) -> Result<Request, Error> {
    let mut parts: Vec<Part> = form.try_collect().await?;

    let mut get_part = |name: &str| {
        parts
            .iter()
            .position(|part| part.name() == name)
            .map(|p| parts.swap_remove(p))
            .ok_or_else(|| Error::PartNotFound(name.to_string()))
    };

    let pin_part = get_part(PIN)?;
    let threshold_part = get_part(THRESHOLD)?;
    let slot_no_part: Option<Part> = get_part(SLOT_NO).ok();
    let funds_part = get_part(FUNDS)?;
    let qr_part = get_part(QR)?;

    let qr = readall(qr_part.stream()).await?;

    let pin = String::from_utf8(readall(pin_part.stream()).await?)?;

    if pin.len() > 4 || pin.is_empty() {
        return Err(Error::WrongPinLength(pin.to_string()));
    }

    if !pin.chars().all(char::is_numeric) {
        return Err(Error::IllegalPinCharacters(pin.to_string()));
    }

    let expected_funds: u64 = String::from_utf8(readall(funds_part.stream()).await?)?
        .parse()
        .map_err(|_| Error::CannotParsePartIntoInt(FUNDS.to_string()))?;
    let slot_no: Option<u64> = match slot_no_part {
        Some(slot_no_part) => Some(
            String::from_utf8(readall(slot_no_part.stream()).await?)?
                .parse()
                .map_err(|_| Error::CannotParsePartIntoInt(SLOT_NO.to_string()))?,
        ),
        None => None,
    };

    let threshold: u64 = String::from_utf8(readall(threshold_part.stream()).await?)?
        .parse()
        .map_err(|_| Error::CannotParsePartIntoInt(THRESHOLD.to_string()))?;

    if threshold < 1 {
        return Err(Error::WrongThresholdValue(threshold));
    }

    Ok(Request {
        qr,
        pin,
        expected_funds,
        threshold,
        slot_no,
    })
}

async fn readall(
    stream: impl futures::Stream<Item = Result<impl Buf, warp::Error>>,
) -> Result<Vec<u8>, warp::Error> {
    stream
        .try_fold(vec![], |mut result, buf| {
            result.put(buf);
            async move { Ok(result) }
        })
        .await
}

impl warp::reject::Reject for Error {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot find form part {0} (or in case of 'qr' also means that file does not exists)")]
    PartNotFound(String),
    #[error("pin should contains only 4 digits")]
    WrongPinLength(String),
    #[error("illegal character in pin (only digits are allowed). Input: '{0}'")]
    IllegalPinCharacters(String),
    #[error("cannot parse '{0}' into int")]
    CannotParsePartIntoInt(String),
    #[error("threshold should be a positive number")]
    WrongThresholdValue(u64),
    #[error("reading multipart error")]
    ReadingMultipartError(#[from] warp::Error),
    #[error("decoding multipart error")]
    DecodeError(#[from] std::string::FromUtf8Error),
}
