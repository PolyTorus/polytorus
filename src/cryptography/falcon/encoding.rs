use bit_vec::BitVec;
use itertools::Itertools;
use num::Integer;

pub fn compress(v: &[i16], byte_length: usize) -> Option<Vec<u8>> {
    let lengths_and_coefficients = v.iter().map(|c| compress_coefficient(*c)).collect_vec();
    let total_length = lengths_and_coefficients
        .iter()
        .map(|(l, _c)| *l)
        .sum::<usize>();

    if total_length > byte_length * 8 {
        return None;
    }

    if v.is_empty() {
        return None;
    }

    let mut bytes = vec![0u8; byte_length];
    let mut counter = 0;
    for (length, coefficient) in lengths_and_coefficients.iter().take(v.len() - 1) {
        let (cdiv8, cmod8) = counter.div_mod_floor(&8);
        bytes[cdiv8] |= coefficient >> cmod8;
        bytes[cdiv8 + 1] |= ((*coefficient as u16) << (8 - cmod8)) as u8;
        let (cldiv8, clmod8) = (counter + length - 1).div_mod_floor(&8);
        bytes[cldiv8] |= 128u8 >> clmod8;
        bytes[cldiv8 + 1] |= (128u16 << (8 - clmod8)) as u8;
        counter += length;
    }

    let (length, coefficient) = lengths_and_coefficients.last().unwrap();
    {
        let (cdiv8, cmod8) = counter.div_mod_floor(&8);
        bytes[cdiv8] |= coefficient >> cmod8;
        bytes[cdiv8 + 1] |= ((*coefficient as u16) << (8 - cmod8)) as u8;
        let (cldiv8, clmod8) = (counter + length - 1).div_mod_floor(&8);
        bytes[cldiv8] |= 128u8 >> clmod8;
        if cldiv8 + 1 < byte_length {
            bytes[cldiv8 + 1] |= (128u16 << (8 - clmod8)) as u8;
        } else if (128u16 << (8 - clmod8)) as u8 != 0 {
            return None;
        }
        counter += length;
    }
    Some(bytes)
}

fn compress_coefficient(coeff: i16) -> (usize, u8) {
    let sign = (coeff < 0) as u8;
    let abs = coeff.unsigned_abs();
    let low = abs as u8 & 127;
    let high = abs >> 7;
    (1 + 7 + high as usize + 1, ((sign << 7) | low))
}

pub fn decompress(x: &[u8], n: usize) -> Option<Vec<i16>> {
    let bitvector = BitVec::from_bytes(x);
    let mut index = 0;
    let mut result = Vec::with_capacity(n);

    let mut abort = false;

    for _ in 0..n - 1 {
        if index + 8 >= bitvector.len() {
            return None;
        }

        let sign = if bitvector[index] { -1 } else { 1 };
        index += 1;

        let mut low_bits = 0i16;
        let (index_div_8, index_mod_8) = index.div_mod_floor(&8);
        low_bits |= (x[index_div_8] as i16) << index_mod_8;
        low_bits |= (x[index_div_8 + 1] as i16) >> (8 - index_mod_8);
        low_bits = (low_bits & 255) >> 1;
        index += 7;

        let mut high_bits = 0;
        while !bitvector[index] {
            index += 1;
            high_bits += 1;

            if high_bits == 95 || index + 1 == bitvector.len() {
                return None;
            }
        }
        index += 1;

        abort |= low_bits == 0 && high_bits == 0 && sign == -1;

        let integer = sign * ((high_bits << 7) | low_bits);
        result.push(integer);
    }

    if index + 8 >= bitvector.len() {
        return None;
    }

    if bitvector.len() == index {
        return None;
    }
    let sign = if bitvector[index] { -1 } else { 1 };
    index += 1;

    let mut low_bits = 0i16;
    let (index_div_8, index_mod_8) = index.div_mod_floor(&8);
    low_bits |= (x[index_div_8] as i16) << index_mod_8;
    if index_mod_8 != 0 && index_div_8 + 1 < x.len() {
        low_bits |= (x[index_div_8 + 1] as i16) >> (8 - index_mod_8);
    } else if index_mod_8 != 0 {
        return None;
    }
    low_bits = (low_bits & 255) >> 1;
    index += 7;

    let mut high_bits = 0;
    if bitvector.len() == index {
        return None;
    }
    while !bitvector[index] {
        index += 1;
        if bitvector.len() == index {
            return None;
        }
        high_bits += 1;
    }

    if abort || (low_bits == 0 && high_bits == 0 && sign == -1) {
        return None;
    }

    let integer = sign * ((high_bits << 7) | low_bits);
    result.push(integer);

    index += 1;
    let (index_div_8, index_mod_8) = index.div_mod_floor(&8);
    for idx in 0..(8 - index_mod_8) {
        if let Some(b) = bitvector.get(index + idx) {
            if b {
                return None;
            }
        }
    }
    for &byte in x.iter().skip(index_div_8 + 1 - (index_mod_8 == 0) as usize) {
        if byte != 0 {
            return None;
        }
    }

    Some(result)
}
