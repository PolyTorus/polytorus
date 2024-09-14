use bit_vec::BitVec;
use itertools::Itertools;
use num_complex::{Complex, Complex64};
use rand::{rngs::StdRng, thread_rng, Rng, RngCore, SeedableRng};

use super::{
    encoding::{compress, decompress},
    field::{Felt, Q},
    fft::FastFft,
    sampling::{ffldl, ffsampling, gram, normalize_tree, LdlTree},
    math::ntru_gen,
    polynomial::{hash_to_point, Polynomial},
};

#[derive(Copy, Clone, Debug)]
pub struct FalconParameters {
    pub n: usize,
    pub sigma: f64,
    pub sigmin: f64,
    pub sig_bound: i64,
    pub sig_bytelen: usize,
}

pub enum FalconVariant {
    Falcon512,
    Falcon1024,
}

impl FalconVariant {
    const fn from_n(n: usize) -> Self {
        match n {
            512 => Self::Falcon512,
            1024 => Self::Falcon1024,
            _ => unreachable!(),
        }
    }
    pub const fn parameters(&self) -> FalconParameters {
        match self {
            FalconVariant::Falcon512 => FalconParameters {
                n: 512,
                sigma: 165.7366171829776,
                sigmin: 1.2778336969128337,
                sig_bound: 34034726,
                sig_bytelen: 666,
            },
            FalconVariant::Falcon1024 => FalconParameters {
                n: 1024,
                sigma: 168.38857144654395,
                sigmin: 1.298280334344292,
                sig_bound: 70265242,
                sig_bytelen: 1280,
            },
        }
    }
}

#[derive(Debug)]
pub enum FalconDeserializationError {
    CannotDetermineFieldElementEncodingMethod,
    CannotInferFalconVariant,
    InvalidHeaderFormat,
    InvalidLogN,
    BadEncodingLength,
    BadFieldElementEncoding,
    WrongVariant,
}

#[derive(Debug, Clone)]
pub struct SecretKey<const N: usize> {
    b0: [Polynomial<i16>; 4],
    tree: LdlTree,
}

impl<const N: usize> SecretKey<N> {
    pub fn generate() -> Self {
        Self::generate_from_seed(thread_rng().gen())
    }

    pub fn generate_from_seed(seed: [u8; 32]) -> Self {
        let b0 = Self::gen_b0(seed);
        Self::from_b0(b0)
    }

    pub fn gen_b0(seed: [u8; 32]) -> [Polynomial<i16>; 4] {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let (f, g, capital_f, capital_g) = ntru_gen(N, &mut rng);
        [g, -f, capital_g, -capital_f]
    }

    pub fn from_b0(b0: [Polynomial<i16>; 4]) -> Self {
        let b0_fft = b0
            .clone()
            .map(|c| c.map(|cc| Complex64::new(*cc as f64, 0.0)).fft());

        let g0_fft = gram(b0_fft);
        let mut tree = ffldl(g0_fft);
        let sigma = FalconVariant::from_n(N).parameters().sigma;
        normalize_tree(&mut tree, sigma);

        SecretKey { b0, tree }
    }

    fn field_element_width(n: usize, polynomial_index: usize) -> usize {
        if polynomial_index == 2 {
            8
        } else {
            match n {
                1024 => 5,
                512 => 6,
                _ => unreachable!(),
            }
        }
    }

    fn serialize_field_element(element_width: usize, element: Felt) -> BitVec {
        let mut bits = BitVec::new();
        let int = element.balanced_value();
        for i in (0..element_width).rev() {
            bits.push(int & (1i16 << i) != 0);
        }
        bits
    }

