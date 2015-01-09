use std::thread::Thread;
use std::{cmp, iter, mem, raw};

// Proxy struct to send raw pointers across task boundaries
struct RawPtr<T>(*const T);

impl<T> Copy for RawPtr<T> {}

unsafe impl<T> Send for RawPtr<T> where T: Send {}

/// Parallelizes an `operation` over a mutable slice
///
/// The `data` will be divided in chunks of `granularity` size. A new thread will be spawned to
/// "operate" over each chunk.
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
/// use std::num::Float;
/// use std::rand::{Rng, XorShiftRng, self};
///
/// let ref mut rng: XorShiftRng = rand::thread_rng().gen();
/// let mut v = range(0, 1_000u).map(|_| rng.gen::<f32>()).collect::<Vec<_>>();
/// # let w = v.iter().map(|x| x.sin()).collect::<Vec<_>>();
/// parallel::divide(v.as_mut_slice(), 100, |data, _| {
///     for x in data.iter_mut() {
///         *x = x.sin();
///     }
/// });
/// # assert_eq!(v, w);
/// ```
pub fn divide<T, F>(data: &mut [T], granularity: usize, operation: F) where
    T: Send,
    F: Fn(&mut [T], usize) + Sync,
{
    assert!(granularity > 0);

    let raw::Slice { data, len } = unsafe { mem::transmute::<_, raw::Slice<T>>(data) };
    let data = RawPtr(data);
    let op = RawPtr(&operation as *const _ as *const ());

    let threads = iter::range_step(0, len, granularity).map(|offset| {
        Thread::scoped(move || {
            // NB Is safe to send the slice/closure because the thread won't outlive this function
            let slice = raw::Slice {
                data: unsafe { data.0.offset(offset as isize) },
                len: cmp::min(granularity, len - offset)
            };
            let data = unsafe { mem::transmute::<_, &mut [T]>(slice) };
            let operation = unsafe { mem::transmute::<_, &F>(op.0) };

            (*operation)(data, offset);
        })
    }).collect::<Vec<_>>();

    for thread in threads.into_iter() {
        if thread.join().is_err() {
            panic!();
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::iter;
    use std::rand::{Rng, XorShiftRng, self};

    #[quickcheck]
    fn clone(size: usize, granularity: usize) -> TestResult {
        if granularity == 0 {
            return TestResult::discard();
        }

        let mut rng: XorShiftRng = rand::thread_rng().gen();
        let original = range(0, size).map(|_| rng.gen::<f64>()).collect::<Vec<_>>();
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
