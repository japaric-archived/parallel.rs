use std::{cmp, iter, mem, raw, task};

/// Parallelizes an `operation` over a mutable slice
///
/// The `data` will be divided in chunks of `granularity` size. A new task will be spawned to
/// "operate" over each chunk.
///
/// `operation` will receive two arguments:
///
/// - The mutable chunk of data, and
/// - The offset of this chunk from the start of `data`
///
/// # Panics
///
/// Panics if any of the underlying tasks panics
///
/// # Example
///
/// Parallel map
///
/// ```
/// use std::num::FloatMath;
/// use std::rand::{Rng, XorShiftRng, mod};
///
/// let ref mut rng: XorShiftRng = rand::task_rng().gen();
/// let mut v = Vec::from_fn(1_000, |_| rng.gen::<f32>());
/// # let w = v.iter().map(|x| x.sin()).collect::<Vec<_>>();
/// parallel::divide(v.as_mut_slice(), 100, |data, _| {
///     for x in data.iter_mut() {
///         *x = x.sin();
///     }
/// });
/// # assert_eq!(v, w);
/// ```
pub fn divide<T, F: Fn(&mut [T], uint) + Sync>(
    data: &mut [T],
    granularity: uint,
    operation: F,
) where
    T: Send,
{
    assert!(granularity > 0);

    let raw::Slice { data, len } = unsafe { mem::transmute::<_, raw::Slice<T>>(data) };
    let op = &operation as *const _ as *const ();

    let futures = iter::range_step(0, len, granularity).map(|offset| {
        task::try_future(move || {
            // NB Is safe to send the slice/closure because the task won't outlive this function
            let slice = raw::Slice {
                data: unsafe { data.offset(offset as int) },
                len: cmp::min(granularity, len - offset)
            };
            let data = unsafe { mem::transmute::<_, &mut [T]>(slice) };
            let operation = unsafe { mem::transmute::<_, &F>(op) };

            (*operation)(data, offset);
        })
    }).collect::<Vec<_>>();

    for future in futures.into_iter() {
        if future.into_inner().is_err() {
            panic!();
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::rand::{Rng, XorShiftRng, mod};

    #[quickcheck]
    fn clone(size: uint, granularity: uint) -> TestResult {
        if granularity == 0 {
            return TestResult::discard();
        }

        let mut rng: XorShiftRng = rand::task_rng().gen();
        let original = Vec::from_fn(size, |_| rng.gen::<f64>());
        let mut clone = Vec::from_elem(size, 0f64);

        let original_slice = original[];
        super::divide(clone[mut], granularity, |data, offset| {
            for (i, x) in data.iter_mut().enumerate() {
                *x = original_slice[offset + i]
            }
        });

        TestResult::from_bool(original == clone)
    }

    #[quickcheck]
    fn new(size: uint, granularity: uint) -> TestResult {
        if granularity == 0 {
            return TestResult::discard();
        }

        let mut v = Vec::from_elem(size, None::<f64>);

        super::divide(v[mut], granularity, |data, _| {
            let mut rng: XorShiftRng = rand::task_rng().gen();

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
