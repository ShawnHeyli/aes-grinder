pub struct Debug {
    granularity: u8,
}

impl Debug {
    pub fn new (gr_given: u8) -> Self {
        Debug {
            granularity: gr_given,
        }
    }

    pub fn print (&self, msg: &String, gr: u8) {
        if self.granularity >= gr {
            println! ("{}", msg);
        }
    }

    pub fn apply_fct<T>(&self, f: fn(&T), module: &T, gr: u8) {
        if self.granularity >= gr {
            f(module);
        }
    }
}
