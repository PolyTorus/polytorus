use std::f64::consts::LN_2;

use rand::{Rng, RngCore};

pub fn base_sampler(bytes: [u8; 9]) -> i16 {
    const RCDT: [u128; 18] = [
        3024686241123004913666,
        1564742784480091954050,
        636254429462080897535,
        199560484645026482916,
        47667343854657281903,
        8595902006365044063,
        1163297957344668388,
        117656387352093658,
        8867391802663976,
        496969357462633,
        20680885154299,
        638331848991,
        14602316184,
        247426747,
        3104126,
        28824,
        198,
        1,
    ];
    let u = u128::from_be_bytes([vec![0u8; 7], bytes.to_vec()].concat().try_into().unwrap());
    RCDT.into_iter().filter(|r| u < *r).count() as i16
}

pub fn approx_exp(x: f64, ccs: f64) -> u64 {
    const C: [u64; 13] = [
        0x00000004741183A3u64,
        0x00000036548CFC06u64,
        0x0000024FDCBF140Au64,
        0x0000171D939DE045u64,
        0x0000D00CF58F6F84u64,
        0x000680681CF796E3u64,
        0x002D82D8305B0FEAu64,
        0x011111110E066FD0u64,
        0x0555555555070F00u64,
        0x155555555581FF00u64,
        0x400000000002B400u64,
        0x7FFFFFFFFFFF4800u64,
        0x8000000000000000u64,
    ];

    let mut z: u64;
    let mut y: u64;
    let twoe63 = 1u64 << 63;

    y = C[0];
    z = f64::floor(x * (twoe63 as f64)) as u64;
    for cu in C.iter().skip(1) {
        let zy = (z as u128) * (y as u128);
        y = cu - ((zy >> 63) as u64);
    }

    z = f64::floor((twoe63 as f64) * ccs) as u64;

    (((z as u128) * (y as u128)) >> 63) as u64
}


pub fn ber_exp(x: f64, ccs: f64, random_bytes: [u8; 7]) -> bool {
    let s = f64::floor(x / LN_2) as usize;
    let r = x - LN_2 * (s as f64);
    let shamt = usize::min(s, 63);
    let z = ((((approx_exp(r, ccs) as u128) << 1) - 1) >> shamt) as u64;
    let mut w = 0i16;
    for (index, i) in (0..64).step_by(8).rev().enumerate() {
        let byte = random_bytes[index];
        w = (byte as i16) - (((z >> i) & 0xff) as i16);
        if w != 0 {
            break;
        }
    }
    w < 0
}

pub fn sampler_z(mu: f64, sigma: f64, sigma_min: f64, rng: &mut dyn RngCore) -> i16 {
    const SIGMA_MAX: f64 = 1.8205;
    const INV_2SIGMA_MAX_SQ: f64 = 1f64 / (2f64 * SIGMA_MAX * SIGMA_MAX);
    let isigma = 1f64 / sigma;
    let dss = 0.5f64 * isigma * isigma;
    let s = f64::floor(mu);
    let r = mu - s;
    let ccs = sigma_min * isigma;
    loop {
        let z0 = base_sampler(rng.gen());
        let random_byte: u8 = rng.gen();
        let b = (random_byte & 1) as i16;
        let z = b + ((b << 1) - 1) * z0;
        let zf_min_r = (z as f64) - r;
        //    x = ((z-r)^2)/(2*sigma^2) - ((z-b)^2)/(2*sigma0^2)
        let x = zf_min_r * zf_min_r * dss - (z0 * z0) as f64 * INV_2SIGMA_MAX_SQ;
        if ber_exp(x, ccs, rng.gen()) {
            return z + (s as i16);
        }
    }
}
