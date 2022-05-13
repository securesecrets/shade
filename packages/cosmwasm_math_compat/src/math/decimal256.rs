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

use crate::{errors::StdError, Uint512};

use super::{Fraction, Isqrt, Uint256};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal256(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is
/// 115792089237316195423570985008687907853269984665640564039457.584007913129639935
/// (which is (2^256 - 1) / 10^18)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal256(#[schemars(with = "String")] Uint256);

#[derive(Snafu, Debug, PartialEq)]
#[snafu(display("Decimal256 range exceeded"))]
pub struct Decimal256RangeExceeded;

impl Decimal256 {
    const DECIMAL_FRACTIONAL: Uint256 = // 1*10**18
        Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 224, 182,
            179, 167, 100, 0, 0,
        ]);
    const DECIMAL_FRACTIONAL_SQUARED: Uint256 = // 1*10**36
        Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 151, 206, 123, 201, 7, 21, 179,
            75, 159, 16, 0, 0, 0, 0,
        ]);
    const DECIMAL_PLACES: usize = 18;
    pub const MAX: Self = Self(Uint256::MAX);

    /// Create a 1.0 Decimal256
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal256
    pub const fn zero() -> Self {
        Self(Uint256::zero())
    }

    /// Convert x% into Decimal256
    pub fn percent(x: u64) -> Self {
        Self(Uint256::from(x) * Uint256::from(10_000_000_000_000_000u128))
    }

    /// Convert permille (x/1000) into Decimal256
    pub fn permille(x: u64) -> Self {
        Self(Uint256::from(x) * Uint256::from(1_000_000_000_000_000u128))
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
    /// # use cosmwasm_math::{Decimal256, Uint256};
    /// let a = Decimal256::from_atomics(1234u64, 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = Decimal256::from_atomics(1234u128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = Decimal256::from_atomics(1u64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    ///
    /// let a = Decimal256::from_atomics(Uint256::MAX, 18).unwrap();
    /// assert_eq!(a, Decimal256::MAX);
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Uint256>,
        decimal_places: u32,
    ) -> Result<Self, Decimal256RangeExceeded> {
        let atomics = atomics.into();
        let ten = Uint256::from(10u64); // TODO: make const
        Ok(match decimal_places.cmp(&(Self::DECIMAL_PLACES as u32)) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES as u32) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = ten.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| Decimal256RangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES as u32); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = ten.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Uint256 range.
                    // Any Uint256 `x` divided by `factor` with `factor > Uint256::MAX` is 0.
                    // Try e.g. Python3: `(2**256-1) // 2**256`
                    Self(Uint256::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a Decimal256
    pub fn from_ratio(numerator: impl Into<Uint256>, denominator: impl Into<Uint256>) -> Self {
        let numerator: Uint256 = numerator.into();
        let denominator: Uint256 = denominator.into();
        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }

        Self(
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
    /// # use cosmwasm_math::{Decimal256, Uint256};
    /// # use std::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = Decimal256::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Uint256::from(1234000000000000000u128));
    ///
    /// // Smallest possible value
    /// let b = Decimal256::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Uint256::from(1u128));
    /// ```
    pub fn atomics(&self) -> Uint256 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`Decimal256::atomics()`].
    pub fn decimal_places(&self) -> u32 {
        Self::DECIMAL_PLACES as u32
    }

    /// Returns the approximate square root as a Decimal256.
    ///
    /// This should not overflow or panic.
    pub fn sqrt(&self) -> Self {
        // Algorithm described in https://hackmd.io/@webmaster128/SJThlukj_
        // We start with the highest precision possible and lower it until
        // there's no overflow.
        //
        // TODO: This could be made more efficient once log10 is in:
        // https://github.com/rust-lang/rust/issues/70887
        // The max precision is something like `18 - log10(self.0) / 2`.
        (0..=Self::DECIMAL_PLACES / 2)
            .rev()
            .find_map(|i| self.sqrt_with_precision(i))
            // The last step (i = 0) is guaranteed to succeed because `isqrt(Uint256::MAX) * 10^9` does not overflow
            .unwrap()
    }

    /// Lower precision means more aggressive rounding, but less risk of overflow.
    /// Precision *must* be a number between 0 and 9 (inclusive).
    ///
    /// Returns `None` if the internal multiplication overflows.
    fn sqrt_with_precision(&self, precision: usize) -> Option<Self> {
        let precision = precision as u32;

        let inner_mul = Uint256::from(100u128).pow(precision);
        self.0.checked_mul(inner_mul).ok().map(|inner| {
            let outer_mul = Uint256::from(10u128).pow(Self::DECIMAL_PLACES as u32 / 2 - precision);
            Self(inner.isqrt().checked_mul(outer_mul).unwrap())
        })
    }
}

