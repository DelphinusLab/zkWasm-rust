extern "C" {
    //pub fn wasm_input(is_public: u32) -> u64;
    //pub fn wasm_dbg(v:u64);
    pub fn require(cond: bool);
    pub fn wasm_trace_size() -> u64;
}

use crate::jubjub::BabyJubjubPoint;
use crate::jubjub::JubjubSignature;
use crate::kvpair::KeyValueMap;
use crate::kvpair::KeyValueMapU64;
use crate::merkle::Merkle;
use primitive_types::U256;

use crate::poseidon::PoseidonHasher;
use wasm_bindgen::prelude::*;

pub fn test_merkle() {
    let mut hasher = PoseidonHasher::new();
    let data = vec![0x1, 0x1, 2, 2];
    for d in data {
        hasher.update(d);
    }
    hasher.finalize();

    let mut merkle = Merkle::new();

    crate::dbg!("testing merkle set 1, index: 0\n");
    merkle.set(0, &[1, 1, 2, 2], false, None);

    let (_, content) = merkle.get(0, false);

    unsafe {
        require(content.len() == 4);
        require(content.as_slice() == [1, 1, 2, 2]);
    }

    crate::dbg!("testing merkle set 2, index: 0\n");
    merkle.set(0, &[3, 4, 5, 6, 7], true, None);

    let (_, content) = merkle.get(0, true);

    unsafe {
        require(content.len() == 5);
        require(content.as_slice() == [3, 4, 5, 6, 7]);
    }

    crate::dbg!("testing merkle set simple, index: 1\n");
    merkle.set_simple(1, &[4, 5, 6, 7], None);
    let mut leaf2 = [0, 0, 0, 0];

    merkle.get_simple(1, &mut leaf2);

    unsafe {
        require(leaf2 == [4, 5, 6, 7]);
    }
}

pub fn test_slice() {
    let mut merkle = Merkle::new();
    let (_, content) = merkle.get(0, true);
    let l = 4096;
    for i in 0..l {
        merkle.set_simple(1, &[4, 5, 6, 7], None);
        let mut leaf2 = [0, 0, 0, 0];

        merkle.get_simple(1, &mut leaf2);

        unsafe {
            require(leaf2 == [4, 5, 6, 7]);
        }
    }
}



fn test_kvpair_value(kvpair: &mut KeyValueMap<Merkle>, key: &[u64; 4], data: &[u64]) {
    let content = kvpair.get(&key);
    unsafe {
        let l1 = content.len();
        let l2 = data.len();
        crate::dbg!("check length {} {}...\n", l1, l2);
        require(content.len() as usize == data.len());
        for i in 0..content.len() as usize {
            let v = content[i];
            let r = data[i];
            crate::dbg!("check value {} {} {} ...\n", i, v, r);
            require(content[i] == data[i]);
        }
    }
}

pub fn test_kvpair() {
    let merkle = Merkle::new();
    let mut kvpair = KeyValueMap::new(merkle);
    let key1 = [1, 2, 3, 4];
    let key2 = [1, 5, 3, 4];
    let key3 = [(1u64 << 32) + 1, 5, 3, 4];
    let key4 = [1, 5, 3, 5];
    let key5 = [1, 5, 3, (2u64 << 32) + 5];
    let key6 = [1, 5, 4, (2u64 << 32) + 5];

    crate::dbg!("testing kvpair key1\n");
    kvpair.set(&key1, &[1]);
    test_kvpair_value(&mut kvpair, &key1, &[1]);

    crate::dbg!("testing kvpair key2 ...\n");
    kvpair.set(&key2, &[2, 3]);
    let root = kvpair.merkle.root;
    crate::dbg!("root is {:?}...\n", root);
    test_kvpair_value(&mut kvpair, &key1, &[1]);
    test_kvpair_value(&mut kvpair, &key2, &[2, 3]);

    crate::dbg!("testing kvpair key3 ...\n");
    kvpair.set(&key3, &[4, 5, 6]);
    let root = kvpair.merkle.root;
    crate::dbg!("root is {:?}...\n", root);
    test_kvpair_value(&mut kvpair, &key1, &[1]);
    test_kvpair_value(&mut kvpair, &key2, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key3, &[4, 5, 6]);

    crate::dbg!("testing kvpair key4 ...\n");
    kvpair.set(&key4, &[7]);
    test_kvpair_value(&mut kvpair, &key1, &[1]);
    test_kvpair_value(&mut kvpair, &key2, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key3, &[4, 5, 6]);
    test_kvpair_value(&mut kvpair, &key4, &[7]);

    crate::dbg!("testing kvpair key5 ...\n");
    kvpair.set(&key5, &[8, 9]);
    //kvpair.set(&key1, &[5]);
    let trace_size = unsafe { wasm_trace_size() };
    kvpair.set(&key1, &[6]);
    let delta_size = unsafe { wasm_trace_size() - trace_size };
    crate::dbg!("delta size is {}\n", delta_size);
    test_kvpair_value(&mut kvpair, &key1, &[6]);
    test_kvpair_value(&mut kvpair, &key2, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key3, &[4, 5, 6]);
    test_kvpair_value(&mut kvpair, &key4, &[7]);
    test_kvpair_value(&mut kvpair, &key5, &[8, 9]);

    let content = kvpair.get(&key6);
    unsafe { require(content.len() == 0) };
}

