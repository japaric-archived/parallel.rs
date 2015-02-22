use std::thread;

/// Parallelizes an `operation` over a mutable slice
///
/// The `data` will be divided in chunks of `granularity` size.
///  A new thread will be spawned to "operate" over each chunk.
///
/// `operation` will receive two arguments:
///
/// - The mutable chunk of data, and
/// - The offset of this chunk from the start of `data`
///
/// # Panics
///
/// Panics if any of the underlying threads panics
///
/// # Example
///
/// Parallel map
///
/// ```
/// extern crate parallel;
/// extern crate rand;
///
/// use rand::{Rng, XorShiftRng};
/// use std::num::Float;
///
/// # fn main() {
/// let ref mut rng: XorShiftRng = rand::thread_rng().gen();
/// let mut v = (0..1_000).map(|_| rng.gen::<f32>()).collect::<Vec<_>>();
/// # let w = v.iter().map(|x| x.sin()).collect::<Vec<_>>();
/// parallel::divide(v.as_mut_slice(), 100, |data, _| {
///     for x in data.iter_mut() {
///         *x = x.sin();
///     }
/// });
/// # assert_eq!(v, w);
/// # }
/// ```
pub fn divide<T, F>(data: &mut [T], granularity: usize, operation: F) where
    T: Send,
    F: Fn(&mut [T], usize) + Sync,
{
    assert!(granularity > 0);

    let operation = &operation;
    let guards: Vec<_> = data.chunks_mut(granularity).zip(0..).map(|(chunk, i)| {
        thread::scoped(move || {
            (*operation)(chunk, i * granularity)
        })
    }).collect();

    for guard in guards {
        guard.join();
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use rand::{Rng, XorShiftRng, self};
    use std::iter;

    #[quickcheck]
    fn clone(size: usize, granularity: usize) -> TestResult {
        if granularity == 0 {
            return TestResult::discard();
        }

        let mut rng: XorShiftRng = rand::thread_rng().gen();
        let original = (0..size).map(|_| rng.gen::<f64>()).collect::<Vec<_>>();
        let mut clone = iter::repeat(0f64).take(size).collect::<Vec<_>>();

        let original_slice = &*original;
        super::divide(&mut *clone, granularity, |data, offset| {
            for (i, x) in data.iter_mut().enumerate() {
                *x = original_slice[offset + i]
            }
        });

        TestResult::from_bool(original == clone)
    }

    #[quickcheck]
    fn new(size: usize, granularity: usize) -> TestResult {
        if granularity == 0 {
            return TestResult::discard();
        }

        let mut v = iter::repeat(None::<f64>).take(size).collect::<Vec<_>>();

        super::divide(&mut *v, granularity, |data, _| {
            let mut rng: XorShiftRng = rand::thread_rng().gen();

            for x in data.iter_mut() {
                *x = Some(rng.gen())
            }
        });

        TestResult::from_bool(v.iter().all(|&x| match x {
            None => false,
            Some(x) => x > 0. && x < 1.
        }))
    }
}
