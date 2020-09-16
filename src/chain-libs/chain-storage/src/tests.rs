use crate::{
    test_utils::{Block, BlockId},
    BlockInfo, BlockStore, Error, Value,
};
use rand_core::{OsRng, RngCore};
use std::{collections::HashSet, iter::FromIterator};

const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;
const BLOCK_NUM_PERMANENT_TEST: usize = 1024;
const FLUSH_TO_BLOCK: usize = 512;

pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a [A]) -> &'a A {
    let s = rng.next_u32() as usize;
    // this doesn't need to be uniform
    &v[s % v.len()]
}

pub fn generate_chain<R: RngCore>(rng: &mut R, store: &BlockStore) -> Vec<Block> {
    let mut blocks = vec![];

    let genesis_block = Block::genesis(None);
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();
    blocks.push(genesis_block);

    for _ in 0..10 {
        let mut parent_block = pick_from_vector(rng, &blocks).clone();
        let r = 1 + (rng.next_u32() % 99);
        for _ in 0..r {
            let block = parent_block.make_child(None);
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap();
            parent_block = block.clone();
            blocks.push(block);
        }
    }

    blocks
}

fn prepare_store() -> (tempfile::TempDir, BlockStore) {
    let file = tempfile::TempDir::new().unwrap();
    let store = BlockStore::file(file.path(), BlockId(0).serialize_as_vec()).unwrap();

    (file, store)
}

#[test]
fn tag_get_non_existent() {
    let (_file, store) = prepare_store();
    assert!(store.get_tag("tip").unwrap().is_none());
}

#[test]
fn tag_non_existent_block() {
    let (_file, store) = prepare_store();
    match store.put_tag("tip", &BlockId(0).serialize_as_vec()) {
        Err(Error::BlockNotFound) => {}
        err => panic!(err),
    }
}

#[test]
fn tag_put() {
    let mut rng = OsRng;

    let (_file, store) = prepare_store();
    let blocks = generate_chain(&mut rng, &store);

    store
        .put_tag("tip", &blocks.last().unwrap().id.serialize_as_vec())
        .unwrap();
    assert_eq!(
        store.get_tag("tip").unwrap().unwrap(),
        blocks.last().unwrap().id.serialize_as_value()
    );
}

#[test]
fn tag_overwrite() {
    let mut rng = OsRng;

    let (_file, store) = prepare_store();
    let blocks = generate_chain(&mut rng, &store);

    store
        .put_tag("tip", &blocks.last().unwrap().id.serialize_as_vec())
        .unwrap();
    store
        .put_tag("tip", &blocks.first().unwrap().id.serialize_as_vec())
        .unwrap();
    assert_eq!(
        store.get_tag("tip").unwrap().unwrap(),
        blocks.first().unwrap().id.serialize_as_value()
    );
}

#[test]
fn block_read_write() {
    let (_file, store) = prepare_store();
    let genesis_block = Block::genesis(None);
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );

    assert!(!store
        .block_exists(genesis_block_info.id().as_ref())
        .unwrap());

    store
        .put_block(
            &genesis_block.serialize_as_vec(),
            genesis_block_info.clone(),
        )
        .unwrap();
    let genesis_block_restored = store
        .get_block_info(&genesis_block.id.serialize_as_vec())
        .unwrap();

    assert!(store
        .block_exists(genesis_block_info.id().as_ref())
        .unwrap());

    assert_eq!(
        &genesis_block.id.serialize_as_vec()[..],
        genesis_block_restored.id().as_ref()
    );
    assert_eq!(
        &genesis_block.parent.serialize_as_vec()[..],
        genesis_block_restored.parent_id().as_ref()
    );
    assert_eq!(
        genesis_block.chain_length,
        genesis_block_restored.chain_length()
    );

    assert_eq!(
        vec![genesis_block.id.serialize_as_value()],
        store.get_tips_ids().unwrap()
    );
}

#[test]
pub fn nth_ancestor() {
    let mut rng = OsRng;
    let file = tempfile::TempDir::new().unwrap();
    let store = BlockStore::file(file.path(), BlockId(0).serialize_as_vec()).unwrap();
    let blocks = generate_chain(&mut rng, &store);

    let mut blocks_fetched = 0;
    let mut total_distance = 0;
    let nr_tests = 1000;

    for _ in 0..nr_tests {
        let block = pick_from_vector(&mut rng, &blocks);
        assert_eq!(
            store.get_block(&block.id.serialize_as_vec()).unwrap(),
            block.serialize_as_value()
        );

        let distance = rng.next_u32().checked_rem(block.chain_length).unwrap_or(0);
        total_distance += distance;

        let ancestor_info = store
            .get_nth_ancestor(&block.id.serialize_as_vec(), distance)
            .unwrap();
        blocks_fetched += block.chain_length - distance;

        assert_eq!(ancestor_info.chain_length() + distance, block.chain_length);
    }

    let blocks_per_test = blocks_fetched as f64 / nr_tests as f64;

    println!(
        "fetched {} intermediate blocks ({} per test), total distance {}",
        blocks_fetched, blocks_per_test, total_distance
    );
}

