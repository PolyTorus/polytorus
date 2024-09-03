use std::vec::IntoIter;

use itertools::Itertools;
use num::{BigInt, FromPrimitive, One, Zero};
use num_complex::Complex64;
use rand::RngCore;

use super::{
    cyclotomic_fourier::CyclotomicFourier,
    field::{Felt, Q},
    fft::FastFft,
    inverse::Inverse,
    polynomial::Polynomial,
    sample::sampler_z,
    u32_field::U32Field,
};

#[doc(hidden)]
pub fn babai_reduce_bigint(
    f: &Polynomial<BigInt>,
    g: &Polynomial<BigInt>,
    capital_f: &mut Polynomial<BigInt>,
    capital_g: &mut Polynomial<BigInt>,
) -> Result<(), String> {
    let bitsize = |bi: &BigInt| (bi.bits() + 7) & (u64::MAX ^ 7);
    let n = f.coefficients.len();
    let size = [
        f.map(bitsize).fold(0, |a, &b| u64::max(a, b)),
        g.map(bitsize).fold(0, |a, &b| u64::max(a, b)),
        53,
    ]
    .into_iter()
    .max()
    .unwrap();
    let shift = (size as i64) - 53;
    let f_adjusted = f
        .map(|bi| Complex64::new(i64::try_from(bi >> shift).unwrap() as f64, 0.0))
        .fft();
    let g_adjusted = g
        .map(|bi| Complex64::new(i64::try_from(bi >> shift).unwrap() as f64, 0.0))
        .fft();

    let f_star_adjusted = f_adjusted.map(|c| c.conj());
    let g_star_adjusted = g_adjusted.map(|c| c.conj());
    let denominator_fft =
        f_adjusted.hadamard_mul(&f_star_adjusted) + g_adjusted.hadamard_mul(&g_star_adjusted);

    let mut counter = 0;
    loop {
        let capital_size = [
            capital_f.map(bitsize).fold(0, |a, &b| u64::max(a, b)),
            capital_g.map(bitsize).fold(0, |a, &b| u64::max(a, b)),
            53,
        ]
        .into_iter()
        .max()
        .unwrap();

        if capital_size < size {
            break;
        }
        let capital_shift = (capital_size as i64) - 53;
        let capital_f_adjusted = capital_f
            .map(|bi| Complex64::new(i64::try_from(bi >> capital_shift).unwrap() as f64, 0.0))
            .fft();
        let capital_g_adjusted = capital_g
            .map(|bi| Complex64::new(i64::try_from(bi >> capital_shift).unwrap() as f64, 0.0))
            .fft();

        let numerator = capital_f_adjusted.hadamard_mul(&f_star_adjusted)
            + capital_g_adjusted.hadamard_mul(&g_star_adjusted);
        let quotient = numerator.hadamard_div(&denominator_fft).ifft();

        let k = quotient.map(|f| Into::<BigInt>::into(f.re.round() as i64));

        if k.is_zero() {
            break;
        }
        let kf = (k.clone().karatsuba(f)).reduce_by_cyclotomic(n);
        let shifted_kf = kf.map(|bi| bi << (capital_size - size));
        let kg = (k.clone().karatsuba(g)).reduce_by_cyclotomic(n);
        let shifted_kg = kg.map(|bi| bi << (capital_size - size));

        *capital_f -= shifted_kf;
        *capital_g -= shifted_kg;

        counter += 1;
        if counter > 1000 {
            return Err(format!("Encountered infinite loop in babai_reduce of falcon-rust.\n\
            Please help the developer(s) fix it! You can do this by sending them the inputs to the function that caused the behavior:\n\
            f: {:?}\n\
            g: {:?}\n\
            capital_f: {:?}\n\
            capital_g: {:?}\n", f.coefficients, g.coefficients, capital_f.coefficients, capital_g.coefficients));
        }
    }
    Ok(())
}

