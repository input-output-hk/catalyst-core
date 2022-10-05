use crate::cli::kedqr::decode::secret_from_payload;
use crate::cli::kedqr::decode::secret_from_qr;
use crate::cli::kedqr::QrCodeOpts;
use catalyst_toolbox::kedqr::PinReadMode;
use color_eyre::eyre::Context;
use color_eyre::Report;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct VerifyQrCodeCmd {
    #[structopt(long = "folder", required_unless = "file")]
    pub folder: Option<PathBuf>,

    #[structopt(long = "file", required_unless = "folder")]
    pub file: Option<PathBuf>,

    #[structopt(short, long, default_value = "1234")]
    pub pin: String,

    #[structopt(long = "pin-from-file")]
    pub read_pin_from_filename: bool,

    #[structopt(short = "s", long = "stop-at-fail")]
    pub stop_at_fail: bool,

    #[structopt(flatten)]
    opts: QrCodeOpts,
}

impl VerifyQrCodeCmd {
    pub fn exec(&self) -> Result<(), Report> {
        let qr_codes: Vec<PathBuf> = {
            if let Some(file) = &self.file {
                vec![file.to_path_buf()]
            } else {
                std::fs::read_dir(&self.folder.as_ref().unwrap())
                    .unwrap()
                    .into_iter()
                    .map(|x| x.unwrap().path())
                    .collect()
            }
        };

        let mut failed_count = 0;

        for (idx, qr_code) in qr_codes.iter().enumerate() {
            let pin = {
                if self.read_pin_from_filename {
                    PinReadMode::FromFileName(qr_code.clone())
                } else {
                    PinReadMode::Global(self.pin.to_string())
                }
            }
            .into_qr_pin()?;

            let result = match self.opts {
                QrCodeOpts::Payload => secret_from_payload(qr_code, pin),
                QrCodeOpts::Img => secret_from_qr(qr_code, pin),
            };

            if let Err(err) = result {
                if self.stop_at_fail {
                    let qr_path = qr_code.to_path_buf().to_string_lossy().to_string();
                    let index = idx + 1;
                    return Err(err).context(format!("qr_code: {qr_path}, index: {index}"));
                } else {
                    failed_count += 1;
                }
            }
        }
        println!(
            "{} QR read. {} succesfull, {} failed",
            qr_codes.len(),
            qr_codes.len() - failed_count,
            failed_count
        );
        Ok(())
    }
}