#[test]
fn simultaneous_read_write() {
    let mut rng = OsRng;
    let file = tempfile::TempDir::new().unwrap();
    let store = BlockStore::file(file.path(), BlockId(0).serialize_as_vec()).unwrap();

    let genesis_block = Block::genesis(None);
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();
    let mut blocks = vec![genesis_block];

    for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
        let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
        let block = last_block.make_child(None);
        blocks.push(block.clone());
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap()
    }

    let store_1 = store.clone();
    let blocks_1 = blocks.clone();

    let thread_1 = std::thread::spawn(move || {
        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let block_id = blocks_1
                .get(rng.next_u32() as usize % blocks_1.len())
                .unwrap()
                .id
                .serialize_as_vec();
            store_1.get_block(&block_id).unwrap();
        }
    });

    let thread_2 = std::thread::spawn(move || {
        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child(None);
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap()
        }
    });

    thread_1.join().unwrap();
    thread_2.join().unwrap();
}

#[test]
fn branch_pruning() {
    const MAIN_BRANCH_LEN: usize = 100;
    const SECOND_BRANCH_LEN: usize = 25;
    const BIFURCATION_POINT: usize = 50;

    let file = tempfile::TempDir::new().unwrap();
    let store = BlockStore::file(file.path(), BlockId(0).serialize_as_vec()).unwrap();

    let mut main_branch_blocks = vec![];

    let genesis_block = Block::genesis(None);
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();

    let mut block = genesis_block.make_child(None);

    main_branch_blocks.push(genesis_block);

    for _i in 1..MAIN_BRANCH_LEN {
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap();
        main_branch_blocks.push(block.clone());
        block = block.make_child(None);
    }

    let mut second_branch_blocks = vec![main_branch_blocks[BIFURCATION_POINT].clone()];

    block = main_branch_blocks[BIFURCATION_POINT].make_child(None);

    for _i in 1..SECOND_BRANCH_LEN {
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap();
        second_branch_blocks.push(block.clone());
        block = block.make_child(None);
    }

    let expected_tips = {
        let mut hs = HashSet::new();
        hs.insert(main_branch_blocks.last().unwrap().id.serialize_as_value());
        hs.insert(second_branch_blocks.last().unwrap().id.serialize_as_value());
        hs
    };
    let actual_tips = HashSet::from_iter(store.get_tips_ids().unwrap().into_iter());
    assert_eq!(expected_tips, actual_tips);

    store
        .prune_branch(&second_branch_blocks.last().unwrap().id.serialize_as_vec())
        .unwrap();

    assert_eq!(
        vec![main_branch_blocks.last().unwrap().id.serialize_as_value()],
        store.get_tips_ids().unwrap()
    );

    store
        .get_block(&second_branch_blocks[0].id.serialize_as_vec())
        .unwrap();

    for i in 1..SECOND_BRANCH_LEN {
        let block_result = store.get_block(&second_branch_blocks[i].id.serialize_as_vec());
        assert!(matches!(block_result, Err(Error::BlockNotFound)));
    }

    // tagged branch must not be removed
    store
        .put_tag(
            "tip",
            &main_branch_blocks.last().unwrap().id.serialize_as_vec(),
        )
        .unwrap();
    store
        .prune_branch(&main_branch_blocks.last().unwrap().id.serialize_as_vec())
        .unwrap();
    assert!(store
        .block_exists(&main_branch_blocks.last().unwrap().id.serialize_as_vec())
        .unwrap());
}

#[test]
fn get_blocks_by_chain_length() {
    const N_BLOCKS: usize = 5;

    let file = tempfile::TempDir::new().unwrap();
    let store = BlockStore::file(file.path(), BlockId(0).serialize_as_vec()).unwrap();

    let genesis_block = Block::genesis(None);
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();

    let mut blocks = vec![];

    for _i in 0..N_BLOCKS {
        let block = genesis_block.make_child(None);
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        let block = block.serialize_as_vec();
        store.put_block(&block, block_info).unwrap();
        blocks.push(block);
    }

    let chain_length = genesis_block.chain_length + 1;

    let expected: HashSet<_, std::collections::hash_map::RandomState> = HashSet::from_iter(
        blocks
            .into_iter()
            .map(|block| Value::owned(block.into_boxed_slice())),
    );
    let actual = HashSet::from_iter(
        store
            .get_blocks_by_chain_length(chain_length)
            .unwrap()
            .into_iter(),
    );

    assert_eq!(expected, actual);
}

