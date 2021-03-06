/*
 * Copyright 2014 Ben Ashford
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![crate_type = "lib"]
#![crate_name = "lazysort"]
#![feature(test)]

extern crate test;
extern crate rand;

use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Greater};

fn pivot(lower: usize, upper: usize) -> usize {
    return lower + ((upper - lower) / 2);
}

pub struct LazySortIterator<T: Clone, F> {
    data: Vec<T>,
    work: Vec<(usize, usize)>,
    by: F,
}

impl<T, F> LazySortIterator<T, F> where
    T: Clone,
    F: FnMut(&T, &T) -> Ordering,
{
    fn new(data: Vec<T>, by: F) -> Self where
        F: FnMut(&T, &T) -> Ordering
    {
        let l = data.len();
        LazySortIterator {
            data: data,
            work: if l == 0 {
                vec![]
            } else {
                vec![(0, l - 1)]
            },
            by: by
        }
    }

    fn partition(&mut self, lower: usize, upper: usize, p: usize) -> usize {
        assert!(lower <= upper);
        assert!(p >= lower);
        assert!(p <= upper);

        let length = upper - lower;
        if length == 0 {
            p
        } else {
            let lasti = upper;
            let (mut i, mut nextp) = (lower, lower);
            self.data.swap(lasti, p);
            while i < lasti {
                if (self.by)(&self.data[i], &self.data[lasti]) == Less {
                    self.data.swap(i, nextp);
                    nextp = nextp + 1;
                }
                i = i + 1;
            }
            self.data.swap(nextp, lasti);
            nextp
        }
    }

    fn qsort(&mut self, lower: usize, upper: usize) -> T {
        assert!(lower <= upper);

        if lower == upper {
            return self.data[lower].clone();
        }

        let p = pivot(lower, upper);
        let p = self.partition(lower, upper, p);

        if p < upper {
            self.work.push((p + 1, upper));
        }
        self.qsort(lower, p)
    }
}

pub trait Sorted {
    type Item: Clone + Ord;

    fn sorted(self) ->
        LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedPartial {
    type Item: Clone + PartialOrd;

    fn sorted_partial(self, first: bool) ->
        LazySortIterator<Self::Item, fn(&Self::Item, &Self::Item) -> Ordering>;
}

pub trait SortedBy {
    type Item: Clone;

    fn sorted_by<F>(self, F) -> LazySortIterator<Self::Item, F>
        where F: Fn(&Self::Item, &Self::Item) -> Ordering;
}

impl<T, I> Sorted for I where
    T: Clone + Eq + Ord,
    I: Iterator<Item=T>
{
    type Item = T;

    fn sorted(self) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        LazySortIterator::new(self.collect(), Ord::cmp)
    }
}

/*
fn partial_cmp_to_cmp<T>(first: bool, a: &T, b: &T) -> Ordering where
    T: PartialOrd
{
    match a.partial_cmp(b) {
        Some(order) => order,
        None => if first {
            Less
        } else {
            Greater
        }
    }
}

impl<T, I> SortedPartial for I where
    T: Clone + PartialOrd,
    I: Iterator<Item=T>,
{
    type Item = T;

    fn sorted_partial(self, first: bool) -> LazySortIterator<T, fn(&Self::Item, &Self::Item) -> Ordering> {
        LazySortIterator::new(self.collect(), move |a: &T, b: &T| {
            match a.partial_cmp(b) {
                Some(order) => order,
                None => if first {
                    Less
                } else {
                    Greater
                }
            }

        })
    }
}
*/

impl<T, I> SortedBy for I where
    T: Clone,
    I: Iterator<Item=T>,
{
    type Item = T;

    fn sorted_by<F>(self, by: F) -> LazySortIterator<T, F> where
        F: Fn(&T, &T) -> Ordering
    {
        LazySortIterator::new(self.collect(), by)
    }
}

impl<T, F> Iterator for LazySortIterator<T, F> where
    T: Clone,
    F: FnMut(&T, &T) -> Ordering,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.work.pop() {
            Some(next_work) => {
                let (lower, upper) = next_work;
                Some(self.qsort(lower, upper))
            },
            None => None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.data.len();
        (l, Some(l))
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use test::Bencher;

    use rand;
    use rand::distributions::{IndependentSample, Range};

    use super::Sorted;
    use super::SortedPartial;
    use super::SortedBy;

    #[test]
    fn sorted_test() {
        let expected: Vec<u64> = vec![1u64, 1, 1, 3, 4, 6, 7, 9, 22];
        let before: Vec<u64> = vec![9u64, 7, 1, 1, 6, 3, 1, 4, 22];
        let after: Vec<u64> = before.iter().sorted().map(|x| *x).collect();

        assert_eq!(expected, after);
    }

    #[test]
    fn empty_test() {
        let before: Vec<u64> = vec![];
        let after: Vec<u64> = before.iter().sorted().map(|x| *x).collect();
        assert_eq!(before, after);
    }

    /*
    #[test]
    fn sorted_partial_test() {
        let expected: Vec<f64> = vec![0.9_f64, 1.0, 1.0, 1.1, 75.3, 75.3];
        let before: Vec<f64> = vec![1.0_f64, 1.1, 0.9, 75.3, 1.0, 75.3];
        let after: Vec<f64> = before.iter().sorted_partial(true).map(|x| *x).collect();

        assert_eq!(expected, after);
    }
    */

    #[test]
    fn sorted_by_test() {
        let expected: Vec<u64> = vec![4, 1, 3, 2];
        let before: Vec<(f64, u64)> = vec![(0.2, 1),
                                           (0.9, 2),
                                           (0.4, 3),
                                           (0.1, 4)];

        let after: Vec<u64> = before.iter()
            .sorted_by(|&a, &b| {
                let (ax, _) = *a;
                let (bx, _) = *b;
                ax.partial_cmp(&bx).unwrap()
            })
            .map(|&(_, y)| y)
            .collect();

        assert_eq!(expected, after);
    }

    // BENCHMARKS

    static RANGE: u64 = 1000000;
    static VEC_SIZE: u64 = 50000;
    static PICK_SIZE_A: usize = 1000;
    static PICK_SIZE_B: usize = 10000;
    static PICK_SIZE_C: usize = *&VEC_SIZE as usize;

    #[bench]
    fn a_standard_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&u64> = numbers.iter().take(PICK_SIZE_A).collect();
        });
    }

    #[bench]
    fn a_lazy_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&u64> = numbers.iter().sorted().take(PICK_SIZE_A).collect();
        });
    }
    #[bench]
    fn b_standard_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&u64> = numbers.iter().take(PICK_SIZE_B).collect();
        });
    }

    #[bench]
    fn b_lazy_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&u64> = numbers.iter().sorted().take(PICK_SIZE_B).collect();
        });
    }
    #[bench]
    fn c_standard_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let mut numbers = numbers_raw.clone();
            numbers.sort();
            let _: Vec<&u64> = numbers.iter().take(PICK_SIZE_C).collect();
        });
    }

    #[bench]
    fn c_lazy_bench(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0u64, RANGE);
        let numbers_raw: Vec<u64> = (0u64..VEC_SIZE).map(|_| between.ind_sample(&mut rng)).collect();

        b.iter(|| {
            let numbers = numbers_raw.clone();

            let _: Vec<&u64> = numbers.iter().sorted().take(PICK_SIZE_C).collect();
        });
    }
}
