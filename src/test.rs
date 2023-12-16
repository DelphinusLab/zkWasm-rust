extern "C" {
    //pub fn wasm_input(is_public: u32) -> u64;
    //pub fn wasm_dbg(v:u64);
    pub fn require(cond: bool);
}

use crate::jubjub::BabyJubjubPoint;
use crate::jubjub::JubjubSignature;
use crate::kvpair::KeyValueMap;
use crate::merkle::Merkle;
use crate::wasm_dbg;
use primitive_types::U256;

use crate::poseidon::PoseidonHasher;
use wasm_bindgen::prelude::*;

pub fn test_merkle() {
    let mut hasher = PoseidonHasher::new();
    let data = vec![0x1, 0x1, 2, 2];
    for d in data {
        hasher.update(d);
    }
    let z = hasher.finalize();
    unsafe { wasm_dbg(z[0]) };
    /*
    unsafe {
        require(z[0] == 1);
    }
    */

    let mut merkle = Merkle::new();
    let mut leaf = [0, 0, 0, 0];

    merkle.set(0, &[1, 1, 2, 2], false);

    let len = merkle.get(0, &mut leaf, false);

    unsafe {
        require(len == 4);
        require(leaf == [1, 1, 2, 2]);
    }

    merkle.set(0, &[3, 4, 5, 6, 7], true);
    let mut leaf = [0, 0, 0, 0, 0];

    let len = merkle.get(0, &mut leaf, true);

    unsafe {
        require(len == 5);
        require(leaf == [3, 4, 5, 6, 7]);
    }

    merkle.set_simple(1, &[4, 5, 6, 7]);
    let mut leaf2 = [0, 0, 0, 0];

    merkle.get_simple(1, &mut leaf2);

    unsafe {
        require(leaf2 == [4, 5, 6, 7]);
    }
}

pub fn test_kvpair() {
    let merkle = Merkle::new();
    let mut kvpair = KeyValueMap::new(merkle, true);
    let key1 = [1, 2, 3, 4];
    let key2 = [1, 5, 3, 4];
    let mut data = [0, 0, 0, 0];

    kvpair.set(&key1, &[1]);
    let len = kvpair.get(&key1, &mut data);
    unsafe {
        require(len == 4);
        require(data[0] == 1);
    }

    kvpair.set(&key2, &[2, 3]);
    let len = kvpair.get(&key1, &mut data);
    unsafe {
        require(len == 1);
        require(data[0] == 1);
    }

    let len = kvpair.get(&key2, &mut data);
    unsafe {
        require(len == 2);
        require(data[0] == 2);
        require(data[1] == 3);
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
        test_merkle();
        test_jubjub();
        test_kvpair();
    }
    if true {
        super::witness::test_witness_obj();
    }
    let a = 0;
    super::dbg!("abc{}\n", a);
    0
}