fn generate_two_branches() -> (tempfile::TempDir, BlockStore, Vec<Block>, Vec<Block>) {
    const MAIN_BRANCH_LEN: usize = 100;
    const SECOND_BRANCH_LEN: usize = 25;
    const BIFURCATION_POINT: usize = 50;

    let (file, store) = prepare_store();

    let mut main_branch_blocks = vec![];

    let genesis_block = Block::genesis(None);
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();

    let mut block = genesis_block.make_child(None);

    main_branch_blocks.push(genesis_block);

    for _i in 1..MAIN_BRANCH_LEN {
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap();
        main_branch_blocks.push(block.clone());
        block = block.make_child(None);
    }

    let mut second_branch_blocks = vec![main_branch_blocks[BIFURCATION_POINT].clone()];

    block = main_branch_blocks[BIFURCATION_POINT].make_child(None);

    for _i in 1..SECOND_BRANCH_LEN {
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap();
        second_branch_blocks.push(block.clone());
        block = block.make_child(None);
    }

    (file, store, main_branch_blocks, second_branch_blocks)
}

#[test]
fn is_ancestor_same_branch() {
    const FIRST: usize = 20;
    const SECOND: usize = 30;

    let (_file, store, main_branch_blocks, _) = generate_two_branches();

    let result = store
        .is_ancestor(
            &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
            &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
        )
        .unwrap()
        .expect("should be a non-None result") as usize;
    assert!(SECOND - FIRST == result);
}

