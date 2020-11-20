use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand_core::{OsRng, RngCore};

use chain_storage::{
    test_utils::{Block, BlockId},
    BlockInfo, BlockStore,
};

const BLOCK_DATA_LENGTH: usize = 1024;
const SEQ_BENCH_N_BLOCKS: u32 = 5210;
const SEQ_BENCH_FLUSH_POINT: u32 = 4096;

fn basic_benchmark(c: &mut Criterion) {
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

fn seq_read_benchmark(c: &mut Criterion) {
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

    for _i in 0..SEQ_BENCH_N_BLOCKS {
        let last_block = blocks.last().unwrap();
        rng.fill_bytes(&mut block_data);
        let block = last_block.make_child(Some(Box::new(block_data)));
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap();
        blocks.push(block);
    }

    let block_ids: Vec<_> = blocks
        .iter()
        .map(|block| block.id.serialize_as_vec())
        .collect();

    c.bench_function("seq_volatile_get_block", |b| {
        b.iter(|| {
            for block_id in block_ids.iter() {
                store.get_block(&block_id).unwrap();
            }
        })
    });

    c.bench_function("seq_volatile_iter", |b| {
        b.iter(|| {
            for block_res in store
                .iter(block_ids.last().unwrap().as_ref(), SEQ_BENCH_N_BLOCKS)
                .unwrap()
            {
                let _block = block_res.unwrap();
            }
        })
    });

    store
        .flush_to_permanent_store(&block_ids[SEQ_BENCH_FLUSH_POINT as usize], 1)
        .unwrap();

    c.bench_function("seq_mixed_get_block", |b| {
        b.iter(|| {
            for block_id in block_ids.iter() {
                store.get_block(&block_id).unwrap();
            }
        })
    });

    c.bench_function("seq_mixed_iter", |b| {
        b.iter(|| {
            for block_res in store
                .iter(block_ids.last().unwrap().as_ref(), SEQ_BENCH_N_BLOCKS)
                .unwrap()
            {
                let _block = block_res.unwrap();
            }
        })
    });

    store
        .flush_to_permanent_store(&block_ids[SEQ_BENCH_N_BLOCKS as usize - 1], 1)
        .unwrap();

    c.bench_function("seq_permanent_get_block", |b| {
        b.iter(|| {
            for block_id in block_ids.iter() {
                store.get_block(&block_id).unwrap();
            }
        })
    });

    c.bench_function("seq_permanent_iter", |b| {
        b.iter(|| {
            for block_res in store
                .iter(block_ids.last().unwrap().as_ref(), SEQ_BENCH_N_BLOCKS)
                .unwrap()
            {
                let _block = block_res.unwrap();
            }
        })
    });
}

criterion_group!(benches, basic_benchmark, seq_read_benchmark);
criterion_main!(benches);
