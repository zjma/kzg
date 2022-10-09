use blstrs::Scalar;
use kzg::polynomial::Polynomial;
use kzg::{coeff_form::KZGProver, setup, KZGParams};
use pairing::group::ff::Field;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn csprng_setup<const MAX_COEFFS: usize>() -> KZGParams {
    let s: Scalar = rand::random::<u64>().into();
    setup(s, MAX_COEFFS)
}

fn bench_create_witness<const NUM_COEFFS: usize>(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([42; 32]);
    let params = csprng_setup::<NUM_COEFFS>();

    c.bench_function(
        format!("bench_create_witness_coeff_form, degree {}", NUM_COEFFS - 1).as_str(),
        |b| {
            let mut coeffs = vec![Scalar::zero(); NUM_COEFFS];
            for i in 0..NUM_COEFFS {
                coeffs[i] = rng.gen::<u64>().into();
            }
            let polynomial = Polynomial::new_from_coeffs(coeffs, NUM_COEFFS - 1);
            let prover = KZGProver::new(&params);
            let _commitment = prover.commit(&polynomial);

            let x: Scalar = Scalar::random(&mut rng);
            let y = polynomial.eval(x);
            b.iter(|| black_box(&prover).create_witness(black_box(&polynomial), black_box((x, y))).unwrap())
        },
    );
}

mod perf;

criterion_group!(
    name = create_witness;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = bench_create_witness<1024>
);
criterion_main!(create_witness);