    fn deserialize_field_element(bits: &BitVec) -> Result<Felt, FalconDeserializationError> {
        if bits[0] && bits.iter().skip(1).all(|b| !b) {
            return Err(FalconDeserializationError::BadFieldElementEncoding);
        }

        let mut uint = 0;
        for bit in bits {
            uint = (uint << 1) | (bit as i16);
        }
        if bits[0] {
            uint = (uint << (16 - bits.len())) >> (16 - bits.len());
        }
        Ok(Felt::new(uint))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let n = self.b0[0].coefficients.len();
        let l = n.checked_ilog2().unwrap() as u8;
        let header: u8 = (5 << 4)
                        | l;

        let f = &self.b0[1];
        let g = &self.b0[0];
        let capital_f = &self.b0[3];

        let mut bits = BitVec::from_bytes(&[header]);
    
        let width = Self::field_element_width(n, 0);
        for &fi in f.coefficients.iter() {
            let mut substring = Self::serialize_field_element(width, Felt::new(-fi));
            bits.append(&mut substring);
        }
        
        let width = Self::field_element_width(n, 1);
        for &fi in g.coefficients.iter() {
            let mut substring = Self::serialize_field_element(width, Felt::new(fi));
            bits.append(&mut substring);
        }
        
        let width = Self::field_element_width(n, 2);
        for &fi in capital_f.coefficients.iter() {
            let mut substring = Self::serialize_field_element(width, Felt::new(-fi));
            bits.append(&mut substring);
        }

        bits.to_bytes()
    }

    
    pub fn from_bytes(byte_vector: &[u8]) -> Result<Self, FalconDeserializationError> {
        
        if byte_vector.len() < 2 {
            return Err(FalconDeserializationError::BadEncodingLength);
        }

        let header = byte_vector[0];
        let bit_buffer = BitVec::from_bytes(&byte_vector[1..]);

        if (header >> 4) != 5 {
            return Err(FalconDeserializationError::InvalidHeaderFormat);
        }

        let logn = (header & 15) as usize;
        let n = match logn {
            9 => 512,
            10 => 1024,
            _ => return Err(FalconDeserializationError::InvalidLogN),
        };

        if n != FalconVariant::from_n(N).parameters().n {
            return Err(FalconDeserializationError::WrongVariant);
        }

        let width_f = Self::field_element_width(n, 0);
        let f = Polynomial::new(
            bit_buffer
                .iter()
                .take(n * width_f)
                .chunks(width_f)
                .into_iter()
                .map(BitVec::from_iter)
                .map(|subs| Self::deserialize_field_element(&subs))
                .collect::<Result<Vec<Felt>, _>>()?,
        );

        let width_g = Self::field_element_width(n, 1);
        let g = Polynomial::new(
            bit_buffer
                .iter()
                .skip(n * width_f)
                .take(n * width_g)
                .chunks(width_g)
                .into_iter()
                .map(BitVec::from_iter)
                .map(|subs| Self::deserialize_field_element(&subs))
                .collect::<Result<Vec<Felt>, _>>()?,
        );

        let width_capital_f = Self::field_element_width(n, 2);
        let capital_f = Polynomial::new(
            bit_buffer
                .iter()
                .skip(n * width_g + n * width_f)
                .take(n * width_capital_f)
                .chunks(width_capital_f)
                .into_iter()
                .map(BitVec::from_iter)
                .map(|subs| Self::deserialize_field_element(&subs))
                .collect::<Result<Vec<Felt>, _>>()?,
        );

        if bit_buffer.len() != n * width_f + n * width_g + n * width_capital_f {
            return Err(FalconDeserializationError::BadEncodingLength);
        }

        let capital_g = g
            .fft()
            .hadamard_div(&f.fft())
            .hadamard_mul(&capital_f.fft())
            .ifft();

        Ok(Self::from_b0([
            g.map(|f| f.balanced_value()),
            -f.map(|f| f.balanced_value()),
            capital_g.map(|f| f.balanced_value()),
            -capital_f.map(|f| f.balanced_value()),
        ]))
    }
}

