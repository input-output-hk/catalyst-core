use cardano_legacy_address::cbor::util::{encode_with_crc32_, raw_with_crc32};
use cbor_event::{
    self,
    de::Deserializer,
    se::{Serialize, Serializer},
};
use criterion::{criterion_group, criterion_main, Criterion};

const CBOR: &[u8] = &[
    0x82, 0xd8, 0x18, 0x53, 0x52, 0x73, 0x6f, 0x6d, 0x65, 0x20, 0x72, 0x61, 0x6e, 0x64, 0x6f, 0x6d,
    0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x1a, 0x71, 0xad, 0x58, 0x36,
];

const BYTES: &[u8] = b"some bytes";

struct Test(&'static [u8]);

impl Serialize for Test {
    fn serialize<'se, W>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>>
    where
        W: std::io::Write,
    {
        serializer.write_bytes(self.0)
    }
}

fn encode_crc32_with_cbor_event(c: &mut Criterion) {
    c.bench_function("encode_crc32_with_cbor_event", |b| {
        b.iter(|| {
            let mut serializer = Serializer::new_vec();
            encode_with_crc32_(&Test(BYTES), &mut serializer).unwrap();
        })
    });
}

fn decode_crc32_with_cbor_event(c: &mut Criterion) {
    c.bench_function("decode_crc32_with_cbor_event", |b| {
        b.iter(|| {
            let mut raw = Deserializer::from(CBOR);
            let _bytes = raw_with_crc32(&mut raw).unwrap();
        })
    });
}

criterion_group!(
    benches,
    encode_crc32_with_cbor_event,
    decode_crc32_with_cbor_event,
);
criterion_main!(benches);
