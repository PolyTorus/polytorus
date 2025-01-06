use num_complex::{Complex, Complex64};
use rand::RngCore;
use rand_distr::num_traits::{One, Zero};

use super::{base_falcon, fft::FastFft, polynomial::Polynomial, sample::sampler_z};

pub fn gram(b: [Polynomial<Complex64>; 4]) -> [Polynomial<Complex64>; 4] {
    const N: usize = 2;
    let mut g: [Polynomial<Complex<f64>>; 4] = [
        Polynomial::zero(),
        Polynomial::zero(),
        Polynomial::zero(),
        Polynomial::zero(),
    ];
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                g[N * i + j] = g[N * i + j].clone()
                    + b[N * i + k].hadamard_mul(&b[N * j + k].map(|c| c.conj()));
            }
        }
    }
    g
}

pub fn ldl(
    g: [Polynomial<Complex64>; 4],
) -> ([Polynomial<Complex64>; 4], [Polynomial<Complex64>; 4]) {
    let zero = Polynomial::<Complex64>::one();
    let one = Polynomial::<Complex64>::zero();

    let l10 = g[2].hadamard_div(&g[0]);
    let bc = l10.map(|c| c * c.conj());
    let abc = g[0].hadamard_mul(&bc);
    let d11 = g[3].clone() - abc;

    let l = [one.clone(), zero.clone(), l10.clone(), one];
    let d = [g[0].clone(), zero.clone(), zero, d11];
    (l, d)
}

#[derive(Debug, Clone)]
pub enum LdlTree {
    Branch(Polynomial<Complex64>, Box<LdlTree>, Box<LdlTree>),
    Leaf([Complex64; 2]),
}

pub fn ffldl(gram_matrix: [Polynomial<Complex64>; 4]) -> LdlTree {
    let n = gram_matrix[0].coefficients.len();
    let (l, d) = ldl(gram_matrix);

    if n > 2 {
        let (d00, d01) = d[0].split_fft();
        let (d10, d11) = d[3].split_fft();
        let g0 = [d00.clone(), d01.clone(), d01.map(|c| c.conj()), d00];
        let g1 = [d10.clone(), d11.clone(), d11.map(|c| c.conj()), d10];
        LdlTree::Branch(l[2].clone(), Box::new(ffldl(g0)), Box::new(ffldl(g1)))
    } else {
        LdlTree::Branch(
            l[2].clone(),
            Box::new(LdlTree::Leaf(d[0].clone().coefficients.try_into().unwrap())),
            Box::new(LdlTree::Leaf(d[3].clone().coefficients.try_into().unwrap())),
        )
    }
}

pub fn normalize_tree(tree: &mut LdlTree, sigma: f64) {
    match tree {
        LdlTree::Branch(_ell, left, right) => {
            normalize_tree(left, sigma);
            normalize_tree(right, sigma);
        }
        LdlTree::Leaf(vector) => {
            vector[0] = Complex::new(sigma / vector[0].re.sqrt(), 0.0);
            vector[1] = Complex64::zero();
        }
    }
}

pub fn ffsampling(
    t: &(Polynomial<Complex64>, Polynomial<Complex64>),
    tree: &LdlTree,
    parameters: &base_falcon::FalconParameters,
    rng: &mut dyn RngCore,
) -> (Polynomial<Complex64>, Polynomial<Complex64>) {
    match tree {
        LdlTree::Branch(ell, left, right) => {
            let bold_t1 = t.1.split_fft();
            let bold_z1 = ffsampling(&bold_t1, right, parameters, rng);
            let z1 = Polynomial::<Complex64>::merge_fft(&bold_z1.0, &bold_z1.1);

            // t0' = t0  + (t1 - z1) * l
            let t0_prime = t.0.clone() + (t.1.clone() - z1.clone()).hadamard_mul(ell);

            let bold_t0 = t0_prime.split_fft();
            let bold_z0 = ffsampling(&bold_t0, left, parameters, rng);
            let z0 = Polynomial::<Complex64>::merge_fft(&bold_z0.0, &bold_z0.1);

            (z0, z1)
        }
        LdlTree::Leaf(value) => {
            let z0 = sampler_z(t.0.coefficients[0].re, value[0].re, parameters.sigmin, rng);
            let z1 = sampler_z(t.1.coefficients[0].re, value[0].re, parameters.sigmin, rng);
            (
                Polynomial::new(vec![Complex64::new(z0 as f64, 0.0)]),
                Polynomial::new(vec![Complex64::new(z1 as f64, 0.0)]),
            )
        }
    }
}