impl<const N: usize> PartialEq for SecretKey<N> {
    fn eq(&self, other: &Self) -> bool {
        let own_f = &self.b0[1];
        let own_g = &self.b0[0];
        let own_capital_f = &self.b0[3];
        let own_capital_g = &self.b0[2];

        let other_f = &other.b0[1];
        let other_g = &other.b0[0];
        let other_capital_f = &other.b0[3];
        let other_capital_g = &other.b0[2];

        own_f == other_f
            && own_g == other_g
            && own_capital_f == other_capital_f
            && own_capital_g == other_capital_g
    }
}

impl<const N: usize> Eq for SecretKey<N> {}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey<const N: usize> {
    h: Polynomial<Felt>,
}

impl<const N: usize> PublicKey<N> {
    pub fn from_secret_key(sk: &SecretKey<N>) -> Self {
        let f = sk.b0[1].map(|&c| -Felt::new(c));
        let f_ntt = f.fft();
        let g = sk.b0[0].map(|&c| Felt::new(c));
        let g_ntt = g.fft();
        let h_ntt = g_ntt.hadamard_div(&f_ntt);
        let h = h_ntt.ifft();
        Self { h }
    }

    pub fn from_bytes(byte_array: &[u8]) -> Result<Self, FalconDeserializationError> {
        let n: usize = match byte_array.len() {
            897 => 512,
            1793 => 1024,
            _ => return Err(FalconDeserializationError::BadEncodingLength),
        };

        if n != N {
            return Err(FalconDeserializationError::WrongVariant);
        }

        let header = byte_array[0];

        if header >> 4 != 0 {
            return Err(FalconDeserializationError::InvalidHeaderFormat);
        }

        let l = n.ilog2();
        if header != l as u8 {
            return Err(FalconDeserializationError::InvalidLogN);
        }

        let bit_buffer = BitVec::from_bytes(&byte_array[1..]);
        let h = Polynomial::new(
            bit_buffer
                .iter()
                .chunks(14)
                .into_iter()
                .map(|ch| {
                    let mut int = 0;
                    for b in ch {
                        int = (int << 1) | (b as i16);
                    }
                    int
                })
                .map(Felt::new)
                .collect_vec(),
        );

        Ok(PublicKey { h })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.h.coefficients.len().ilog2() as u8;
        let mut bit_buffer = BitVec::from_bytes(&[header]);

        for hi in self.h.coefficients.iter() {
            for i in (0..14).rev() {
                bit_buffer.push(hi.value() & (1 << i) != 0);
            }
        }

        bit_buffer.to_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature<const N: usize> {
    r: [u8; 40],
    s: Vec<u8>,
}

impl<const N: usize> Signature<N> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let felt_encoding = 2;
        let n = self.s.len();
        let l = n.checked_ilog2().unwrap() as u8;
        let header: u8 = (felt_encoding << 5)
                        | (1 << 4)
                        | l;

        [vec![header], self.r.to_vec(), self.s.clone()].concat()
    }

    pub fn from_bytes(byte_vector: &[u8]) -> Result<Self, FalconDeserializationError> {
        let n = if byte_vector.len() == FalconVariant::Falcon512.parameters().sig_bytelen {
            512
        } else if byte_vector.len() == FalconVariant::Falcon1024.parameters().sig_bytelen {
            1024
        } else {
            return Err(FalconDeserializationError::CannotInferFalconVariant);
        };

        if n != N {
            return Err(FalconDeserializationError::WrongVariant);
        }

        let header = byte_vector[0];
        let salt: [u8; 40] = byte_vector[1..=40].try_into().unwrap();
        let signature_vector = &byte_vector[41..];

        let felt_encoding: u8 = 2;
        if (header >> 5) & 3 != felt_encoding {
            return Err(FalconDeserializationError::CannotDetermineFieldElementEncodingMethod);
        }

        if (header >> 7) != 0 || ((header >> 4) & 1) == 0 {
            return Err(FalconDeserializationError::InvalidHeaderFormat);
        }

        let logn = (header & 15) as usize;
        if n != (1 << logn) {
            return Err(FalconDeserializationError::InvalidLogN);
        }

        Ok(Signature::<N> {
            r: salt,
            s: signature_vector.to_vec(),
        })
    }
}

