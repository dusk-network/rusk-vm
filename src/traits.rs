// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Primitives for the runtime modules.

pub use integer_sqrt::IntegerSquareRoot;
pub use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedShl, CheckedShr,
    CheckedSub, One, Zero,
};

use std::convert::{TryFrom, TryInto};

/// Just like `From` except that if the source value is too big to fit into the destination type
/// then it'll saturate the destination.
pub trait UniqueSaturatedFrom<T: Sized>: Sized {
    /// Convert from a value of `T` into an equivalent instance of `Self`.
    fn unique_saturated_from(t: T) -> Self;
}

/// Just like `Into` except that if the source value is too big to fit into the destination type
/// then it'll saturate the destination.
pub trait UniqueSaturatedInto<T: Sized>: Sized {
    /// Consume self to return an equivalent value of `T`.
    fn unique_saturated_into(self) -> T;
}

impl<T: Sized, S: TryFrom<T> + Bounded + Sized> UniqueSaturatedFrom<T> for S {
    fn unique_saturated_from(t: T) -> Self {
        S::try_from(t).unwrap_or_else(|_| Bounded::max_value())
    }
}

impl<T: Bounded + Sized, S: TryInto<T> + Sized> UniqueSaturatedInto<T> for S {
    fn unique_saturated_into(self) -> T {
        self.try_into().unwrap_or_else(|_| Bounded::max_value())
    }
}

/// Simple trait to use checked mul and max value to give a saturated mul operation over
/// supported types.
pub trait Saturating {
    /// Saturated addition - if the product can't fit in the type then just use max-value.
    fn saturating_add(self, o: Self) -> Self;

    /// Saturated subtraction - if the product can't fit in the type then just use max-value.
    fn saturating_sub(self, o: Self) -> Self;

    /// Saturated multiply - if the product can't fit in the type then just use max-value.
    fn saturating_mul(self, o: Self) -> Self;
}

impl<T: CheckedMul + Bounded + num_traits::Saturating> Saturating for T {
    fn saturating_add(self, o: Self) -> Self {
        <Self as num_traits::Saturating>::saturating_add(self, o)
    }
    fn saturating_sub(self, o: Self) -> Self {
        <Self as num_traits::Saturating>::saturating_sub(self, o)
    }
    fn saturating_mul(self, o: Self) -> Self {
        self.checked_mul(&o).unwrap_or_else(Bounded::max_value)
    }
}

/// Convenience type to work around the highly unergonomic syntax needed
/// to invoke the functions of overloaded generic traits, in this case
/// `SaturatedFrom` and `SaturatedInto`.
pub trait SaturatedConversion {
    /// Convert from a value of `T` into an equivalent instance of `Self`.
    ///
    /// This just uses `UniqueSaturatedFrom` internally but with this
    /// variant you can provide the destination type using turbofish syntax
    /// in case Rust happens not to assume the correct type.
    fn saturated_from<T>(t: T) -> Self
    where
        Self: UniqueSaturatedFrom<T>,
    {
        <Self as UniqueSaturatedFrom<T>>::unique_saturated_from(t)
    }

    /// Consume self to return an equivalent value of `T`.
    ///
    /// This just uses `UniqueSaturatedInto` internally but with this
    /// variant you can provide the destination type using turbofish syntax
    /// in case Rust happens not to assume the correct type.
    fn saturated_into<T>(self) -> T
    where
        Self: UniqueSaturatedInto<T>,
    {
        <Self as UniqueSaturatedInto<T>>::unique_saturated_into(self)
    }
}
impl<T: Sized> SaturatedConversion for T {}
