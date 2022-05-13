use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use snafu::Snafu;
use std::{
    cmp::Ordering,
    convert::TryInto,
    fmt::{self, Write},
    ops,
    str::FromStr,
};

use crate::errors::StdError;

use super::{Fraction, Isqrt, Uint128, Uint256};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal(#[schemars(with = "String")] Uint128);

#[derive(Snafu, Debug, PartialEq)]
#[snafu(display("Decimal range exceeded"))]
pub struct DecimalRangeExceeded;

impl Decimal {
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);
    // 1*10**18
    const DECIMAL_FRACTIONAL_SQUARED: Uint128 =
        Uint128::new(1_000_000_000_000_000_000_000_000_000_000_000_000u128);
    // (1*10**18)**2 = 1*10**36
    const DECIMAL_PLACES: usize = 18;
    // This needs to be an even number.

    pub const MAX: Self = Self(Uint128::MAX);

    /// Create a 1.0 Decimal
    pub const fn one() -> Self {
        Decimal(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    pub const fn zero() -> Self {
        Decimal(Uint128::zero())
    }

    /// Convert x% into Decimal
    pub fn percent(x: u64) -> Self {
        Decimal(((x as u128) * 10_000_000_000_000_000).into())
    }

    /// Convert permille (x/1000) into Decimal
    pub fn permille(x: u64) -> Self {
        Decimal(((x as u128) * 1_000_000_000_000_000).into())
    }

    /// Creates a decimal from a number of atomic units and the number
    /// of decimal places. The inputs will be converted internally to form
    /// a decimal with 18 decimal places. So the input 123 and 2 will create
    /// the decimal 1.23.
    ///
    /// Using 18 decimal places is slightly more efficient than other values
    /// as no internal conversion is necessary.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_math::{Decimal, Uint128};
    /// let a = Decimal::from_atomics(Uint128::new(1234), 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = Decimal::from_atomics(1234u128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = Decimal::from_atomics(1u64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Uint128>,
        decimal_places: u32,
    ) -> Result<Self, DecimalRangeExceeded> {
        let atomics = atomics.into();
        const TEN: Uint128 = Uint128::new(10);
        Ok(match decimal_places.cmp(&(Self::DECIMAL_PLACES as u32)) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES as u32) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = TEN.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| DecimalRangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES as u32); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = TEN.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Uint128 range.
                    // Any Uint128 `x` divided by `factor` with `factor > Uint128::MAX` is 0.
                    // Try e.g. Python3: `(2**128-1) // 2**128`
                    Self(Uint128::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn from_ratio(numerator: impl Into<Uint128>, denominator: impl Into<Uint128>) -> Self {
        let numerator: Uint128 = numerator.into();
        let denominator: Uint128 = denominator.into();
        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }

        Decimal(
            // numerator * DECIMAL_FRACTIONAL / denominator
            numerator.multiply_ratio(Self::DECIMAL_FRACTIONAL, denominator),
        )
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// A decimal is an integer of atomic units plus a number that specifies the
    /// position of the decimal dot. So any decimal can be expressed as two numbers.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_math::{Decimal, Uint128};
    /// # use std::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = Decimal::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Uint128::new(1234000000000000000));
    ///
    /// // Smallest possible value
    /// let b = Decimal::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Uint128::new(1));
    /// ```
    pub fn atomics(&self) -> Uint128 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`Decimal::atomics()`].
    pub fn decimal_places(&self) -> u32 {
        Self::DECIMAL_PLACES as u32
    }

    /// Returns the approximate square root as a Decimal.
    ///
    /// This should not overflow or panic.
    pub fn sqrt(&self) -> Self {
        // Algorithm described in https://hackmd.io/@webmaster128/SJThlukj_
        // We start with the highest precision possible and lower it until
        // there's no overflow.
        //
        // TODO: This could be made more efficient once log10 is in:
        // https://github.com/rust-lang/rust/issues/70887
        // The max precision is something like `9 - log10(self.0) / 2`.
        (0..=Self::DECIMAL_PLACES / 2)
            .rev()
            .find_map(|i| self.sqrt_with_precision(i))
            // The last step (i = 0) is guaranteed to succeed because `isqrt(u128::MAX) * 10^9` does not overflow
            .unwrap()
    }

    /// Lower precision means more aggressive rounding, but less risk of overflow.
    /// Precision *must* be a number between 0 and 9 (inclusive).
    ///
    /// Returns `None` if the internal multiplication overflows.
    fn sqrt_with_precision(&self, precision: usize) -> Option<Self> {
        let precision = precision as u32;

        let inner_mul = 100u128.pow(precision);
        self.0.checked_mul(inner_mul.into()).ok().map(|inner| {
            let outer_mul = 10u128.pow(Self::DECIMAL_PLACES as u32 / 2 - precision);
            Decimal(inner.isqrt().checked_mul(Uint128::from(outer_mul)).unwrap())
        })
    }
}

impl Fraction<Uint128> for Decimal {
    #[inline]
    fn numerator(&self) -> Uint128 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Uint128 {
        Self::DECIMAL_FRACTIONAL
    }

    /// Returns the multiplicative inverse `1/d` for decimal `d`.
    ///
    /// If `d` is zero, none is returned.
    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // Let self be p/q with p = self.0 and q = DECIMAL_FRACTIONAL.
            // Now we calculate the inverse a/b = q/p such that b = DECIMAL_FRACTIONAL. Then
            // `a = DECIMAL_FRACTIONAL*DECIMAL_FRACTIONAL / self.0`.
            Some(Decimal(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl FromStr for Decimal {
    type Err = StdError;

    /// Converts the decimal string to a Decimal
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than DECIMAL_PLACES fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part
            .parse::<Uint128>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(Self::DECIMAL_FRACTIONAL)
            .map_err(|_| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<Uint128>()
                .map_err(|_| StdError::generic_err("Error parsing fractional"))?;
            let exp =
                (Self::DECIMAL_PLACES.checked_sub(fractional_part.len())).ok_or_else(|| {
                    StdError::generic_err(format!(
                        "Cannot parse more than {} fractional digits",
                        Self::DECIMAL_PLACES
                    ))
                })?;
            debug_assert!(exp <= Self::DECIMAL_PLACES);
            let fractional_factor = Uint128::from(10u128.pow(exp as u32));
            atomics = atomics
                .checked_add(
                    // The inner multiplication can't overflow because
                    // fractional < 10^DECIMAL_PLACES && fractional_factor <= 10^DECIMAL_PLACES
                    fractional.checked_mul(fractional_factor).unwrap(),
                )
                .map_err(|_| StdError::generic_err("Value too big"))?;
        }

        if parts_iter.next().is_some() {
            return Err(StdError::generic_err("Unexpected number of dots"));
        }

        Ok(Decimal(atomics))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Self::DECIMAL_FRACTIONAL;
        let fractional = (self.0).checked_rem(Self::DECIMAL_FRACTIONAL).unwrap();

        if fractional.is_zero() {
            write!(f, "{}", whole)
        } else {
            let fractional_string =
                format!("{:0>padding$}", fractional, padding = Self::DECIMAL_PLACES);
            f.write_str(&whole.to_string())?;
            f.write_char('.')?;
            f.write_str(fractional_string.trim_end_matches('0'))?;
            Ok(())
        }
    }
}

impl ops::Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
    }
}

impl ops::Add<&Decimal> for Decimal {
    type Output = Self;

    fn add(self, other: &Decimal) -> Self {
        Decimal(self.0 + other.0)
    }
}

impl ops::Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal(self.0 - other.0)
    }
}

