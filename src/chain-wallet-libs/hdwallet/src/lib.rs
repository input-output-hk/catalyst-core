extern crate bip39;
#[macro_use]
extern crate cbor_event;

pub mod bip44;
pub mod keygen;
pub mod rindex;
pub mod scheme;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
