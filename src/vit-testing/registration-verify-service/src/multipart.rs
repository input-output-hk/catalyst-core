use crate::request::{Request, Source};
use bytes::BufMut;
use futures::TryStreamExt;
use thiserror::Error;
use warp::multipart::Part;
use warp::Buf;

const PIN: &str = "pin";
const PKEY_BYTES: &str = "pkey";
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

    let pin_part: Option<Part> = get_part(PIN).ok();
    let qr_part: Option<Part> = get_part(QR).ok();
    let pub_key_bytes: Option<Part> = get_part(PKEY_BYTES).ok();

    let threshold_part = get_part(THRESHOLD)?;
    let slot_no_part: Option<Part> = get_part(SLOT_NO).ok();
    let funds_part = get_part(FUNDS)?;

    if pin_part.is_none() && qr_part.is_none() && pub_key_bytes.is_none() {
        return Err(Error::EitherQrOrPublicKeyNeedsToBeDefined);
    }

    if pin_part.is_none() && qr_part.is_some() {
        return Err(Error::PinNeedsToBeDefined(PIN.to_string()));
    }

    if pin_part.is_some() && qr_part.is_none() {
        return Err(Error::QrNeedsToBeDefined(QR.to_string()));
    }

    let source: Result<Source, Error> = {
        if let Some(pin_part) = pin_part {
            let qr_part = qr_part.unwrap();

            let qr = readall(qr_part.stream()).await?;

            let pin = String::from_utf8(readall(pin_part.stream()).await?)?;

            if pin.len() > 4 || pin.is_empty() {
                return Err(Error::WrongPinLength(pin));
            }
            if !pin.chars().all(char::is_numeric) {
                return Err(Error::IllegalPinCharacters(pin));
            }

            Ok(Source::Qr { content: qr, pin })
        } else {
            let pub_key_bytes = pub_key_bytes.unwrap();
            let pub_key_string = String::from_utf8(readall(pub_key_bytes.stream()).await?)?;
            Ok(Source::PublicKeyBytes(
                hex::decode(&pub_key_string)
                    .map_err(|_| Error::CannotDecodePublicKey(pub_key_string))?,
            ))
        }
    };

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
        source: source?,
        expected_funds,
        threshold,
        slot_no,
        tag: None,
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
    #[error("either qr with ping or public key needs to be defined")]
    EitherQrOrPublicKeyNeedsToBeDefined,
    #[error("pin part needs to be defined: '{0}'")]
    PinNeedsToBeDefined(String),
    #[error("qr part needs to be defined: '{0}'")]
    QrNeedsToBeDefined(String),
    #[error("cannot decode public key from bytes: '{0}'")]
    CannotDecodePublicKey(String),
}
