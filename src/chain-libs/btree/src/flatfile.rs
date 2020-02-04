use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::{fs, path};

pub const SZ_BITS: usize = 24;
pub const POS_BITS: u64 = 40;
pub const MAX_BLOB_SIZE: usize = 2 << SZ_BITS; // 16MB blob
pub const MAX_POS_OFFSET: u64 = 2 << POS_BITS - 1; // last possible position 1byte below 1TB

/// Position of a blob in an appender
///
/// The maximum position is defined as MAX_POS_OFFSET
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(u64);

/// Appender store blob of data (each of maximum size of 16 Mb) offering
/// also a direct access to known index whilst it is appended
pub struct Appender {
    rhandle: fs::File,
    ahandle: fs::File,
}

const MAGIC_SIZE: usize = 8;
const MAGIC: [u8; MAGIC_SIZE] = [0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88];
const DATA_START: u64 = 4096;

impl Appender {
    /// Reopen or create a new appender with the appending file
    pub fn new<P: AsRef<path::Path>>(filename: P) -> Result<Self, io::Error> {
        let filename = filename.as_ref();

        if !filename.exists() {
            let mut f = fs::File::create(&filename)?;
            f.write_all(&MAGIC)?;
            f.set_len(DATA_START)?;
        }

        let mut rhandle = fs::OpenOptions::new().read(true).open(&filename)?;

        let ahandle = {
            let mut ahandle = fs::OpenOptions::new().append(true).open(&filename)?;
            ahandle.seek(SeekFrom::End(0))?;
            ahandle
        };

        let mut buf = [0u8; MAGIC_SIZE];
        rhandle.read_exact(&mut buf)?;
        rhandle.seek(SeekFrom::Start(DATA_START))?;

        if buf != MAGIC {
            return Err(Error::new(ErrorKind::Other, "magic mismatch"));
        }

        Ok(Self { rhandle, ahandle })
    }

    /// Check if this appender can still be appended to.
    pub fn can_append(&mut self) -> Result<bool, io::Error> {
        let pos = self.ahandle.seek(SeekFrom::Current(0))?;
        Ok(pos <= MAX_POS_OFFSET)
    }

    /// Append a blob of data and return the file offset
    ///
    /// Can only append data of MAX_BLOB_SIZE
    pub fn append(&mut self, buf: &[u8]) -> Result<Pos, io::Error> {
        if buf.len() > MAX_BLOB_SIZE {
            return Err(Error::new(ErrorKind::Other, "blob size too big"));
        }
        // if (buf.len() & 0b11) != 0 {
        //     return Err(Error::new(
        //         ErrorKind::Other,
        //         "blob size is not a multiple of 4",
        //     ));
        // }
        let pos = self.ahandle.seek(SeekFrom::Current(0))?;
        if pos > MAX_POS_OFFSET {
            return Err(Error::new(ErrorKind::Other, "offset position too big"));
        }
        let blen = buf.len() as u32;
        let szbuf = blen.to_le_bytes();
        self.ahandle.write_all(&szbuf[..])?;
        self.ahandle.write_all(buf)?;

        // self.ahandle.sync_data()?;
        Ok(Pos(pos))
    }

    /// Get the blob stored at position @pos
    pub fn get_at(&mut self, pos: Pos) -> Result<Box<[u8]>, io::Error> {
        self.rhandle.seek(SeekFrom::Start(pos.0))?;

        let mut szbuf = [0u8; 4];
        self.rhandle.read_exact(&mut szbuf)?;
        let len = u32::from_le_bytes(szbuf);
        let mut v = vec![0u8; len as usize];
        self.rhandle.read_exact(&mut v)?;
        Ok(v.into())
    }

    pub fn sync(&mut self) -> Result<(), io::Error> {
        self.ahandle.sync_data()?;
        Ok(())
    }
}

impl From<Pos> for u64 {
    fn from(pos: Pos) -> u64 {
        pos.0
    }
}

impl From<u64> for Pos {
    fn from(n: u64) -> Pos {
        Pos(n)
    }
}
