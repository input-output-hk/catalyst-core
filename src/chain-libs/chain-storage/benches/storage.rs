use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand_core::{OsRng, RngCore};

use chain_storage::{
    test_utils::{Block, BlockId},
    BlockInfo, BlockStore,
};

const BLOCK_DATA_LENGTH: usize = 1024;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = OsRng;
    let mut block_data = [0; BLOCK_DATA_LENGTH];

    rng.fill_bytes(&mut block_data);
    let genesis_block = Block::genesis(Some(Box::new(block_data)));

    let tempdir = tempfile::TempDir::new().unwrap();
    let path = {
        let mut path = tempdir.path().to_path_buf();
        path.push("test");
        path
    };
    let store = BlockStore::file(path, BlockId(0).serialize_as_vec()).unwrap();
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();

    let mut blocks = vec![genesis_block];

    c.bench_function("put_block", |b| {
        b.iter_batched(
            || {
                let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
                rng.fill_bytes(&mut block_data);
                let block = last_block.make_child(Some(Box::new(block_data)));
                blocks.push(block.clone());
                block
            },
            |block| {
                let block_info = BlockInfo::new(
                    block.id.serialize_as_vec(),
                    block.parent.serialize_as_vec(),
                    block.chain_length,
                );
                store
                    .put_block(&block.serialize_as_vec(), block_info)
                    .unwrap()
            },
            BatchSize::PerIteration,
        )
    });

    c.bench_function("get_block", |b| {
        b.iter_batched(
            || {
                blocks
                    .get(rng.next_u32() as usize % blocks.len())
                    .unwrap()
                    .id
                    .serialize_as_vec()
            },
            |block_id| store.get_block(&block_id).unwrap(),
            BatchSize::PerIteration,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
