use std::os;

/// Parallelizes `operation` over given `data`.
///
/// The data are divided into chunks based on the number of processing cores
/// available to the rust process according to `std::os::num_cpus`.
///
/// # Panics
///
/// Panics if any of the underlying threads panics
///
/// # Example
///
/// ```
/// extern crate parallel;
///
/// # fn main() {
/// let mut v = (1..10).collect::<Vec<usize>>();
/// # let w = v.iter().map(|x| x + 1).collect::<Vec<usize>>();
/// parallel::apply(v.as_mut_slice(), |x| {
///     *x = *x + 1;
///     //..
/// });
/// # assert_eq!(v, w);
/// # }
/// ```
pub fn apply<T, F>(data: &mut [T], operation: F) where
    T: Send,
    F: Fn(&mut T) + Sync,
{
    let granularity = data.len() / os::num_cpus() + 1;

    ::divide(data, granularity, |data, _|{
        for e in data.iter_mut() {
            operation(e);
        }
    })
}