impl Fraction<Uint256> for Decimal256 {
    #[inline]
    fn numerator(&self) -> Uint256 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Uint256 {
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
            Some(Self(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl FromStr for Decimal256 {
    type Err = StdError;

    /// Converts the decimal string to a Decimal256
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than DECIMAL_PLACES fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part
            .parse::<Uint256>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(Self::DECIMAL_FRACTIONAL)
            .map_err(|_| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<Uint256>()
                .map_err(|_| StdError::generic_err("Error parsing fractional"))?;
            let exp =
                (Self::DECIMAL_PLACES.checked_sub(fractional_part.len())).ok_or_else(|| {
                    StdError::generic_err(format!(
                        "Cannot parse more than {} fractional digits",
                        Self::DECIMAL_PLACES
                    ))
                })?;
            debug_assert!(exp <= Self::DECIMAL_PLACES);
            let fractional_factor = Uint256::from(10u128).pow(exp as u32);
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

        Ok(Self(atomics))
    }
}

impl fmt::Display for Decimal256 {
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

impl ops::Add for Decimal256 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl ops::Add<&Decimal256> for Decimal256 {
    type Output = Self;

    fn add(self, other: &Decimal256) -> Self {
        Self(self.0 + other.0)
    }
}

impl ops::Sub for Decimal256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl ops::Mul for Decimal256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // Decimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_uint512 = self.numerator().full_mul(other.numerator())
            / Uint512::from_uint256(Self::DECIMAL_FRACTIONAL); // from_uint256 is a const method and should be "free"
        match result_as_uint512.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}

/// Both d*u and u*d with d: Decimal256 and u: Uint256 returns an Uint256. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal256 result for the same calculation, use Decimal256(d*u) or Decimal256(u*d).
impl ops::Mul<Decimal256> for Uint256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal256) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint256::zero();
        }
        self.multiply_ratio(rhs.0, Decimal256::DECIMAL_FRACTIONAL)
    }
}

impl ops::Mul<Uint256> for Decimal256 {
    type Output = Uint256;

    fn mul(self, rhs: Uint256) -> Self::Output {
        rhs * self
    }
}

impl ops::Div<Uint256> for Decimal256 {
    type Output = Self;

    fn div(self, rhs: Uint256) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::DivAssign<Uint256> for Decimal256 {
    fn div_assign(&mut self, rhs: Uint256) {
        self.0 /= rhs;
    }
}

impl<A> std::iter::Sum<A> for Decimal256
where
    Self: ops::Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), ops::Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Decimal256Visitor)
    }
}

struct Decimal256Visitor;

impl<'de> de::Visitor<'de> for Decimal256Visitor {
    type Value = Decimal256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Self::Value::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::StdError;
    use cosmwasm_std::{from_slice, to_vec};

    #[test]
    fn decimal256_one() {
        let value = Decimal256::one();
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal256_zero() {
        let value = Decimal256::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn decimal256_percent() {
        let value = Decimal256::percent(50);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / Uint256::from(2u8));
    }

    #[test]
    fn decimal256_permille() {
        let value = Decimal256::permille(125);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / Uint256::from(8u8));
    }

