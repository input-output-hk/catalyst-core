#![allow(dead_code)]

mod bitmap;
mod hamt;
mod hash;
mod helper;
mod node;
mod operation;
mod sharedref;

pub use hamt::*;

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::hash_map::DefaultHasher;
    use std::collections::BTreeMap;
    use std::convert::Infallible;
    use std::hash::Hash;

    use proptest::collection::size_range;
    use proptest::prelude::*;
    use test_strategy::proptest;

    #[derive(Debug, Clone, test_strategy::Arbitrary)]
    enum PlanOperation {
        Insert(Vec<u8>, u32),
        DeleteOneMatching(usize),
        DeleteOne(usize),
        Update(usize),
        UpdateRemoval(usize),
        Replace(usize, u32),
        ReplaceWith(usize),
    }

    #[test]
    fn insert_lookup() {
        let h: Hamt<DefaultHasher, String, u32> = Hamt::new();

        let k1 = "ABC".to_string();
        let v1 = 12u32;

        let k2 = "DEF".to_string();
        let v2 = 24u32;

        let k3 = "XYZ".to_string();
        let v3 = 42u32;

        let h1: Hamt<DefaultHasher, String, u32> = h.insert(k1.clone(), v1).unwrap();
        let h2 = h.insert(k2.clone(), v2).unwrap();

        assert_eq!(h1.lookup(&k1), Some(&v1));
        assert_eq!(h2.lookup(&k2), Some(&v2));
        assert_eq!(h1.lookup(&k2), None);
        assert_eq!(h2.lookup(&k1), None);

        let h3 = h1.insert(k3.clone(), v3).unwrap();

        assert_eq!(h1.lookup(&k3), None);
        assert_eq!(h2.lookup(&k3), None);

        assert_eq!(h3.lookup(&k1), Some(&v1));
        assert_eq!(h3.lookup(&k2), None);
        assert_eq!(h3.lookup(&k3), Some(&v3));

        let (h4, oldk1) = h3.replace(&k1, v3).unwrap();
        assert_eq!(oldk1, v1);
        assert_eq!(h4.lookup(&k1), Some(&v3));
    }

    #[test]
    fn dup_insert() {
        let mut h: Hamt<DefaultHasher, &String, u32> = Hamt::new();
        let dkey = "A".to_string();
        h = h.insert(&dkey, 1).unwrap();
        assert_eq!(
            h.insert(&dkey, 2).and(Ok(())),
            Err(InsertError::EntryExists)
        )
    }

    #[test]
    fn empty_size() {
        let h: Hamt<DefaultHasher, &String, u32> = Hamt::new();
        assert_eq!(h.size(), 0)
    }

    #[test]
    fn delete_key_not_exist() {
        let mut h: Hamt<DefaultHasher, &String, u32> = Hamt::new();
        let dkey = "A".to_string();
        h = h.insert(&dkey, 1).unwrap();
        assert_eq!(
            h.remove_match(&&dkey, &2).and(Ok(())),
            Err(RemoveError::ValueNotMatching)
        )
    }

    #[test]
    fn delete_value_not_match() {
        let mut h: Hamt<DefaultHasher, &String, u32> = Hamt::new();
        let dkey = "A".to_string();
        h = h.insert(&dkey, 1).unwrap();
        assert_eq!(
            h.remove_match(&&dkey, &2).and(Ok(())),
            Err(RemoveError::ValueNotMatching)
        )
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn next_u32(x: &u32) -> Result<Option<u32>, Infallible> {
        Ok(Some(*x + 1))
    }

    #[test]
    fn delete() {
        let mut h: Hamt<DefaultHasher, String, u32> = Hamt::new();

        let keys = [
            ("KEY1", 10000u32),
            ("KEY2", 20000),
            ("KEY3", 30000),
            ("KEY4", 40000),
            ("KEY5", 50000),
            ("KEY6", 60000),
            ("KEY7", 70000),
            ("KEY8", 80000),
            ("KEY9", 10000),
            ("KEY10", 20000),
            ("KEY11", 30000),
            ("KEY12", 40000),
            ("KEY13", 50000),
            ("KEY14", 60000),
            ("KEY15", 70000),
            ("KEY16", 80000),
        ];

        let k1 = "ABC".to_string();
        let v1 = 12u32;

        let k2 = "DEF".to_string();
        let v2 = 24u32;

        let k3 = "XYZ".to_string();
        let v3 = 42u32;

        for (k, v) in keys.iter() {
            h = h.insert((*k).to_owned(), *v).unwrap();
        }

        h = h.insert(k1.clone(), v1).unwrap();
        h = h.insert(k2.clone(), v2).unwrap();
        h = h.insert(k3.clone(), v3).unwrap();

        let h2 = h
            .remove_match(&k1, &v1)
            .expect("cannot remove from already inserted");

        assert_eq!(h.size(), keys.len() + 3);
        assert_eq!(h2.size(), keys.len() + 2);

        assert_eq!(h.lookup(&k1), Some(&v1));
        assert_eq!(h2.lookup(&k1), None);

        h = h.remove_match(&k2, &v2).unwrap();
        h = h.remove_match(&k3, &v3).unwrap();

        assert_eq!(
            h.remove_match(&k3, &v3).and(Ok(())),
            Err(RemoveError::KeyNotFound),
        );
        assert_eq!(
            h.remove_match(&k1, &v2).and(Ok(())),
            Err(RemoveError::ValueNotMatching),
        );
        assert_eq!(h2.insert(k2, v3).and(Ok(())), Err(InsertError::EntryExists));

        assert_eq!(
            h2.update(&"ZZZ".to_string(), next_u32).and(Ok(())),
            Err(UpdateError::KeyNotFound)
        );

        assert_eq!(h.size(), keys.len() + 1);
        assert_eq!(h2.size(), keys.len() + 2);
    }

    //use hash::HashedKey;
    //use std::marker::PhantomData;

    /* commented -- as this doesn't do what it says on the tin.
    it doesn't test for h collision, but node splitting

    #[test]
    fn collision() {
        let k0 = "keyx".to_string();
        let h1 = HashedKey::compute(PhantomData::<DefaultHasher>, &k0);
        let l = h1.level_index(0);
        let mut found = None;
        for i in 0..10000 {
            let x = format!("key{}", i);
            let h2 = HashedKey::compute(PhantomData::<DefaultHasher>, &x);
            if h1 == h2 {
            //if h2.level_index(0) == l {
                found = Some(x.clone());
                break;
            }
        }

        match found {
            None => assert!(false),
            Some(x) => {
                let mut h: Hamt<DefaultHasher, String, u32> = Hamt::new();
                println!("k0: {}", k0);
                h = h.insert(k0.clone(), 1u32).unwrap();
                println!("x: {}", x);
                h = h.insert(x.clone(), 2u32).unwrap();
                assert_eq!(h.size(), 2);
                assert_eq!(h.lookup(&k0), Some(&1u32));
                assert_eq!(h.lookup(&x), Some(&2u32));

                let h2 = h.remove_match(&x, &2u32).unwrap();
                assert_eq!(h2.lookup(&k0), Some(&1u32));
                assert_eq!(h2.size(), 1);

                let h3 = h.remove_match(&k0, &1u32).unwrap();
                assert_eq!(h3.lookup(&x), Some(&2u32));
                assert_eq!(h3.size(), 1);
            }
        }
    }
    */

    fn property_btreemap_eq<A: Eq + Ord + Hash, B: PartialEq>(
        reference: &BTreeMap<A, B>,
        h: &Hamt<DefaultHasher, A, B>,
    ) -> bool {
        // using the btreemap reference as starting point
        for (k, v) in reference.iter() {
            if h.lookup(k) != Some(v) {
                return false;
            }
        }
        // then asking the hamt for any spurious values
        for (k, v) in h.iter() {
            if reference.get(k) != Some(v) {
                return false;
            }
        }
        true
    }

    #[proptest]
    fn insert_equivalent(#[any(size_range(..2000).lift())] xs: Vec<(Vec<u8>, u32)>) {
        let mut reference = BTreeMap::new();
        let mut h: Hamt<DefaultHasher, Vec<u8>, u32> = Hamt::new();
        for (k, v) in xs.iter() {
            if reference.get(k).is_some() {
                continue;
            }
            reference.insert(k.clone(), *v);
            h = h.insert(k.clone(), *v).unwrap();
        }

        prop_assert!(property_btreemap_eq(&reference, &h));
    }

    fn get_key_nth<K: Clone, V>(b: &BTreeMap<K, V>, n: usize) -> Option<K> {
        let keys_nb = b.len();
        if keys_nb == 0 {
            return None;
        }
        let mut keys = b.keys();
        Some(keys.nth(n % keys_nb).unwrap().clone())
    }

    #[allow(clippy::type_complexity)]
    fn plan_to_hamt_and_btree(
        xs: Vec<PlanOperation>,
    ) -> (Hamt<DefaultHasher, Vec<u8>, u32>, BTreeMap<Vec<u8>, u32>) {
        let mut reference = BTreeMap::new();
        let mut h: Hamt<DefaultHasher, Vec<u8>, u32> = Hamt::new();
        //println!("plan {} operations", xs.0.len());
        for op in xs.iter() {
            match op {
                PlanOperation::Insert(k, v) => {
                    if reference.get(k).is_some() {
                        continue;
                    }
                    reference.insert(k.clone(), *v);
                    h = h.insert(k.clone(), *v).unwrap();
                }
                PlanOperation::DeleteOne(r) => match get_key_nth(&reference, *r) {
                    None => continue,
                    Some(k) => {
                        reference.remove(&k);
                        h = h.remove(&k).unwrap();
                    }
                },
                PlanOperation::DeleteOneMatching(r) => match get_key_nth(&reference, *r) {
                    None => continue,
                    Some(k) => {
                        let v = *reference.get(&k).unwrap();
                        reference.remove(&k);
                        h = h.remove_match(&k, &v).unwrap();
                    }
                },
                PlanOperation::Replace(r, newv) => match get_key_nth(&reference, *r) {
                    None => continue,
                    Some(k) => {
                        let v = reference.get_mut(&k).unwrap();
                        *v = *newv;

                        h = h.replace(&k, *newv).unwrap().0;
                    }
                },
                PlanOperation::ReplaceWith(r) => match get_key_nth(&reference, *r) {
                    None => continue,
                    Some(k) => {
                        let v = reference.get_mut(&k).unwrap();
                        *v = v.wrapping_mul(2);

                        h = h.replace_with(&k, |v| v.wrapping_mul(2)).unwrap();
                    }
                },
                PlanOperation::Update(r) => match get_key_nth(&reference, *r) {
                    None => continue,
                    Some(k) => {
                        let v = reference.get_mut(&k).unwrap();
                        match next_u32(v).unwrap() {
                            None => {
                                reference.remove(&k);
                            }
                            Some(newv) => *v = newv,
                        }

                        h = h.update(&k, next_u32).unwrap();
                    }
                },
                PlanOperation::UpdateRemoval(r) => match get_key_nth(&reference, *r) {
                    None => continue,
                    Some(k) => {
                        reference.remove(&k);
                        h = h.update::<_, Infallible>(&k, |_| Ok(None)).unwrap();
                    }
                },
            }
        }
        (h, reference)
    }

    fn arbitrary_hamt_and_btree(
    ) -> impl Strategy<Value = (Hamt<DefaultHasher, Vec<u8>, u32>, BTreeMap<Vec<u8>, u32>)> {
        any_with::<Vec<PlanOperation>>(size_range(0..4096).lift()).prop_map(plan_to_hamt_and_btree)
    }

    // TODO use pattern matching in args after supporting it in test-strategy

    #[proptest]
    fn plan_equivalent(
        #[allow(clippy::type_complexity)]
        #[strategy(arbitrary_hamt_and_btree())]
        data: (Hamt<DefaultHasher, Vec<u8>, u32>, BTreeMap<Vec<u8>, u32>),
    ) {
        let (h, reference) = data;
        prop_assert!(property_btreemap_eq(&reference, &h));
    }

    #[proptest]
    #[allow(clippy::type_complexity)]
    fn iter_equivalent(
        #[allow(clippy::type_complexity)]
        #[strategy(arbitrary_hamt_and_btree())]
        data: (Hamt<DefaultHasher, Vec<u8>, u32>, BTreeMap<Vec<u8>, u32>),
    ) {
        use std::iter::FromIterator;
        let (h, reference) = data;
        let after_iter = BTreeMap::from_iter(h.iter().map(|(k, v)| (k.clone(), *v)));
        prop_assert_eq!(reference, after_iter);
    }
}
