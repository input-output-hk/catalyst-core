use btree::{BTreeStore, FixedSize, Storeable};

use std::io;
use std::io::BufRead;

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
struct Key([u8; 32]);

fn main() -> io::Result<()> {
    let db: BTreeStore<Key> = BTreeStore::open("blocksdb")
        .or_else(|_| BTreeStore::new("blocksdb", 32, 4096))
        .unwrap();

    for line in io::stdin().lock().lines() {
        // TODO: use a real parser?
        let line = line.unwrap();
        let parts = line.split(' ').collect::<Vec<_>>();

        let op = parts[0];
        let id = parts[1];

        let mut decoded: [u8; 32] = [0u8; 32];

        hex::decode_to_slice(id.trim(), &mut decoded as &mut [u8]).expect("invalid hex string");

        match op.trim() {
            "PUT" => {
                let blob = hex::decode(parts[2].trim()).expect("Couldn't decode hexblob");
                // ignore duplicated keys
                db.insert(Key(decoded), &blob).unwrap_or(());
            }
            "GET" => {
                if let Some(v) = db.get(&Key(decoded)).unwrap() {
                    println!("{}", hex::encode(v));
                } else {
                    println!("Key not found :(");
                }
            }
            _ => panic!("Invalid query"),
        };
    }

    Ok(())
}

impl<'a> Storeable<'a> for Key {
    type Error = std::io::Error;
    type Output = Self;

    fn write(&self, buf: &mut [u8]) -> Result<(), Self::Error> {
        buf.copy_from_slice(&self.0[..]);
        Ok(())
    }

    fn read(mut buf: &'a [u8]) -> Result<Self::Output, Self::Error> {
        use std::io::Read;
        let mut bytes = [0u8; 32];
        buf.read_exact(&mut bytes).expect("deserialize failed");
        Ok(Key(bytes))
    }
}

impl FixedSize for Key {
    fn max_size() -> usize {
        std::mem::size_of::<Self>()
    }
}
