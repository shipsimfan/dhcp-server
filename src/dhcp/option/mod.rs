mod class;

pub use class::*;

pub struct DHCPOption {
    class: DHCPOptionClass,
    value: Vec<u8>,
}

#[derive(Debug)]
pub enum ParseOptionError {
    MissingClass,
    MissingLength,
    InvalidLength,
}

impl DHCPOption {
    pub fn new(class: DHCPOptionClass, value: Vec<u8>) -> Self {
        DHCPOption { class, value }
    }

    pub fn parse(slice: &[u8]) -> Result<Self, ParseOptionError> {
        if slice.len() == 0 {
            return Err(ParseOptionError::MissingClass);
        }

        let class = DHCPOptionClass::parse(slice[0]);

        if class == DHCPOptionClass::End {
            return Ok(DHCPOption {
                class,
                value: Vec::new(),
            });
        } else if slice.len() == 1 {
            return Err(ParseOptionError::MissingLength);
        }

        let length = slice[1] as usize;

        let mut value = Vec::new();
        for i in 0..length {
            if i + 2 >= slice.len() {
                return Err(ParseOptionError::InvalidLength);
            }

            value.push(slice[i + 2]);
        }

        Ok(DHCPOption { class, value })
    }

    pub fn class(&self) -> DHCPOptionClass {
        self.class
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_slice()
    }

    pub fn generate(&self) -> Vec<u8> {
        let mut vec = vec![self.class.generate()];

        if self.class == DHCPOptionClass::End {
            return vec;
        }

        vec.push(self.value.len() as u8);

        for value in &self.value {
            vec.push(*value);
        }

        vec
    }
}

impl std::fmt::Display for DHCPOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.class, self.value)
    }
}

impl std::error::Error for ParseOptionError {}

impl std::fmt::Display for ParseOptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ParseOptionError::MissingClass => "Missing class",
                ParseOptionError::MissingLength => "Missing length",
                ParseOptionError::InvalidLength => "Invalid length",
            }
        )
    }
}
