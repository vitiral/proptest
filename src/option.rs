//-
// Copyright 2017 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Strategies for generating `std::Option` values.

#![cfg_attr(feature="cargo-clippy",
    allow(type_complexity, expl_impl_clone_on_copy))]

use std::fmt;
use std::marker::PhantomData;

use strategy::*;
use test_runner::*;

mapfn! {
    [] fn WrapSome[<T : fmt::Debug>](t: T) -> Option<T> {
        Some(t)
    }
}

struct NoneStrategy<T>(PhantomData<T>);
impl<T> Clone for NoneStrategy<T> {
    fn clone(&self) -> Self { *self }
}
impl<T> Copy for NoneStrategy<T> { }
impl<T> fmt::Debug for NoneStrategy<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoneStrategy")
    }
}
impl<T : fmt::Debug> Strategy for NoneStrategy<T> {
    type Value = Self;

    fn new_value(&self, _: &mut TestRunner) -> NewTree<Self> {
        Ok(*self)
    }
}
impl<T : fmt::Debug> ValueTree for NoneStrategy<T> {
    type Value = Option<T>;

    fn current(&self) -> Option<T> { None }
    fn simplify(&mut self) -> bool { false }
    fn complicate(&mut self) -> bool { false }
}

opaque_strategy_wrapper! {
    /// Strategy which generates `Option` values whose inner `Some` values are
    /// generated by another strategy.
    ///
    /// Constructed by other functions in this module.
    #[derive(Clone)]
    pub struct OptionStrategy[<T>][where T : Strategy]
        (TupleUnion<(W<NoneStrategy<ValueFor<T>>>,
                     W<statics::Map<T, WrapSome>>)>)
        -> OptionValueTree<T::Value>;
    /// `ValueTree` type corresponding to `OptionStrategy`.
    #[derive(Clone, Debug)]
    pub struct OptionValueTree[<T>][where T : ValueTree]
        (TupleUnionValueTree<(NoneStrategy<T::Value>,
                              Option<statics::Map<T, WrapSome>>)>)
        -> Option<T::Value>;
}

// XXX Unclear why this is necessary; #[derive(Debug)] *should* generate
// exactly this, but for some reason it adds a `T::Value : Debug` constraint as
// well.
impl<T : Strategy + fmt::Debug> fmt::Debug for OptionStrategy<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OptionStrategy({:?})", self.0)
    }
}

/// Return a strategy producing `Optional` values wrapping values from the
/// given delegate strategy.
///
/// `Some` values shrink to `None`.
///
/// `Some` and `None` are each chosen with 50% probability.
pub fn of<T : Strategy>(t: T) -> OptionStrategy<T> {
    weighted(0.5, t)
}

/// Return a strategy producing `Optional` values wrapping values from the
/// given delegate strategy.
///
/// `Some` values shrink to `None`.
///
/// `Some` is chosen with a probability given by `probability_of_some`, which
/// must be between 0.0 and 1.0, both exclusive.
pub fn weighted<T : Strategy>(probability_of_some: f64, t: T)
                              -> OptionStrategy<T> {
    let (weight_some, weight_none) = float_to_weight(probability_of_some);

    OptionStrategy(TupleUnion::new((
        (weight_none, NoneStrategy(PhantomData)),
        (weight_some, statics::Map::new(t, WrapSome)),
    )))
}

#[cfg(test)]
mod test {
    use super::*;

    fn count_some_of_1000(s: OptionStrategy<Just<i32>>) -> u32 {
        let mut runner = TestRunner::default();
        let mut count = 0;
        for _ in 0..1000 {
            count += s.new_value(&mut runner).unwrap()
                .current().is_some() as u32;
        }

        count
    }

    #[test]
    fn probability_defaults_to_0p5() {
        let count = count_some_of_1000(of(Just(42i32)));
        assert!(count > 450 && count < 550);
    }

    #[test]
    fn probability_handled_correctly() {
        let count = count_some_of_1000(weighted(0.9, Just(42i32)));
        assert!(count > 800 && count < 950);

        let count = count_some_of_1000(weighted(0.1, Just(42i32)));
        assert!(count > 50 && count < 150);
    }

    #[test]
    fn test_sanity() {
        check_strategy_sanity(of(0i32..1000i32), None);
    }
}
