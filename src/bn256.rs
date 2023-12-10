use crate::ff::PrimeField;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fr([u64; 4]);

pub fn repr_to_u256(b: <Fr as PrimeField>::Repr) -> [u64; 4] {
    let b = b.as_ref();
    [
        u64::from_le_bytes(b[0..8].try_into().unwrap()),
        u64::from_le_bytes(b[8..16].try_into().unwrap()),
        u64::from_le_bytes(b[16..24].try_into().unwrap()),
        u64::from_le_bytes(b[24..32].try_into().unwrap()),
    ]
}

pub fn repr_from_u256(b: &mut [u8], data: &[u64; 4]) {
    let bi: [u8; 32] = data[0]
        .to_le_bytes()
        .into_iter()
        .chain(data[1].to_le_bytes().into_iter())
        .chain(data[2].to_le_bytes().into_iter())
        .chain(data[3].to_le_bytes().into_iter())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    b.clone_from_slice(&bi);
}