    #[test]
    fn decimal256_from_atomics_works() {
        let one = Decimal256::one();
        let two = one + one;

        assert_eq!(Decimal256::from_atomics(1u128, 0).unwrap(), one);
        assert_eq!(Decimal256::from_atomics(10u128, 1).unwrap(), one);
        assert_eq!(Decimal256::from_atomics(100u128, 2).unwrap(), one);
        assert_eq!(Decimal256::from_atomics(1000u128, 3).unwrap(), one);
        assert_eq!(
            Decimal256::from_atomics(1000000000000000000u128, 18).unwrap(),
            one
        );
        assert_eq!(
            Decimal256::from_atomics(10000000000000000000u128, 19).unwrap(),
            one
        );
        assert_eq!(
            Decimal256::from_atomics(100000000000000000000u128, 20).unwrap(),
            one
        );

        assert_eq!(Decimal256::from_atomics(2u128, 0).unwrap(), two);
        assert_eq!(Decimal256::from_atomics(20u128, 1).unwrap(), two);
        assert_eq!(Decimal256::from_atomics(200u128, 2).unwrap(), two);
        assert_eq!(Decimal256::from_atomics(2000u128, 3).unwrap(), two);
        assert_eq!(
            Decimal256::from_atomics(2000000000000000000u128, 18).unwrap(),
            two
        );
        assert_eq!(
            Decimal256::from_atomics(20000000000000000000u128, 19).unwrap(),
            two
        );
        assert_eq!(
            Decimal256::from_atomics(200000000000000000000u128, 20).unwrap(),
            two
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            Decimal256::from_atomics(4321u128, 20).unwrap(),
            Decimal256::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(6789u128, 20).unwrap(),
            Decimal256::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 38).unwrap(),
            Decimal256::from_str("3.402823669209384634").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 39).unwrap(),
            Decimal256::from_str("0.340282366920938463").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 45).unwrap(),
            Decimal256::from_str("0.000000340282366920").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 51).unwrap(),
            Decimal256::from_str("0.000000000000340282").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 56).unwrap(),
            Decimal256::from_str("0.000000000000000003").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 57).unwrap(),
            Decimal256::from_str("0.000000000000000000").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, u32::MAX).unwrap(),
            Decimal256::from_str("0.000000000000000000").unwrap()
        );

        // Can be used with max value
        let max = Decimal256::MAX;
        assert_eq!(
            Decimal256::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
            max
        );

        // Overflow is only possible with digits < 18
        let result = Decimal256::from_atomics(Uint256::MAX, 17);
        assert_eq!(result.unwrap_err(), Decimal256RangeExceeded);
    }

    #[test]
    fn decimal256_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal256::from_ratio(1u128, 1u128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(53u128, 53u128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(125u128, 125u128), Decimal256::one());

        // 1.5
        assert_eq!(
            Decimal256::from_ratio(3u128, 2u128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(150u128, 100u128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(333u128, 222u128),
            Decimal256::percent(150)
        );

        // 0.125
        assert_eq!(
            Decimal256::from_ratio(1u64, 8u64),
            Decimal256::permille(125)
        );
        assert_eq!(
            Decimal256::from_ratio(125u64, 1000u64),
            Decimal256::permille(125)
        );

        // 1/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(1u64, 3u64),
            Decimal256(Uint256::from_str("333333333333333333").unwrap())
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(2u64, 3u64),
            Decimal256(Uint256::from_str("666666666666666666").unwrap())
        );

        // large inputs
        assert_eq!(Decimal256::from_ratio(0u128, u128::MAX), Decimal256::zero());
        assert_eq!(
            Decimal256::from_ratio(u128::MAX, u128::MAX),
            Decimal256::one()
        );
        // 340282366920938463463 is the largest integer <= Decimal256::MAX
        assert_eq!(
            Decimal256::from_ratio(340282366920938463463u128, 1u128),
            Decimal256::from_str("340282366920938463463").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal256_from_ratio_panics_for_zero_denominator() {
        Decimal256::from_ratio(1u128, 0u128);
    }

    #[test]
    fn decimal256_implements_fraction() {
        let fraction = Decimal256::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Uint256::from_str("1234567000000000000000").unwrap()
        );
        assert_eq!(
            fraction.denominator(),
            Uint256::from_str("1000000000000000000").unwrap()
        );
    }

    #[test]
    fn decimal256_from_str_works() {
        // Integers
        assert_eq!(Decimal256::from_str("0").unwrap(), Decimal256::percent(0));
        assert_eq!(Decimal256::from_str("1").unwrap(), Decimal256::percent(100));
        assert_eq!(Decimal256::from_str("5").unwrap(), Decimal256::percent(500));
        assert_eq!(
            Decimal256::from_str("42").unwrap(),
            Decimal256::percent(4200)
        );
        assert_eq!(Decimal256::from_str("000").unwrap(), Decimal256::percent(0));
        assert_eq!(
            Decimal256::from_str("001").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("005").unwrap(),
            Decimal256::percent(500)
        );
        assert_eq!(
            Decimal256::from_str("0042").unwrap(),
            Decimal256::percent(4200)
        );

        // Decimals
        assert_eq!(
            Decimal256::from_str("1.0").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("1.5").unwrap(),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::percent(50)
        );
        assert_eq!(
            Decimal256::from_str("0.123").unwrap(),
            Decimal256::permille(123)
        );

        assert_eq!(
            Decimal256::from_str("40.00").unwrap(),
            Decimal256::percent(4000)
        );
        assert_eq!(
            Decimal256::from_str("04.00").unwrap(),
            Decimal256::percent(400)
        );
        assert_eq!(
            Decimal256::from_str("00.40").unwrap(),
            Decimal256::percent(40)
        );
        assert_eq!(
            Decimal256::from_str("00.04").unwrap(),
            Decimal256::percent(4)
        );

        // Can handle 18 fractional digits
        assert_eq!(
            Decimal256::from_str("7.123456789012345678").unwrap(),
            Decimal256(Uint256::from(7123456789012345678u128))
        );
        assert_eq!(
            Decimal256::from_str("7.999999999999999999").unwrap(),
            Decimal256(Uint256::from(7999999999999999999u128))
        );

        // Works for documented max value
        assert_eq!(
            Decimal256::from_str(
                "115792089237316195423570985008687907853269984665640564039457.584007913129639935"
            )
            .unwrap(),
            Decimal256::MAX
        );
    }

    #[test]
    fn decimal256_from_str_errors_for_broken_whole_part() {
        match Decimal256::from_str("").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_broken_fractinal_part() {
        match Decimal256::from_str("1.").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_more_than_36_fractional_digits() {
        match Decimal256::from_str("7.1234567890123456789").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal256::from_str("7.1230000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_invalid_number_of_dots() {
        match Decimal256::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_more_than_max_value() {
        // Integer
        match Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458")
            .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }

        // Decimal
        match Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458.0")
            .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
        match Decimal256::from_str(
            "115792089237316195423570985008687907853269984665640564039457.584007913129639936",
        )
        .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal256_atomics_works() {
        let zero = Decimal256::zero();
        let one = Decimal256::one();
        let half = Decimal256::percent(50);
        let two = Decimal256::percent(200);
        let max = Decimal256::MAX;

        assert_eq!(zero.atomics(), Uint256::from(0u128));
        assert_eq!(one.atomics(), Uint256::from(1000000000000000000u128));
        assert_eq!(half.atomics(), Uint256::from(500000000000000000u128));
        assert_eq!(two.atomics(), Uint256::from(2000000000000000000u128));
        assert_eq!(max.atomics(), Uint256::MAX);
    }

    #[test]
    fn decimal256_decimal_places_works() {
        let zero = Decimal256::zero();
        let one = Decimal256::one();
        let half = Decimal256::percent(50);
        let two = Decimal256::percent(200);
        let max = Decimal256::MAX;

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal256_is_zero_works() {
        assert!(Decimal256::zero().is_zero());
        assert!(Decimal256::percent(0).is_zero());
        assert!(Decimal256::permille(0).is_zero());

        assert!(!Decimal256::one().is_zero());
        assert!(!Decimal256::percent(123).is_zero());
        assert!(!Decimal256::permille(1234).is_zero());
    }

    #[test]
    fn decimal256_inv_works() {
        // d = 0
        assert_eq!(Decimal256::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal256::one().inv(), Some(Decimal256::one()));

        // d > 1 exact
        assert_eq!(
            Decimal256::from_str("2").unwrap().inv(),
            Some(Decimal256::from_str("0.5").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("20").unwrap().inv(),
            Some(Decimal256::from_str("0.05").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("200").unwrap().inv(),
            Some(Decimal256::from_str("0.005").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("2000").unwrap().inv(),
            Some(Decimal256::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            Decimal256::from_str("3").unwrap().inv(),
            Some(Decimal256::from_str("0.333333333333333333").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("6").unwrap().inv(),
            Some(Decimal256::from_str("0.166666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            Decimal256::from_str("0.5").unwrap().inv(),
            Some(Decimal256::from_str("2").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.05").unwrap().inv(),
            Some(Decimal256::from_str("20").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.005").unwrap().inv(),
            Some(Decimal256::from_str("200").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.0005").unwrap().inv(),
            Some(Decimal256::from_str("2000").unwrap())
        );
    }

    #[test]
    fn decimal256_add() {
        let value = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(
            value.0,
            Decimal256::DECIMAL_FRACTIONAL * Uint256::from(3u8) / Uint256::from(2u8)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal256_add_overflow_panics() {
        let _value = Decimal256::MAX + Decimal256::percent(50);
    }

    #[test]
    fn decimal256_sub() {
        let value = Decimal256::one() - Decimal256::percent(50); // 0.5
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / Uint256::from(2u8));
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn decimal256_sub_overflow_panics() {
        let _value = Decimal256::zero() - Decimal256::percent(50);
    }

    #[test]
    fn decimal256_implements_mul() {
        let one = Decimal256::one();
        let two = one + one;
        let half = Decimal256::percent(50);

        // 1*x and x*1
        assert_eq!(one * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(one * Decimal256::percent(1), Decimal256::percent(1));
        assert_eq!(one * Decimal256::percent(10), Decimal256::percent(10));
        assert_eq!(one * Decimal256::percent(100), Decimal256::percent(100));
        assert_eq!(one * Decimal256::percent(1000), Decimal256::percent(1000));
        assert_eq!(one * Decimal256::MAX, Decimal256::MAX);
        assert_eq!(Decimal256::percent(0) * one, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * one, Decimal256::percent(1));
        assert_eq!(Decimal256::percent(10) * one, Decimal256::percent(10));
        assert_eq!(Decimal256::percent(100) * one, Decimal256::percent(100));
        assert_eq!(Decimal256::percent(1000) * one, Decimal256::percent(1000));
        assert_eq!(Decimal256::MAX * one, Decimal256::MAX);

        // double
        assert_eq!(two * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(two * Decimal256::percent(1), Decimal256::percent(2));
        assert_eq!(two * Decimal256::percent(10), Decimal256::percent(20));
        assert_eq!(two * Decimal256::percent(100), Decimal256::percent(200));
        assert_eq!(two * Decimal256::percent(1000), Decimal256::percent(2000));
        assert_eq!(Decimal256::percent(0) * two, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * two, Decimal256::percent(2));
        assert_eq!(Decimal256::percent(10) * two, Decimal256::percent(20));
        assert_eq!(Decimal256::percent(100) * two, Decimal256::percent(200));
        assert_eq!(Decimal256::percent(1000) * two, Decimal256::percent(2000));

        // half
        assert_eq!(half * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(half * Decimal256::percent(1), Decimal256::permille(5));
        assert_eq!(half * Decimal256::percent(10), Decimal256::percent(5));
        assert_eq!(half * Decimal256::percent(100), Decimal256::percent(50));
        assert_eq!(half * Decimal256::percent(1000), Decimal256::percent(500));
        assert_eq!(Decimal256::percent(0) * half, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * half, Decimal256::permille(5));
        assert_eq!(Decimal256::percent(10) * half, Decimal256::percent(5));
        assert_eq!(Decimal256::percent(100) * half, Decimal256::percent(50));
        assert_eq!(Decimal256::percent(1000) * half, Decimal256::percent(500));

        fn dec(input: &str) -> Decimal256 {
            Decimal256::from_str(input).unwrap()
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
        let max = Decimal256::MAX;
        assert_eq!(
            max * dec("1.0"),
            dec("115792089237316195423570985008687907853269984665640564039457.584007913129639935")
        );
        assert_eq!(
            max * dec("0.1"),
            dec("11579208923731619542357098500868790785326998466564056403945.758400791312963993")
        );
        assert_eq!(
            max * dec("0.01"),
            dec("1157920892373161954235709850086879078532699846656405640394.575840079131296399")
        );
        assert_eq!(
            max * dec("0.001"),
            dec("115792089237316195423570985008687907853269984665640564039.457584007913129639")
        );
        assert_eq!(
            max * dec("0.000001"),
            dec("115792089237316195423570985008687907853269984665640564.039457584007913129")
        );
        assert_eq!(
            max * dec("0.000000001"),
            dec("115792089237316195423570985008687907853269984665640.564039457584007913")
        );
        assert_eq!(
            max * dec("0.000000000001"),
            dec("115792089237316195423570985008687907853269984665.640564039457584007")
        );
        assert_eq!(
            max * dec("0.000000000000001"),
            dec("115792089237316195423570985008687907853269984.665640564039457584")
        );
        assert_eq!(
            max * dec("0.000000000000000001"),
            dec("115792089237316195423570985008687907853269.984665640564039457")
        );
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal256_mul_overflow_panics() {
        let _value = Decimal256::MAX * Decimal256::percent(101);
    }

    #[test]
    // in this test the Decimal256 is on the right
    fn uint128_decimal_multiply() {
        // a*b
        let left = Uint256::from(300u128);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(450u32));

        // a*0
        let left = Uint256::from(300u128);
        let right = Decimal256::zero();
        assert_eq!(left * right, Uint256::from(0u128));

        // 0*a
        let left = Uint256::from(0u128);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(0u128));
    }

    #[test]
    // in this test the Decimal256 is on the left
    fn decimal256_uint128_multiply() {
        // a*b
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(300u128);
        assert_eq!(left * right, Uint256::from(450u128));

        // 0*a
        let left = Decimal256::zero();
        let right = Uint256::from(300u128);
        assert_eq!(left * right, Uint256::from(0u128));

        // a*0
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(0u128);
        assert_eq!(left * right, Uint256::from(0u128));
    }

    #[test]
    fn decimal256_uint128_division() {
        // a/b
        let left = Decimal256::percent(150); // 1.5
        let right = Uint256::from(3u128);
        assert_eq!(left / right, Decimal256::percent(50));

        // 0/a
        let left = Decimal256::zero();
        let right = Uint256::from(300u128);
        assert_eq!(left / right, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal256_uint128_divide_by_zero() {
        let left = Decimal256::percent(150); // 1.5
        let right = Uint256::from(0u128);
        let _result = left / right;
    }

    #[test]
    fn decimal256_uint128_div_assign() {
        // a/b
        let mut dec = Decimal256::percent(150); // 1.5
        dec /= Uint256::from(3u128);
        assert_eq!(dec, Decimal256::percent(50));

        // 0/a
        let mut dec = Decimal256::zero();
        dec /= Uint256::from(300u128);
        assert_eq!(dec, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal256_uint128_div_assign_by_zero() {
        // a/0
        let mut dec = Decimal256::percent(50);
        dec /= Uint256::from(0u128);
    }

    #[test]
    fn decimal256_uint128_sqrt() {
        assert_eq!(Decimal256::percent(900).sqrt(), Decimal256::percent(300));

        assert!(Decimal256::percent(316) < Decimal256::percent(1000).sqrt());
        assert!(Decimal256::percent(1000).sqrt() < Decimal256::percent(317));
    }

    /// sqrt(2) is an irrational number, i.e. all 36 decimal places should be used.
    #[test]
    fn decimal256_uint128_sqrt_is_precise() {
        assert_eq!(
            Decimal256::from_str("2").unwrap().sqrt(),
            Decimal256::from_str("1.414213562373095048").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
        );
    }

    #[test]
    fn decimal256_uint128_sqrt_does_not_overflow() {
        assert_eq!(
            Decimal256::from_str("40000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .sqrt(),
            Decimal256::from_str("200000000000000000000000000000").unwrap()
        );
    }

    #[test]
    fn decimal256_uint128_sqrt_intermediate_precision_used() {
        assert_eq!(
            Decimal256::from_str("40000000000000000000000000000000000000000000000001")
                .unwrap()
                .sqrt(),
            // The last few digits (39110) are truncated below due to the algorithm
            // we use. Larger numbers will cause less precision.
            // https://www.wolframalpha.com/input/?i=sqrt%2840000000000000000000000000000000000000000000000001%29
            Decimal256::from_str("6324555320336758663997787.088865437067400000").unwrap()
        );
    }

    #[test]
    fn decimal256_to_string() {
        // Integers
        assert_eq!(Decimal256::zero().to_string(), "0");
        assert_eq!(Decimal256::one().to_string(), "1");
        assert_eq!(Decimal256::percent(500).to_string(), "5");

        // Decimals
        assert_eq!(Decimal256::percent(125).to_string(), "1.25");
        assert_eq!(Decimal256::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal256::percent(3).to_string(), "0.03");
        assert_eq!(Decimal256::permille(987).to_string(), "0.987");

        assert_eq!(
            Decimal256(Uint256::from(1u128)).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10u128)).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100u128)).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000u128)).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000u128)).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000u128)).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000u128)).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000u128)).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000u128)).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000000u128)).to_string(),
            "0.000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000u128)).to_string(),
            "0.00000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000u128)).to_string(),
            "0.0000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000000u128)).to_string(),
            "0.00001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000000u128)).to_string(),
            "0.0001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000000000000u128)).to_string(),
            "0.001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000000000u128)).to_string(),
            "0.01"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000000000u128)).to_string(),
            "0.1"
        );
    }

    #[test]
    fn decimal256_iter_sum() {
        let items = vec![
            Decimal256::zero(),
            Decimal256::from_str("2").unwrap(),
            Decimal256::from_str("2").unwrap(),
        ];
        assert_eq!(
            items.iter().sum::<Decimal256>(),
            Decimal256::from_str("4").unwrap()
        );
        assert_eq!(
            items.into_iter().sum::<Decimal256>(),
            Decimal256::from_str("4").unwrap()
        );

        let empty: Vec<Decimal256> = vec![];
        assert_eq!(Decimal256::zero(), empty.iter().sum());
    }

    #[test]
    fn decimal256_serialize() {
        assert_eq!(to_vec(&Decimal256::zero()).unwrap(), br#""0""#);
        assert_eq!(to_vec(&Decimal256::one()).unwrap(), br#""1""#);
        assert_eq!(to_vec(&Decimal256::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_vec(&Decimal256::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(to_vec(&Decimal256::percent(876)).unwrap(), br#""8.76""#);
        assert_eq!(to_vec(&Decimal256::percent(8765)).unwrap(), br#""87.65""#);
    }

    #[test]
    fn decimal256_deserialize() {
        assert_eq!(
            from_slice::<Decimal256>(br#""0""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""1""#).unwrap(),
            Decimal256::one()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""000""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""001""#).unwrap(),
            Decimal256::one()
        );

        assert_eq!(
            from_slice::<Decimal256>(br#""0.08""#).unwrap(),
            Decimal256::percent(8)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""0.87""#).unwrap(),
            Decimal256::percent(87)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""8.76""#).unwrap(),
            Decimal256::percent(876)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""87.65""#).unwrap(),
            Decimal256::percent(8765)
        );
    }
}
