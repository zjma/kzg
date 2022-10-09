use blstrs::Scalar;
use kzg::{eval_form::{compute_lagrange_basis, KZGProverEvalForm}, ft::EvaluationDomain, setup, KZGParams};
use pairing::group::ff::Field;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn test_setup(rng: &mut SmallRng, d: usize) -> KZGParams {
    let s: Scalar = rng.gen::<u64>().into();
    setup(s, d)
}

fn random_evals(rng: &mut SmallRng, d: usize) -> EvaluationDomain {
    let mut coeffs = vec![Scalar::zero(); d];

    for i in 0..d {
        coeffs[i] = rng.gen::<u64>().into();
    }

    EvaluationDomain::from_coeffs(coeffs).unwrap()
}

fn bench_commit<const NUM_COEFFS: usize>(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([42; 32]);
    let params = test_setup(&mut rng, NUM_COEFFS);
    let lagrange_basis = compute_lagrange_basis(&params);


    c.bench_function(
        format!("bench_commit_eval_form, degree {}", NUM_COEFFS - 1).as_str(),
        |b| {
            let evals = random_evals(&mut rng, NUM_COEFFS);
            let prover = KZGProverEvalForm::new(&params, lagrange_basis.0.as_slice());
            b.iter(|| {
                prover.commit(black_box(&evals))
            })
        },
    );
}

mod perf;

criterion_group!(
    name = commit;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = bench_commit<1024>
);
criterion_main!(commit);