pub fn keygen<const N: usize>(seed: [u8; 32]) -> (SecretKey<N>, PublicKey<N>) {
    let sk = SecretKey::generate_from_seed(seed);
    let pk = PublicKey::from_secret_key(&sk);
    (sk, pk)
}

pub fn sign<const N: usize>(m: &[u8], sk: &SecretKey<N>) -> Signature<N> {
    let mut rng = thread_rng();
    let mut r = [0u8; 40];
    rng.fill_bytes(&mut r);

    let params = FalconVariant::from_n(N).parameters();
    let bound = params.sig_bound;
    let n = params.n;

    let r_cat_m = [r.to_vec(), m.to_vec()].concat();

    let c = hash_to_point(&r_cat_m, n);
    let one_over_q = 1.0 / (Q as f64);
    let c_over_q_fft = c
        .map(|cc| Complex::new(one_over_q * cc.value() as f64, 0.0))
        .fft();

    let capital_f_fft = sk.b0[3].map(|&i| Complex64::new(-i as f64, 0.0)).fft();
    let f_fft = sk.b0[1].map(|&i| Complex64::new(-i as f64, 0.0)).fft();
    let capital_g_fft = sk.b0[2].map(|&i| Complex64::new(i as f64, 0.0)).fft();
    let g_fft = sk.b0[0].map(|&i| Complex64::new(i as f64, 0.0)).fft();
    let t0 = c_over_q_fft.hadamard_mul(&capital_f_fft);
    let t1 = -c_over_q_fft.hadamard_mul(&f_fft);

    let s = loop {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        let bold_s = loop {
            let z = ffsampling(&(t0.clone(), t1.clone()), &sk.tree, &params, &mut rng);
            let t0_min_z0 = t0.clone() - z.0;
            let t1_min_z1 = t1.clone() - z.1;

            let s0 = t0_min_z0.hadamard_mul(&g_fft) + t1_min_z1.hadamard_mul(&capital_g_fft);
            let s1 = t0_min_z0.hadamard_mul(&f_fft) + t1_min_z1.hadamard_mul(&capital_f_fft);

            let length_squared: f64 = (s0
                .coefficients
                .iter()
                .map(|a| (a * a.conj()).re)
                .sum::<f64>()
                + s1.coefficients
                    .iter()
                    .map(|a| (a * a.conj()).re)
                    .sum::<f64>())
                / (n as f64);

            if length_squared > (bound as f64) {
                continue;
            }

            break [s0, s1];
        };
        let s2 = bold_s[1].ifft();
        let maybe_s = compress(
            &s2.coefficients
                .iter()
                .map(|a| a.re.round() as i16)
                .collect_vec(),
            params.sig_bytelen - 41,
        );

        match maybe_s {
            Some(s) => {
                break s;
            }
            None => {
                continue;
            }
        };
    };

    Signature { r, s }
}

pub fn verify<const N: usize>(m: &[u8], sig: &Signature<N>, pk: &PublicKey<N>) -> bool {
    let n = N;
    let params = FalconVariant::from_n(N).parameters();
    let r_cat_m = [sig.r.to_vec(), m.to_vec()].concat();
    let c = hash_to_point(&r_cat_m, n);

    let s2 = match decompress(&sig.s, n) {
        Some(success) => success,
        None => {
            return false;
        }
    };
    let s2_ntt = Polynomial::new(s2.iter().map(|a| Felt::new(*a)).collect_vec()).fft();
    let h_ntt = pk.h.fft();
    let c_ntt = c.fft();

    let s1_ntt = c_ntt - s2_ntt.hadamard_mul(&h_ntt);
    let s1 = s1_ntt.ifft();

    let length_squared = s1
        .coefficients
        .iter()
        .map(|i| i.balanced_value() as i64)
        .map(|i| (i * i))
        .sum::<i64>()
        + s2.iter().map(|&i| i as i64).map(|i| (i * i)).sum::<i64>();
    length_squared < params.sig_bound
}
