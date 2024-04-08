struct Number {
    value: u8,
    poly: &u16,
}

trait Invertible {
    /// multiplicative inverse of b(x), denoted b-1(x)
    fn invert(&self) -> Self;
}

impl Add for Number {
    type Output = Self;

    fn add(&self, &other: Self) -> Self {
        Self {value: (a ^ b), poly}
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(&self, other: Self) -> u8 {
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
            if(tmp != 0){
                b = b ^ poly;
            }
            a = a >> 1;
        }

        Self {value: result, poly}
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
            value: self.poly,
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
            t.value += self.poly;
        }

        t
    }
}