#[doc(hidden)]
pub fn babai_reduce_i32(
    f: &Polynomial<i32>,
    g: &Polynomial<i32>,
    capital_f: &mut Polynomial<i32>,
    capital_g: &mut Polynomial<i32>,
) -> Result<(), String> {
    let f_ntt: Polynomial<U32Field> = f.map(|&i| U32Field::new(i)).fft();
    let g_ntt: Polynomial<U32Field> = g.map(|&i| U32Field::new(i)).fft();

    let bitsize = |itr: IntoIter<i32>| {
        (itr.map(|i| i.abs()).max().unwrap() * 2)
            .ilog2()
            .next_multiple_of(8) as usize
    };
    let size = usize::max(
        bitsize(
            f.coefficients
                .iter()
                .chain(g.coefficients.iter())
                .cloned()
                .collect_vec()
                .into_iter(),
        ),
        53,
    );

    let shift = (size as i64) - 53;
    let f_adjusted = f
        .map(|i| Complex64::new(i64::try_from(i >> shift).unwrap() as f64, 0.0))
        .fft();
    let g_adjusted = g
        .map(|i| Complex64::new(i64::try_from(i >> shift).unwrap() as f64, 0.0))
        .fft();

    let f_star_adjusted = f_adjusted.map(|c| c.conj());
    let g_star_adjusted = g_adjusted.map(|c| c.conj());
    let denominator_fft =
        f_adjusted.hadamard_mul(&f_star_adjusted) + g_adjusted.hadamard_mul(&g_star_adjusted);

    let mut counter = 0;
    loop {
        let capital_size = [
            bitsize(
                capital_f
                    .coefficients
                    .iter()
                    .chain(capital_g.coefficients.iter())
                    .copied()
                    .collect_vec()
                    .into_iter(),
            ),
            53,
        ]
        .into_iter()
        .max()
        .unwrap();

        if capital_size < size {
            break;
        }
        let capital_shift = (capital_size as i64) - 53;
        let capital_f_adjusted = capital_f
            .map(|bi| Complex64::new(i64::try_from(bi >> capital_shift).unwrap() as f64, 0.0))
            .fft();
        let capital_g_adjusted = capital_g
            .map(|bi| Complex64::new(i64::try_from(bi >> capital_shift).unwrap() as f64, 0.0))
            .fft();

        let numerator = capital_f_adjusted.hadamard_mul(&f_star_adjusted)
            + capital_g_adjusted.hadamard_mul(&g_star_adjusted);
        let quotient = numerator.hadamard_div(&denominator_fft).ifft();

        let k_ntt = quotient.map(|f| U32Field::new(f.re.round() as i32)).fft();

        if k_ntt.is_zero() {
            break;
        }

        let kf_ntt = k_ntt.hadamard_mul(&f_ntt).ifft();
        let kg_ntt = k_ntt.hadamard_mul(&g_ntt).ifft();

        let kf = kf_ntt.map(|p| p.balanced_value());
        let kg = kg_ntt.map(|p| p.balanced_value());

        *capital_f -= kf;
        *capital_g -= kg;

        counter += 1;
        if counter > 1000 {
            return Err(format!("Encountered infinite loop in babai_reduce of falcon-rust.\n\\
            Please help the developer(s) fix it! You can do this by sending them the inputs to the function that caused the behavior:\n\\
            f: {:?}\n\\
            g: {:?}\n\\
            capital_f: {:?}\n\\
            capital_g: {:?}\n", f.coefficients, g.coefficients, capital_f.coefficients, capital_g.coefficients));
        }
    }
    Ok(())
}

fn xgcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    let (mut old_r, mut r) = (a.clone(), b.clone());
    let (mut old_s, mut s) = (BigInt::one(), BigInt::zero());
    let (mut old_t, mut t) = (BigInt::zero(), BigInt::one());

    while r != BigInt::zero() {
        let quotient = old_r.clone() / r.clone();
        (old_r, r) = (r.clone(), old_r.clone() - quotient.clone() * r);
        (old_s, s) = (s.clone(), old_s.clone() - quotient.clone() * s);
        (old_t, t) = (t.clone(), old_t.clone() - quotient * t);
    }

    (old_r, old_s, old_t)
}

pub fn ntru_solve(f: &Polynomial<BigInt>, g: &Polynomial<BigInt>,) -> Option<(Polynomial<BigInt>, Polynomial<BigInt>)> {
    let n = f.coefficients.len();
    if n == 1 {
        let (gcd, u, v) = xgcd(&f.coefficients[0], &g.coefficients[0]);
        if gcd != BigInt::one() {
            return None;
        }
        return Some((
            (Polynomial::new(vec![-v * BigInt::from_u32(Q).unwrap()])),
            Polynomial::new(vec![u * BigInt::from_u32(Q).unwrap()]),
        ));
    }

    let f_prime = f.field_norm();
    let g_prime = g.field_norm();
    let (capital_f_prime, capital_g_prime) = ntru_solve(&f_prime, &g_prime)?;

    let capital_f_prime_xsq = capital_f_prime.lift_next_cyclotomic();
    let capital_g_prime_xsq = capital_g_prime.lift_next_cyclotomic();
    let f_minx = f.galois_adjoint();
    let g_minx = g.galois_adjoint();

    let mut capital_f = (capital_f_prime_xsq.karatsuba(&g_minx)).reduce_by_cyclotomic(n);
    let mut capital_g = (capital_g_prime_xsq.karatsuba(&f_minx)).reduce_by_cyclotomic(n);

    match babai_reduce_bigint(f, g, &mut capital_f, &mut capital_g) {
        Ok(_) => Some((capital_f, capital_g)),
        Err(_e) => {
            #[cfg(test)]
            {
                panic!("{}", _e);
            }
            #[cfg(not(test))]
            {
                None
            }
        }
    }
}

pub fn ntru_solve_entrypoint(f: Polynomial<i32>, g: Polynomial<i32>,) -> Option<(Polynomial<i32>, Polynomial<i32>)> {
    let n = f.coefficients.len();

    let g_prime = g.field_norm().map(|c| BigInt::from(*c));
    let f_prime = f.field_norm().map(|c| BigInt::from(*c));
    let (capital_f_prime_bi, capital_g_prime_bi) = ntru_solve(&f_prime, &g_prime)?;

    let capital_f_prime_coefficients = capital_f_prime_bi
        .coefficients
        .into_iter()
        .map(|c| i32::try_from(c))
        .collect_vec();
    let capital_g_prime_coefficients = capital_g_prime_bi
        .coefficients
        .into_iter()
        .map(|c| i32::try_from(c))
        .collect_vec();

    if !capital_f_prime_coefficients
        .iter()
        .chain(capital_g_prime_coefficients.iter())
        .all(|c| c.is_ok())
    {
        return None;
    }
    let capital_f_prime = Polynomial::new(
        capital_f_prime_coefficients
            .into_iter()
            .map(|c| c.unwrap().clone())
            .collect_vec(),
    );
    let capital_g_prime = Polynomial::new(
        capital_g_prime_coefficients
            .into_iter()
            .map(|c| c.unwrap().clone())
            .collect_vec(),
    );

    let capital_f_prime_xsq = capital_f_prime.lift_next_cyclotomic();
    let capital_g_prime_xsq = capital_g_prime.lift_next_cyclotomic();
    let f_minx = f.galois_adjoint();
    let g_minx = g.galois_adjoint();

    let psi_rev = U32Field::generate_powers(n, false);
    let psi_rev_inv = U32Field::generate_powers(n, true); 
    let ninv = U32Field::new(n as i32).inverse_or_zero();
    let mut cfp_ntt = capital_f_prime_xsq.map(|c| U32Field::new(*c as i32));
    let mut cgp_ntt = capital_g_prime_xsq.map(|c| U32Field::new(*c as i32));
    let mut gm_ntt = g_minx.map(|c| U32Field::new(*c as i32));
    let mut fm_ntt = f_minx.map(|c| U32Field::new(*c as i32));
    U32Field::fft(&mut cfp_ntt.coefficients, &psi_rev);
    U32Field::fft(&mut cgp_ntt.coefficients, &psi_rev);
    U32Field::fft(&mut gm_ntt.coefficients, &psi_rev);
    U32Field::fft(&mut fm_ntt.coefficients, &psi_rev);
    let mut cf_ntt = cfp_ntt.hadamard_mul(&gm_ntt);
    let mut cg_ntt = cgp_ntt.hadamard_mul(&fm_ntt);
    U32Field::ifft(&mut cf_ntt.coefficients, &psi_rev_inv);
    U32Field::ifft(&mut cg_ntt.coefficients, &psi_rev_inv);

    let mut capital_f = cf_ntt.map(|c| c.balanced_value());
    let mut capital_g = cg_ntt.map(|c| c.balanced_value());

    match babai_reduce_i32(&f, &g, &mut capital_f, &mut capital_g) {
        Ok(_) => Some((capital_f, capital_g)),
        Err(_e) => {
            #[cfg(test)]
            {
                panic!("{}", _e);
            }
            #[cfg(not(test))]
            {
                None
            }
        }
    }
}

#[doc(hidden)]
pub fn ntru_gen(
    n: usize,
    rng: &mut dyn RngCore,
) -> (
    Polynomial<i16>,
    Polynomial<i16>,
    Polynomial<i16>,
    Polynomial<i16>,
) {
    loop {
        let f = gen_poly(n, rng);
        let g = gen_poly(n, rng);

        let f_ntt = f.map(|&i| Felt::new(i)).fft();
        if f_ntt.coefficients.iter().any(|e| e.is_zero()) {
            continue;
        }
        let gamma = gram_schmidt_norm_squared(&f, &g);
        if gamma > 1.3689f64 * (Q as f64) {
            continue;
        }

        if let Some((capital_f, capital_g)) =
            ntru_solve_entrypoint(f.map(|&i| i as i32), g.map(|&i| i as i32))
        {
            return (
                f,
                g,
                capital_f.map(|&i| i as i16),
                capital_g.map(|&i| i as i16),
            );
        }
    }
}

pub fn gen_poly(n: usize, rng: &mut dyn RngCore) -> Polynomial<i16> {
    let mu = 0.0;
    let sigma_star = 1.43300980528773;
    const NUM_COEFFICIENTS: usize = 4096;
    Polynomial {
        coefficients: (0..NUM_COEFFICIENTS)
            .map(|_| sampler_z(mu, sigma_star, sigma_star - 0.001, rng))
            .collect_vec()
            .chunks(NUM_COEFFICIENTS / n)
            .map(|ch| ch.iter().sum())
            .collect_vec(),
    }
}


pub fn gram_schmidt_norm_squared(f: &Polynomial<i16>, g: &Polynomial<i16>) -> f64 {
    let n = f.coefficients.len();
    let norm_f_squared = f.l2_norm_squared();
    let norm_g_squared = g.l2_norm_squared();
    let gamma1 = norm_f_squared + norm_g_squared;

    let f_fft = f.map(|i| Complex64::new(*i as f64, 0.0)).fft();
    let g_fft = g.map(|i| Complex64::new(*i as f64, 0.0)).fft();
    let f_adj_fft = f_fft.map(|c| c.conj());
    let g_adj_fft = g_fft.map(|c| c.conj());
    let ffgg_fft = f_fft.hadamard_mul(&f_adj_fft) + g_fft.hadamard_mul(&g_adj_fft);
    let ffgg_fft_inverse = ffgg_fft.hadamard_inv();
    let qf_over_ffgg_fft = f_adj_fft
        .map(|c| c * (Q as f64))
        .hadamard_mul(&ffgg_fft_inverse);
    let qg_over_ffgg_fft = g_adj_fft
        .map(|c| c * (Q as f64))
        .hadamard_mul(&ffgg_fft_inverse);
    let norm_f_over_ffgg_squared = qf_over_ffgg_fft
        .coefficients
        .iter()
        .map(|c| (c * c.conj()).re)
        .sum::<f64>()
        / (n as f64);
    let norm_g_over_ffgg_squared = qg_over_ffgg_fft
        .coefficients
        .iter()
        .map(|c| (c * c.conj()).re)
        .sum::<f64>()
        / (n as f64);

    let gamma2 = norm_f_over_ffgg_squared + norm_g_over_ffgg_squared;

    f64::max(gamma1, gamma2)
}