pub fn test_kvpair_u64() {
    let merkle = Merkle::new();
    let mut kvpair = KeyValueMapU64::new(merkle);
    let count = 4;
    for i in 0..count {
        for j in 0..count {
            let key = i + (j << 32);
            let data = i * 16 + j;
            let trace_size = unsafe { wasm_trace_size() };
            kvpair.set(key, data);
            let delta_size = unsafe { wasm_trace_size() - trace_size };
            crate::dbg!("fill size {}\n", delta_size);
        }
    }

    for i in 0..count {
        for j in 0..count {
            let key = i + (j << 32);
            let data = i * 16 + j;
            let trace_size = unsafe { wasm_trace_size() };
            let data_in = kvpair.get(key);
            let delta_size = unsafe { wasm_trace_size() - trace_size };
            crate::dbg!("get size is {}\n", delta_size);
            if data != data_in {
                crate::dbg!("key {} data {}, data_in {}\n", key, data, data_in);
            }
            unsafe { require(data == data_in) };
        }
    }
}

pub fn test_jubjub() {
    let c = BabyJubjubPoint {
        x: U256([0, 0, 0, 0]),
        y: U256([1, 0, 0, 0]),
    };
    let p = BabyJubjubPoint::msm(vec![(&c, &[1, 0, 0, 0])].as_slice());

    unsafe {
        require(p.x.0[0] == 0);
        require(p.y.0[0] == 1);
    }

    let sig = JubjubSignature {
        sig_r: BabyJubjubPoint {
            x: U256([
                3942246333445170378,
                4927712981048651912,
                7483524259745080053,
                60536396037540871,
            ]),
            y: U256([
                14850245140538961756,
                11076552477444376689,
                6805567804001881962,
                3473463521075824379,
            ]),
        },
        sig_s: [
            13068069613806562103,
            2598268142923890778,
            9227627411507601187,
            303022261472651166,
        ],
    };

    let pk = BabyJubjubPoint {
        x: U256([
            7885996749535148040,
            5452996086172756687,
            10631572794003595355,
            1413880906945024417,
        ]),
        y: U256([
            13330009580783412631,
            14458870954835491754,
            9623332966787297474,
            160649411381582638,
        ]),
    };

    sig.verify(&pk, &[32195221423877958, 0, 0, 0]);
}

#[cfg(feature = "witness")]
mod witness_test {
    use crate::allocator::alloc_witness_memory;
    use crate::require;
    use crate::witness::*;
    use crate::{
        wasm_witness_indexed_insert, wasm_witness_indexed_pop, wasm_witness_indexed_push,
        wasm_witness_insert, wasm_witness_pop, wasm_witness_set_index,
    };
    #[inline(never)]
    pub fn prepare_u64_vec(a: i64) {
        prepare_witness_obj(
            &mut |x| unsafe { wasm_witness_insert(x) },
            |x: &u64| {
                let mut a = vec![];
                for i in 0..2000 {
                    a.push(*x + (i as u64));
                }
                a
            },
            &(a as u64),
        );
    }

    pub fn test_witness_obj() {
        let base_addr = alloc_witness_memory();
        prepare_u64_vec(0);
        let obj = load_witness_obj::<Vec<u64>>(|| unsafe { wasm_witness_pop() }, base_addr);
        let v = unsafe { &*obj };
        for i in 0..100 {
            unsafe { require(v[i] == (i as u64)) };
        }
    }

    pub fn test_witness_indexed(i: u64) {
        unsafe {
            wasm_witness_set_index(i);
            wasm_witness_indexed_push(0x0);
            wasm_witness_indexed_push(0x1);
            wasm_witness_indexed_insert(0x2);
            let a = wasm_witness_indexed_pop();
            require(a == 0x1);
            let a = wasm_witness_indexed_pop();
            require(a == 0x0);
            let a = wasm_witness_indexed_pop();
            require(a == 0x2);
        }
    }

