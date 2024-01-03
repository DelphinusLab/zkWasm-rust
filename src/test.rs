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
    let mut leaf = [0, 0, 0, 0];

    crate::dbg!("testing merkle set 1, index: 0\n");
    merkle.set(0, &[1, 1, 2, 2], false, None);

    let len = merkle.get(0, &mut leaf, &mut [0; 4], false);

    unsafe {
        require(len == 4);
        require(leaf == [1, 1, 2, 2]);
    }

    crate::dbg!("testing merkle set 2, index: 0\n");
    merkle.set(0, &[3, 4, 5, 6, 7], true, None);
    let mut leaf = [0, 0, 0, 0, 0];

    let len = merkle.get(0, &mut leaf, &mut [0; 4], true);

    unsafe {
        require(len == 5);
        require(leaf == [3, 4, 5, 6, 7]);
    }

    crate::dbg!("testing merkle set simple, index: 1\n");
    merkle.set_simple(1, &[4, 5, 6, 7], None);
    let mut leaf2 = [0, 0, 0, 0];

    merkle.get_simple(1, &mut leaf2);

    unsafe {
        require(leaf2 == [4, 5, 6, 7]);
    }
}

fn test_kvpair_value(
    kvpair: &mut KeyValueMap<Merkle>,
    key: &[u64; 4],
    data_buf: &mut [u64],
    data: &[u64],
) {
    let len = kvpair.get(&key, data_buf);
    unsafe {
        require(len as usize == data.len());
        for i in 0..len as usize {
            require(data_buf[i] == data[i]);
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

    let mut data_buf = [0; 16]; // indicator, 4 for key + 4 for data

    crate::dbg!("testing kvpair key1\n");
    kvpair.set(&key1, &[1]);
    test_kvpair_value(&mut kvpair, &key1, &mut data_buf, &[1]);

    crate::dbg!("testing kvpair key2 ...\n");
    kvpair.set(&key2, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key1, &mut data_buf, &[1]);
    test_kvpair_value(&mut kvpair, &key2, &mut data_buf, &[2, 3]);

    crate::dbg!("testing kvpair key3 ...\n");
    kvpair.set(&key3, &[4, 5, 6]);
    test_kvpair_value(&mut kvpair, &key1, &mut data_buf, &[1]);
    test_kvpair_value(&mut kvpair, &key2, &mut data_buf, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key3, &mut data_buf, &[4, 5, 6]);

    crate::dbg!("testing kvpair key4 ...\n");
    kvpair.set(&key4, &[7]);
    test_kvpair_value(&mut kvpair, &key1, &mut data_buf, &[1]);
    test_kvpair_value(&mut kvpair, &key2, &mut data_buf, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key3, &mut data_buf, &[4, 5, 6]);
    test_kvpair_value(&mut kvpair, &key4, &mut data_buf, &[7]);

    crate::dbg!("testing kvpair key5 ...\n");
    kvpair.set(&key5, &[8, 9]);
    //kvpair.set(&key1, &[5]);
    let trace_size = unsafe { wasm_trace_size() };
    kvpair.set(&key1, &[6]);
    let delta_size = unsafe { wasm_trace_size() - trace_size };
    crate::dbg!("delta size is {}\n", delta_size);
    test_kvpair_value(&mut kvpair, &key1, &mut data_buf, &[6]);
    test_kvpair_value(&mut kvpair, &key2, &mut data_buf, &[2, 3]);
    test_kvpair_value(&mut kvpair, &key3, &mut data_buf, &[4, 5, 6]);
    test_kvpair_value(&mut kvpair, &key4, &mut data_buf, &[7]);
    test_kvpair_value(&mut kvpair, &key5, &mut data_buf, &[8, 9]);

    let len = kvpair.get(&key6, &mut data_buf);
    unsafe { require(len == 0) };
}

pub fn test_kvpair_u64() {
    let merkle = Merkle::new();
    let mut kvpair = KeyValueMapU64::new(merkle);
    let count = 4;
    for i in 0..count {
        for j in 0..count {
            crate::dbg!("fill {} {}\n", i, j);
            let key = i + (j<<32);
            let data = i * 16 + j;
            kvpair.set(key, data);
            crate::dbg!("fill done {} {}\n", i, j);
        }
    }

    for i in 0..count {
        for j in 0..count {
            crate::dbg!("test {} {}", i, j);
            let key = i + (j<<32);
            let data = i * 16 + j;
            let data_in = kvpair.get(key);
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
    }
    if true {
        super::witness::test_witness_obj();
    }
    super::dbg!("test done\n");
    0
}
