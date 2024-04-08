use core::ops::Add;
use std::{fmt::Display, ops::Mul};

#[derive(Debug, Clone, Copy)]
pub struct Number {
    value: u8,
    poly: u16,
}

impl Number {
    pub fn new(value: u8, poly: u16) -> Self {
        Number { value, poly }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
    
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub trait Invertible {
    /// multiplicative inverse of b(x), denoted b-1(x)
    fn invert(&self) -> Self;
}

impl Add for Number {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {value: (self.value ^ other.value), poly: self.poly}
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, other: Self) -> Number {
        let mut result = 0;
        let mut a = self.value;
        let mut b = other.value;
        let mut tmp:u8; 

        while a > 0 {
            if a % 2 == 1 {
                result = result ^ b;
            }
            tmp = b & 0x80;
            b = b << 1;
            if tmp != 0 {
                b = b ^ (self.poly as u8);
            }
            a = a >> 1;
        }

        Self {value: result, poly: self.poly}
    }
}

impl Invertible for Number {
    fn invert(&self) -> Self {
        // Extended Euclidean Algorithm
        let mut t = Number {
            value: 0,
            poly: self.poly,
        };
        let mut newt = Number {
            value: 1,
            poly: self.poly,
        };
        let mut r = self.clone();
        let mut newr = Number {
            value: (self.poly as u8),
            poly: self.poly,
        };

        while newr.value != 0 {
            let quotient = r.value / newr.value;
            let temp = newr.clone();
            newr = Number {
                value: r.value - quotient * newr.value,
                poly: self.poly,
            };
            r = temp;
            let temp = newt.clone();
            newt = Number {
                value: t.value - quotient * newt.value,
                poly: self.poly,
            };
            t = temp;
        }

        if r.value > 1 {
            panic!("{} is not invertible", self.value);
        }

        if t.value < 0 {
            t.value += self.poly as u8;
        }

        t
    }
}

impl From<u8> for Number {
    fn from(value: u8) -> Self {
        Number {
            value,
            poly: 0x11b,
        }
    }
}