    use derive_builder::WitnessObj;

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    struct TestA {
        a: u64,
        b: u64,
        c: Vec<u64>,
    }

    #[inline(never)]
    pub fn prepare_test_a(a: i64) {
        prepare_witness_obj(
            &mut |x| unsafe { wasm_witness_insert(x) },
            |x: &u64| {
                let mut c = vec![];
                for i in 0..10 {
                    c.push(*x + (i as u64));
                }
                TestA { a: 1, b: 2, c }
            },
            &(a as u64),
        );
    }

    pub fn test_witness_obj_test_a() {
        let base_addr = alloc_witness_memory();
        prepare_test_a(10);
        let obj = load_witness_obj::<TestA>(|| unsafe { wasm_witness_pop() }, base_addr);
        let v = unsafe { &*obj };
        super::super::dbg!("test a is {:?}\n", v);
    }

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    struct TestB {
        a: Vec<TestA>,
        c: Vec<u64>,
        b: u64,
    }

    #[inline(never)]
    pub fn prepare_test_b(a: i64) {
        prepare_witness_obj(
            &mut |x| unsafe { wasm_witness_insert(x) },
            |x: &u64| {
                let mut c = vec![];
                let mut a_array = vec![];
                for _ in 0..3 {
                    for i in 0..10 {
                        c.push(*x + (i as u64));
                    }
                    let a = TestA {
                        a: 1,
                        b: 2,
                        c: c.clone(),
                    };
                    a_array.push(a);
                }
                TestB {
                    a: a_array,
                    b: 3,
                    c,
                }
            },
            &(a as u64),
        );
    }

    pub fn test_witness_obj_test_b() {
        let base_addr = alloc_witness_memory();
        prepare_test_b(0);
        let obj = load_witness_obj::<TestB>(|| unsafe { wasm_witness_pop() }, base_addr);
        let v = unsafe { &*obj };
        super::super::dbg!("test b is {:?}\n", v);
    }

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    pub struct AA {
        x: u64,
    }

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    pub struct BB {
        y: u64,
    }

    #[derive(WitnessObj, PartialEq, Clone, Debug)]
    pub enum EA {
        A(AA),
        B(BB),
    }

    pub fn prepare_test_enum(a: i64) {
        prepare_witness_obj(
            &mut |x| unsafe { wasm_witness_insert(x) },
            |x: &u64| EA::B(BB { y: *x }),
            &(a as u64),
        );
    }

    pub fn test_witness_obj_test_enum() {
        let base_addr = alloc_witness_memory();
        prepare_test_enum(10);
        let obj = load_witness_obj::<EA>(|| unsafe { wasm_witness_pop() }, base_addr);
        let v = unsafe { &*obj };
        super::super::dbg!("test enum_a is {:?}\n", v);
    }

    /*
    pub fn prepare_test_enum_b(a: i64) {
        prepare_witness_obj(
            &mut |x| unsafe { wasm_witness_insert(x) },
            |x: &u64| EB::C(CC{ x: EA::B(BB { y: *x }), y: 255}),
            &(a as u64),
        );
    }

    pub fn test_witness_obj_test_enum_b() {
        let base_addr = alloc_witness_memory();
        prepare_test_enum_b(111);
        let obj = load_witness_obj::<EB>(|| unsafe { wasm_witness_pop() }, base_addr);
        let v = unsafe { &*obj };
        super::super::dbg!("test enum_b is {:?}\n", v);
    }
    */
}

#[wasm_bindgen]
pub fn zkmain() -> i64 {
    if true {
        crate::dbg!("testing merkle\n");
        test_merkle();
        crate::dbg!("testing jubjub\n");
        test_jubjub();
        crate::dbg!("testing kvpair\n");
        test_kvpair();
        crate::dbg!("testing kvpair u64\n");
        test_kvpair_u64();
        crate::dbg!("testing slice\n");
        test_slice();
    }
    if true {
        witness_test::test_witness_obj();
        witness_test::test_witness_obj_test_a();
        witness_test::test_witness_obj_test_b();
        witness_test::test_witness_indexed(0xff);
        witness_test::test_witness_indexed(0x1);
        witness_test::test_witness_indexed(0x2);
        witness_test::test_witness_indexed(0xff);
        witness_test::test_witness_obj_test_enum();
    }
    super::dbg!("test done\n");
    0
}
