//! ### Liquidity Book u24 Library
//! Author: Haseeb
//!

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    fmt,
    ops::{Add, Div, Mul, Sub},
};

pub struct U24(u32);

impl U24 {
    pub const MAX: u32 = 0xFFFFFF;

    pub fn new(value: u32) -> Option<Self> {
        if value <= Self::MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl PartialEq for U24 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for U24 {}

impl PartialOrd for U24 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U24 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Add for U24 {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Option<Self> {
        Self::new(self.0 + other.0)
    }
}

impl Sub for U24 {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).and_then(Self::new)
    }
}

impl Mul for U24 {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Option<Self> {
        self.0.checked_mul(other.0).and_then(Self::new)
    }
}

impl Div for U24 {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Option<Self> {
        if other.0 == 0 {
            None
        } else {
            self.0.checked_div(other.0).and_then(Self::new)
        }
    }
}

impl fmt::Debug for U24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "U24({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Div, Mul, Sub};

    use super::U24;

    #[test]
    fn test_u24_add() {
        let a = U24::new(0xAAAAAA).unwrap();
        let b = U24::new(0x555555).unwrap();
        let sum = a.add(b).unwrap();
        assert_eq!(sum.value(), 0xFFFFFF);
    }

    #[test]
    fn test_u24_add_overflow() {
        let a = U24::new(0xAAAAAB).unwrap();
        let b = U24::new(0x555555).unwrap();
        let sum = a.add(b);
        assert!(sum.is_none());
    }

    #[test]
    fn test_u24_sub() {
        let a = U24::new(0xAAAAAA).unwrap();
        let b = U24::new(0x555555).unwrap();
        let diff = a.sub(b).unwrap();
        assert_eq!(diff.value(), 0x555555);
    }

    #[test]
    fn test_u24_sub_underflow() {
        let a = U24::new(0x555555).unwrap();
        let b = U24::new(0xAAAAAA).unwrap();
        let diff = a.sub(b);
        assert!(diff.is_none());
    }

    #[test]
    fn test_u24_new_within_bounds() {
        let a = U24::new(0x123456).unwrap();
        assert_eq!(a.value(), 0x123456);
    }

    #[test]
    fn test_u24_new_out_of_bounds() {
        let a = U24::new(0x1000000);
        assert!(a.is_none());
    }

    #[test]
    fn test_u24_mul() {
        let a = U24::new(0x000100).unwrap();
        let b = U24::new(0x000010).unwrap();
        let product = a.mul(b).unwrap();
        assert_eq!(product.value(), 0x001000);
    }

    #[test]
    fn test_u24_mul_overflow() {
        let a = U24::new(0x010000).unwrap();
        let b = U24::new(0x010000).unwrap();
        let product = a.mul(b);
        assert!(product.is_none());
    }

    #[test]
    fn test_u24_div() {
        let a = U24::new(0x001000).unwrap();
        let b = U24::new(0x000010).unwrap();
        let quotient = a.div(b).unwrap();
        assert_eq!(quotient.value(), 0x000100);
    }

    #[test]
    fn test_u24_div_by_zero() {
        let a = U24::new(0x001000).unwrap();
        let b = U24::new(0x000000).unwrap();
        let quotient = a.div(b);
        assert!(quotient.is_none());
    }

    #[test]
    fn test_u24_eq() {
        let a = U24::new(0x000100).unwrap();
        let b = U24::new(0x000100).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_u24_neq() {
        let a = U24::new(0x000100).unwrap();
        let b = U24::new(0x000101).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_u24_cmp() {
        let a = U24::new(0x000100).unwrap();
        let b = U24::new(0x000101).unwrap();
        assert!(a < b);
        assert!(b > a);
    }
}
