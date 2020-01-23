use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand_core::{OsRng, RngCore};

use chain_core::property::Block;
use chain_storage::store::{testing::Block as TestBlock, BlockStore};
use chain_storage_sqlite_old::SQLiteBlockStore;

const BLOCK_DATA_LENGTH: usize = 1024;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = OsRng;
    let mut block_data = [0; BLOCK_DATA_LENGTH];

    rng.fill_bytes(&mut block_data);
    let genesis_block = TestBlock::genesis(Some(Box::new(block_data.clone())));

    let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let mut store = SQLiteBlockStore::<TestBlock>::file(path);
    store.put_block(&genesis_block).unwrap();

    let mut blocks = vec![genesis_block];

    c.bench_function("put_block", |b| {
        b.iter_batched(
            || {
                let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
                rng.fill_bytes(&mut block_data);
                let block = last_block.make_child(Some(Box::new(block_data.clone())));
                blocks.push(block.clone());
                block
            },
            |block| store.put_block(&block).unwrap(),
            BatchSize::PerIteration,
        )
    });

    c.bench_function("get_block", |b| {
        b.iter_batched(
            || {
                blocks
                    .get(rng.next_u32() as usize % blocks.len())
                    .unwrap()
                    .id()
            },
            |block_id| store.get_block(&block_id).unwrap(),
            BatchSize::PerIteration,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