#[test]
fn is_ancestor_wrong_order() {
    const FIRST: usize = 30;
    const SECOND: usize = 20;

    let (_file, store, main_branch_blocks, _) = generate_two_branches();

    let result = store
        .is_ancestor(
            &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
            &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
        )
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn is_ancestor_different_branches() {
    const FIRST: usize = 60;
    const SECOND: usize = 10;

    let (_file, store, main_branch_blocks, second_branch_blocks) = generate_two_branches();

    let result = store
        .is_ancestor(
            &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
            &second_branch_blocks[SECOND].id.serialize_as_vec()[..],
        )
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn is_ancestor_permanent_volatile() {
    const PERMANENT_STORAGE_START: usize = 40;
    const FIRST: usize = 10;
    const SECOND: usize = 50;

    let (_file, store, main_branch_blocks, _) = generate_two_branches();

    store
        .flush_to_permanent_store(
            &main_branch_blocks[PERMANENT_STORAGE_START]
                .id
                .serialize_as_vec(),
        )
        .unwrap();

    let result = store
        .is_ancestor(
            &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
            &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
        )
        .unwrap()
        .expect("should be a non-None result") as usize;
    assert!(SECOND - FIRST == result);
}

#[test]
fn is_ancestor_only_permanent() {
    const PERMANENT_STORAGE_START: usize = 40;
    const FIRST: usize = 10;
    const SECOND: usize = 20;

    let (_file, store, main_branch_blocks, _) = generate_two_branches();

    store
        .flush_to_permanent_store(
            &main_branch_blocks[PERMANENT_STORAGE_START]
                .id
                .serialize_as_vec(),
        )
        .unwrap();

    let result = store
        .is_ancestor(
            &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
            &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
        )
        .unwrap()
        .expect("should be a non-None result") as usize;
    assert!(SECOND - FIRST == result);
}

fn prepare_and_fill_store(n: usize) -> (tempfile::TempDir, BlockStore, Vec<Block>) {
    const BLOCK_DATA_LENGTH: usize = 512;

    let mut rng = OsRng;
    let mut block_data = [0; BLOCK_DATA_LENGTH];

    let file = tempfile::TempDir::new().unwrap();
    let store = BlockStore::file(file.path(), BlockId(0).serialize_as_vec()).unwrap();

    let mut blocks = vec![];

    rng.fill_bytes(&mut block_data);
    let genesis_block = Block::genesis(Some(block_data.clone().to_vec().into_boxed_slice()));
    let genesis_block_info = BlockInfo::new(
        genesis_block.id.serialize_as_vec(),
        genesis_block.parent.serialize_as_vec(),
        genesis_block.chain_length,
    );
    store
        .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
        .unwrap();

    rng.fill_bytes(&mut block_data);
    let mut block = genesis_block.make_child(Some(block_data.clone().to_vec().into_boxed_slice()));

    blocks.push(genesis_block);

    for _i in 1..n {
        let block_info = BlockInfo::new(
            block.id.serialize_as_vec(),
            block.parent.serialize_as_vec(),
            block.chain_length,
        );
        store
            .put_block(&block.serialize_as_vec(), block_info)
            .unwrap();
        blocks.push(block.clone());
        rng.fill_bytes(&mut block_data);
        block = block.make_child(Some(block_data.clone().to_vec().into_boxed_slice()));
    }

    (file, store, blocks)
}

fn prepare_permament_store() -> (tempfile::TempDir, BlockStore, Vec<Block>) {
    let (file, store, blocks) = prepare_and_fill_store(BLOCK_NUM_PERMANENT_TEST);

    store
        .flush_to_permanent_store(&blocks[FLUSH_TO_BLOCK].id.serialize_as_vec())
        .unwrap();

    (file, store, blocks)
}

#[test]
fn permanent_store_read() {
    let (_file, store, blocks) = prepare_permament_store();

    for block in blocks.iter() {
        let block_id = block.id.serialize_as_vec();

        let block_info = store.get_block_info(&block_id).unwrap();
        assert_eq!(&block.id.serialize_as_vec()[..], block_info.id().as_ref());
        assert_eq!(
            &block.parent.serialize_as_vec()[..],
            block_info.parent_id().as_ref()
        );
        assert_eq!(block.chain_length, block_info.chain_length());

        let actual_block = store.get_block(&block_id).unwrap();
        assert_eq!(block.serialize_as_value().as_ref(), actual_block.as_ref());
    }
}

#[test]
fn permanent_store_tag() {
    const TAGS_TEST_LENGTH: usize = 20;

    let (_file, store, blocks) = prepare_permament_store();

    store
        .put_tag("test1", &blocks[TAGS_TEST_LENGTH].id.serialize_as_vec())
        .unwrap();
}

#[test]
fn permanent_store_prune_main_branch() {
    let (_file, store, blocks) = prepare_permament_store();

    store
        .prune_branch(&blocks.last().unwrap().id.serialize_as_vec())
        .unwrap();

    for i in 0..=FLUSH_TO_BLOCK {
        assert!(store
            .block_exists(&blocks[i].id.serialize_as_vec())
            .unwrap());
    }

    for i in (FLUSH_TO_BLOCK + 1)..FLUSH_TO_BLOCK {
        assert!(!store.block_exists(&blocks[i].serialize_as_vec()).unwrap());
    }

    assert_eq!(
        vec![blocks[FLUSH_TO_BLOCK].id.serialize_as_value()],
        store.get_tips_ids().unwrap()
    );
}

#[test]
fn permanent_store_get_by_chain_length() {
    const CHAIN_LENGTH: usize = 20;

    let (_file, store, blocks) = prepare_permament_store();

    let chain_length = blocks[CHAIN_LENGTH].chain_length;
    assert_eq!(
        vec![blocks[CHAIN_LENGTH].serialize_as_value()],
        store.get_blocks_by_chain_length(chain_length).unwrap()
    );
}

#[test]
fn iterator_only_volatile_storage() {
    const TEST_BLOCK_NUM: usize = 32;

    let (_file, store, blocks) = prepare_and_fill_store(TEST_BLOCK_NUM);

    for (i, block) in store
        .iter(
            &blocks[blocks.len() - 1].id.serialize_as_vec()[..],
            TEST_BLOCK_NUM as u32,
        )
        .unwrap()
        .enumerate()
    {
        assert_eq!(blocks[i].serialize_as_value(), block.unwrap());
    }
}

#[test]
fn iterator_volatile_and_permanent_storage() {
    const TEST_BLOCK_NUM: usize = 32;
    const FLUSH_AT: usize = 16;

    let (_file, store, blocks) = prepare_and_fill_store(TEST_BLOCK_NUM);

    store
        .flush_to_permanent_store(&blocks[FLUSH_AT].id.serialize_as_vec()[..])
        .unwrap();

    for (i, block) in store
        .iter(
            &blocks[blocks.len() - 1].id.serialize_as_vec()[..],
            TEST_BLOCK_NUM as u32,
        )
        .unwrap()
        .enumerate()
    {
        assert_eq!(blocks[i].serialize_as_value(), block.unwrap());
    }
}

#[test]
fn iterator_only_permanent_storage() {
    const TEST_BLOCK_NUM: usize = 32;

    let (_file, store, blocks) = prepare_and_fill_store(TEST_BLOCK_NUM);

    store
        .flush_to_permanent_store(&blocks[blocks.len() - 1].id.serialize_as_vec()[..])
        .unwrap();

    for (i, block) in store
        .iter(
            &blocks[blocks.len() - 1].id.serialize_as_vec()[..],
            TEST_BLOCK_NUM as u32,
        )
        .unwrap()
        .enumerate()
    {
        assert_eq!(blocks[i].serialize_as_value(), block.unwrap());
    }
}