impl ops::Mul for Decimal {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // Decimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_uint256 = self.numerator().full_mul(other.numerator())
            / Uint256::from_uint128(Self::DECIMAL_FRACTIONAL); // from_uint128 is a const method and should be "free"
        match result_as_uint256.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}

/// Both d*u and u*d with d: Decimal and u: Uint128 returns an Uint128. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal result for the same calculation, use Decimal(d*u) or Decimal(u*d).
impl ops::Mul<Decimal> for Uint128 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint128::zero();
        }
        self.multiply_ratio(rhs.0, Decimal::DECIMAL_FRACTIONAL)
    }
}

impl ops::Mul<Uint128> for Decimal {
    type Output = Uint128;

    fn mul(self, rhs: Uint128) -> Self::Output {
        rhs * self
    }
}

impl ops::Div<Uint128> for Decimal {
    type Output = Self;

    fn div(self, rhs: Uint128) -> Self::Output {
        Decimal(self.0 / rhs)
    }
}

impl ops::DivAssign<Uint128> for Decimal {
    fn div_assign(&mut self, rhs: Uint128) {
        self.0 /= rhs;
    }
}

impl<A> std::iter::Sum<A> for Decimal
where
    Self: ops::Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), ops::Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl<'de> de::Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Decimal::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_slice, to_vec};

    #[test]
    fn decimal_one() {
        let value = Decimal::one();
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal::percent(50);
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(2u8));
    }

    #[test]
    fn decimal_permille() {
        let value = Decimal::permille(125);
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(8u8));
    }

    #[test]
    fn decimal_from_atomics_works() {
        let one = Decimal::one();
        let two = one + one;

        assert_eq!(Decimal::from_atomics(1u128, 0).unwrap(), one);
        assert_eq!(Decimal::from_atomics(10u128, 1).unwrap(), one);
        assert_eq!(Decimal::from_atomics(100u128, 2).unwrap(), one);
        assert_eq!(Decimal::from_atomics(1000u128, 3).unwrap(), one);
        assert_eq!(
            Decimal::from_atomics(1000000000000000000u128, 18).unwrap(),
            one
        );
        assert_eq!(
            Decimal::from_atomics(10000000000000000000u128, 19).unwrap(),
            one
        );
        assert_eq!(
            Decimal::from_atomics(100000000000000000000u128, 20).unwrap(),
            one
        );

        assert_eq!(Decimal::from_atomics(2u128, 0).unwrap(), two);
        assert_eq!(Decimal::from_atomics(20u128, 1).unwrap(), two);
        assert_eq!(Decimal::from_atomics(200u128, 2).unwrap(), two);
        assert_eq!(Decimal::from_atomics(2000u128, 3).unwrap(), two);
        assert_eq!(
            Decimal::from_atomics(2000000000000000000u128, 18).unwrap(),
            two
        );
        assert_eq!(
            Decimal::from_atomics(20000000000000000000u128, 19).unwrap(),
            two
        );
        assert_eq!(
            Decimal::from_atomics(200000000000000000000u128, 20).unwrap(),
            two
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            Decimal::from_atomics(4321u128, 20).unwrap(),
            Decimal::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(6789u128, 20).unwrap(),
            Decimal::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 38).unwrap(),
            Decimal::from_str("3.402823669209384634").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 39).unwrap(),
            Decimal::from_str("0.340282366920938463").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 45).unwrap(),
            Decimal::from_str("0.000000340282366920").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 51).unwrap(),
            Decimal::from_str("0.000000000000340282").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 56).unwrap(),
            Decimal::from_str("0.000000000000000003").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 57).unwrap(),
            Decimal::from_str("0.000000000000000000").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, u32::MAX).unwrap(),
            Decimal::from_str("0.000000000000000000").unwrap()
        );

        // Can be used with max value
        let max = Decimal::MAX;
        assert_eq!(
            Decimal::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
            max
        );

        // Overflow is only possible with digits < 18
        let result = Decimal::from_atomics(u128::MAX, 17);
        assert_eq!(result.unwrap_err(), DecimalRangeExceeded);
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal::from_ratio(1u128, 1u128), Decimal::one());
        assert_eq!(Decimal::from_ratio(53u128, 53u128), Decimal::one());
        assert_eq!(Decimal::from_ratio(125u128, 125u128), Decimal::one());

        // 1.5
        assert_eq!(Decimal::from_ratio(3u128, 2u128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(150u128, 100u128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(333u128, 222u128), Decimal::percent(150));

        // 0.125
        assert_eq!(Decimal::from_ratio(1u64, 8u64), Decimal::permille(125));
        assert_eq!(Decimal::from_ratio(125u64, 1000u64), Decimal::permille(125));

        // 1/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(1u64, 3u64),
            Decimal(Uint128::from(333_333_333_333_333_333u128))
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(2u64, 3u64),
            Decimal(Uint128::from(666_666_666_666_666_666u128))
        );

        // large inputs
        assert_eq!(Decimal::from_ratio(0u128, u128::MAX), Decimal::zero());
        assert_eq!(Decimal::from_ratio(u128::MAX, u128::MAX), Decimal::one());
        // 340282366920938463463 is the largest integer <= Decimal::MAX
        assert_eq!(
            Decimal::from_ratio(340282366920938463463u128, 1u128),
            Decimal::from_str("340282366920938463463").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal::from_ratio(1u128, 0u128);
    }

    #[test]
    fn decimal_implements_fraction() {
        let fraction = Decimal::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Uint128::from(1_234_567_000_000_000_000_000u128)
        );
        assert_eq!(
            fraction.denominator(),
            Uint128::from(1_000_000_000_000_000_000u128)
        );
    }

    #[test]
    fn decimal_from_str_works() {
        // Integers
        assert_eq!(Decimal::from_str("0").unwrap(), Decimal::percent(0));
        assert_eq!(Decimal::from_str("1").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("5").unwrap(), Decimal::percent(500));
        assert_eq!(Decimal::from_str("42").unwrap(), Decimal::percent(4200));
        assert_eq!(Decimal::from_str("000").unwrap(), Decimal::percent(0));
        assert_eq!(Decimal::from_str("001").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("005").unwrap(), Decimal::percent(500));
        assert_eq!(Decimal::from_str("0042").unwrap(), Decimal::percent(4200));

        // Decimals
        assert_eq!(Decimal::from_str("1.0").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("1.5").unwrap(), Decimal::percent(150));
        assert_eq!(Decimal::from_str("0.5").unwrap(), Decimal::percent(50));
        assert_eq!(Decimal::from_str("0.123").unwrap(), Decimal::permille(123));

        assert_eq!(Decimal::from_str("40.00").unwrap(), Decimal::percent(4000));
        assert_eq!(Decimal::from_str("04.00").unwrap(), Decimal::percent(400));
        assert_eq!(Decimal::from_str("00.40").unwrap(), Decimal::percent(40));
        assert_eq!(Decimal::from_str("00.04").unwrap(), Decimal::percent(4));

        // Can handle DECIMAL_PLACES fractional digits
        assert_eq!(
            Decimal::from_str("7.123456789012345678").unwrap(),
            Decimal(Uint128::from(7123456789012345678u128))
        );
        assert_eq!(
            Decimal::from_str("7.999999999999999999").unwrap(),
            Decimal(Uint128::from(7999999999999999999u128))
        );

        // Works for documented max value
        assert_eq!(
            Decimal::from_str("340282366920938463463.374607431768211455").unwrap(),
            Decimal::MAX
        );
    }

    #[test]
    fn decimal_from_str_errors_for_broken_whole_part() {
        match Decimal::from_str("").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_broken_fractinal_part() {
        match Decimal::from_str("1.").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_18_fractional_digits() {
        match Decimal::from_str("7.1234567890123456789").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits",)
            }
            e => panic!("Unexpected error: {:?}", e),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal::from_str("7.1230000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_invalid_number_of_dots() {
        match Decimal::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_max_value() {
        // Integer
        match Decimal::from_str("340282366920938463464").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }

        // Decimal
        match Decimal::from_str("340282366920938463464.0").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
        match Decimal::from_str("340282366920938463463.374607431768211456").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_atomics_works() {
        let zero = Decimal::zero();
        let one = Decimal::one();
        let half = Decimal::percent(50);
        let two = Decimal::percent(200);
        let max = Decimal::MAX;

        assert_eq!(zero.atomics(), Uint128::new(0));
        assert_eq!(one.atomics(), Uint128::new(1000000000000000000));
        assert_eq!(half.atomics(), Uint128::new(500000000000000000));
        assert_eq!(two.atomics(), Uint128::new(2000000000000000000));
        assert_eq!(max.atomics(), Uint128::MAX);
    }

    #[test]
    fn decimal_decimal_places_works() {
        let zero = Decimal::zero();
        let one = Decimal::one();
        let half = Decimal::percent(50);
        let two = Decimal::percent(200);
        let max = Decimal::MAX;

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal_is_zero_works() {
        assert!(Decimal::zero().is_zero());
        assert!(Decimal::percent(0).is_zero());
        assert!(Decimal::permille(0).is_zero());

        assert!(!Decimal::one().is_zero());
        assert!(!Decimal::percent(123).is_zero());
        assert!(!Decimal::permille(1234).is_zero());
    }

    #[test]
    fn decimal_inv_works() {
        // d = 0
        assert_eq!(Decimal::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal::one().inv(), Some(Decimal::one()));

        // d > 1 exact
        assert_eq!(
            Decimal::from_str("2").unwrap().inv(),
            Some(Decimal::from_str("0.5").unwrap())
        );
        assert_eq!(
            Decimal::from_str("20").unwrap().inv(),
            Some(Decimal::from_str("0.05").unwrap())
        );
        assert_eq!(
            Decimal::from_str("200").unwrap().inv(),
            Some(Decimal::from_str("0.005").unwrap())
        );
        assert_eq!(
            Decimal::from_str("2000").unwrap().inv(),
            Some(Decimal::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            Decimal::from_str("3").unwrap().inv(),
            Some(Decimal::from_str("0.333333333333333333").unwrap())
        );
        assert_eq!(
            Decimal::from_str("6").unwrap().inv(),
            Some(Decimal::from_str("0.166666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            Decimal::from_str("0.5").unwrap().inv(),
            Some(Decimal::from_str("2").unwrap())
        );
        assert_eq!(
            Decimal::from_str("0.05").unwrap().inv(),
            Some(Decimal::from_str("20").unwrap())
        );
        assert_eq!(
            Decimal::from_str("0.005").unwrap().inv(),
            Some(Decimal::from_str("200").unwrap())
        );
        assert_eq!(
            Decimal::from_str("0.0005").unwrap().inv(),
            Some(Decimal::from_str("2000").unwrap())
        );
    }

    #[test]
    fn decimal_add() {
        let value = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(
            value.0,
            Decimal::DECIMAL_FRACTIONAL * Uint128::from(3u8) / Uint128::from(2u8)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal_add_overflow_panics() {
        let _value = Decimal::MAX + Decimal::percent(50);
    }

    #[test]
    fn decimal_sub() {
        let value = Decimal::one() - Decimal::percent(50); // 0.5
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(2u8));
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn decimal_sub_overflow_panics() {
        let _value = Decimal::zero() - Decimal::percent(50);
    }

    #[test]
    fn decimal_implements_mul() {
        let one = Decimal::one();
        let two = one + one;
        let half = Decimal::percent(50);

        // 1*x and x*1
        assert_eq!(one * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(one * Decimal::percent(1), Decimal::percent(1));
        assert_eq!(one * Decimal::percent(10), Decimal::percent(10));
        assert_eq!(one * Decimal::percent(100), Decimal::percent(100));
        assert_eq!(one * Decimal::percent(1000), Decimal::percent(1000));
        assert_eq!(one * Decimal::MAX, Decimal::MAX);
        assert_eq!(Decimal::percent(0) * one, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * one, Decimal::percent(1));
        assert_eq!(Decimal::percent(10) * one, Decimal::percent(10));
        assert_eq!(Decimal::percent(100) * one, Decimal::percent(100));
        assert_eq!(Decimal::percent(1000) * one, Decimal::percent(1000));
        assert_eq!(Decimal::MAX * one, Decimal::MAX);

        // double
        assert_eq!(two * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(two * Decimal::percent(1), Decimal::percent(2));
        assert_eq!(two * Decimal::percent(10), Decimal::percent(20));
        assert_eq!(two * Decimal::percent(100), Decimal::percent(200));
        assert_eq!(two * Decimal::percent(1000), Decimal::percent(2000));
        assert_eq!(Decimal::percent(0) * two, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * two, Decimal::percent(2));
        assert_eq!(Decimal::percent(10) * two, Decimal::percent(20));
        assert_eq!(Decimal::percent(100) * two, Decimal::percent(200));
        assert_eq!(Decimal::percent(1000) * two, Decimal::percent(2000));

        // half
        assert_eq!(half * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(half * Decimal::percent(1), Decimal::permille(5));
        assert_eq!(half * Decimal::percent(10), Decimal::percent(5));
        assert_eq!(half * Decimal::percent(100), Decimal::percent(50));
        assert_eq!(half * Decimal::percent(1000), Decimal::percent(500));
        assert_eq!(Decimal::percent(0) * half, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * half, Decimal::permille(5));
        assert_eq!(Decimal::percent(10) * half, Decimal::percent(5));
        assert_eq!(Decimal::percent(100) * half, Decimal::percent(50));
        assert_eq!(Decimal::percent(1000) * half, Decimal::percent(500));

        fn dec(input: &str) -> Decimal {
            Decimal::from_str(input).unwrap()
        }

        // Move left
        let a = dec("123.127726548762582");
        assert_eq!(a * dec("1"), dec("123.127726548762582"));
        assert_eq!(a * dec("10"), dec("1231.27726548762582"));
        assert_eq!(a * dec("100"), dec("12312.7726548762582"));
        assert_eq!(a * dec("1000"), dec("123127.726548762582"));
        assert_eq!(a * dec("1000000"), dec("123127726.548762582"));
        assert_eq!(a * dec("1000000000"), dec("123127726548.762582"));
        assert_eq!(a * dec("1000000000000"), dec("123127726548762.582"));
        assert_eq!(a * dec("1000000000000000"), dec("123127726548762582"));
        assert_eq!(a * dec("1000000000000000000"), dec("123127726548762582000"));
        assert_eq!(dec("1") * a, dec("123.127726548762582"));
        assert_eq!(dec("10") * a, dec("1231.27726548762582"));
        assert_eq!(dec("100") * a, dec("12312.7726548762582"));
        assert_eq!(dec("1000") * a, dec("123127.726548762582"));
        assert_eq!(dec("1000000") * a, dec("123127726.548762582"));
        assert_eq!(dec("1000000000") * a, dec("123127726548.762582"));
        assert_eq!(dec("1000000000000") * a, dec("123127726548762.582"));
        assert_eq!(dec("1000000000000000") * a, dec("123127726548762582"));
        assert_eq!(dec("1000000000000000000") * a, dec("123127726548762582000"));

        // Move right
        let max = Decimal::MAX;
        assert_eq!(
            max * dec("1.0"),
            dec("340282366920938463463.374607431768211455")
        );
        assert_eq!(
            max * dec("0.1"),
            dec("34028236692093846346.337460743176821145")
        );
        assert_eq!(
            max * dec("0.01"),
            dec("3402823669209384634.633746074317682114")
        );
        assert_eq!(
            max * dec("0.001"),
            dec("340282366920938463.463374607431768211")
        );
        assert_eq!(
            max * dec("0.000001"),
            dec("340282366920938.463463374607431768")
        );
        assert_eq!(
            max * dec("0.000000001"),
            dec("340282366920.938463463374607431")
        );
        assert_eq!(
            max * dec("0.000000000001"),
            dec("340282366.920938463463374607")
        );
        assert_eq!(
            max * dec("0.000000000000001"),
            dec("340282.366920938463463374")
        );
        assert_eq!(
            max * dec("0.000000000000000001"),
            dec("340.282366920938463463")
        );
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_mul_overflow_panics() {
        let _value = Decimal::MAX * Decimal::percent(101);
    }

    #[test]
    // in this test the Decimal is on the right
    fn uint128_decimal_multiply() {
        // a*b
        let left = Uint128::new(300);
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, Uint128::new(450));

        // a*0
        let left = Uint128::new(300);
        let right = Decimal::zero();
        assert_eq!(left * right, Uint128::new(0));

        // 0*a
        let left = Uint128::new(0);
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, Uint128::new(0));
    }

    #[test]
    // in this test the Decimal is on the left
    fn decimal_uint128_multiply() {
        // a*b
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = Uint128::new(300);
        assert_eq!(left * right, Uint128::new(450));

        // 0*a
        let left = Decimal::zero();
        let right = Uint128::new(300);
        assert_eq!(left * right, Uint128::new(0));

        // a*0
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = Uint128::new(0);
        assert_eq!(left * right, Uint128::new(0));
    }

    #[test]
    fn decimal_uint128_division() {
        // a/b
        let left = Decimal::percent(150); // 1.5
        let right = Uint128::new(3);
        assert_eq!(left / right, Decimal::percent(50));

        // 0/a
        let left = Decimal::zero();
        let right = Uint128::new(300);
        assert_eq!(left / right, Decimal::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_divide_by_zero() {
        let left = Decimal::percent(150); // 1.5
        let right = Uint128::new(0);
        let _result = left / right;
    }

    #[test]
    fn decimal_uint128_div_assign() {
        // a/b
        let mut dec = Decimal::percent(150); // 1.5
        dec /= Uint128::new(3);
        assert_eq!(dec, Decimal::percent(50));

        // 0/a
        let mut dec = Decimal::zero();
        dec /= Uint128::new(300);
        assert_eq!(dec, Decimal::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_div_assign_by_zero() {
        // a/0
        let mut dec = Decimal::percent(50);
        dec /= Uint128::new(0);
    }

    #[test]
    fn decimal_uint128_sqrt() {
        assert_eq!(Decimal::percent(900).sqrt(), Decimal::percent(300));

        assert!(Decimal::percent(316) < Decimal::percent(1000).sqrt());
        assert!(Decimal::percent(1000).sqrt() < Decimal::percent(317));
    }

    /// sqrt(2) is an irrational number, i.e. all 18 decimal places should be used.
    #[test]
    fn decimal_uint128_sqrt_is_precise() {
        assert_eq!(
            Decimal::from_str("2").unwrap().sqrt(),
            Decimal::from_str("1.414213562373095048").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
        );
    }

    #[test]
    fn decimal_uint128_sqrt_does_not_overflow() {
        assert_eq!(
            Decimal::from_str("400").unwrap().sqrt(),
            Decimal::from_str("20").unwrap()
        );
    }

    #[test]
    fn decimal_uint128_sqrt_intermediate_precision_used() {
        assert_eq!(
            Decimal::from_str("400001").unwrap().sqrt(),
            // The last two digits (27) are truncated below due to the algorithm
            // we use. Larger numbers will cause less precision.
            // https://www.wolframalpha.com/input/?i=sqrt%28400001%29
            Decimal::from_str("632.456322602596803200").unwrap()
        );
    }

    #[test]
    fn decimal_to_string() {
        // Integers
        assert_eq!(Decimal::zero().to_string(), "0");
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::percent(500).to_string(), "5");

        // Decimals
        assert_eq!(Decimal::percent(125).to_string(), "1.25");
        assert_eq!(Decimal::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal::percent(3).to_string(), "0.03");
        assert_eq!(Decimal::permille(987).to_string(), "0.987");

        assert_eq!(
            Decimal(Uint128::from(1u128)).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10u128)).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100u128)).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000u128)).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000u128)).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000u128)).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000000u128)).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000u128)).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000u128)).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000000000u128)).to_string(),
            "0.000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000000u128)).to_string(),
            "0.00000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000000u128)).to_string(),
            "0.0000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000000000u128)).to_string(),
            "0.00001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000000000u128)).to_string(),
            "0.0001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000000000000000u128)).to_string(),
            "0.001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000000000000u128)).to_string(),
            "0.01"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000000000000u128)).to_string(),
            "0.1"
        );
    }

    #[test]
    fn decimal_iter_sum() {
        let items = vec![
            Decimal::zero(),
            Decimal(Uint128::from(2u128)),
            Decimal(Uint128::from(2u128)),
        ];
        assert_eq!(items.iter().sum::<Decimal>(), Decimal(Uint128::from(4u128)));
        assert_eq!(
            items.into_iter().sum::<Decimal>(),
            Decimal(Uint128::from(4u128))
        );

        let empty: Vec<Decimal> = vec![];
        assert_eq!(Decimal::zero(), empty.iter().sum());
    }

    #[test]
    fn decimal_serialize() {
        assert_eq!(to_vec(&Decimal::zero()).unwrap(), br#""0""#);
        assert_eq!(to_vec(&Decimal::one()).unwrap(), br#""1""#);
        assert_eq!(to_vec(&Decimal::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_vec(&Decimal::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(to_vec(&Decimal::percent(876)).unwrap(), br#""8.76""#);
        assert_eq!(to_vec(&Decimal::percent(8765)).unwrap(), br#""87.65""#);
    }

    #[test]
    fn decimal_deserialize() {
        assert_eq!(from_slice::<Decimal>(br#""0""#).unwrap(), Decimal::zero());
        assert_eq!(from_slice::<Decimal>(br#""1""#).unwrap(), Decimal::one());
        assert_eq!(from_slice::<Decimal>(br#""000""#).unwrap(), Decimal::zero());
        assert_eq!(from_slice::<Decimal>(br#""001""#).unwrap(), Decimal::one());

        assert_eq!(
            from_slice::<Decimal>(br#""0.08""#).unwrap(),
            Decimal::percent(8)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""0.87""#).unwrap(),
            Decimal::percent(87)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""8.76""#).unwrap(),
            Decimal::percent(876)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""87.65""#).unwrap(),
            Decimal::percent(8765)
        );
    }
}
