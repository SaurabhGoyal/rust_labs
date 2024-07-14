use sha1::{Digest, Sha1};

pub fn run() {
    let data1 = [8_u8; 700];
    let data2 = [9_u8; 500];
    let data3 = [10_u8; 400];
    let mut data = [0_u8; 1600];
    data[..700].copy_from_slice(&data1);
    data[700..1200].copy_from_slice(&data2);
    data[1200..].copy_from_slice(&data3);

    let mut hasher = Sha1::new();
    let mut hash_full: [u8; 20] = [0; 20];
    hasher.update(&data[..]);
    hash_full.copy_from_slice(&hasher.finalize()[..]);
    println!("Full {:?}", hash_full);

    let mut hasher = Sha1::new();
    let mut hash_parts: [u8; 20] = [0; 20];
    hasher.update(&data[..675]);
    hasher.update(&data[675..1020]);
    hasher.update(&data[1020..]);
    hash_parts.copy_from_slice(&hasher.finalize()[..]);
    println!("In parts {:?}", hash_parts);

    assert_eq!(hash_full, hash_parts);
}
