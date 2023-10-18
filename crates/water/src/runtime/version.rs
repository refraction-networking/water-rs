use std::fmt;
use std::str::FromStr;


pub enum Version {
    V0,
    V1,
    V2,
}

impl Version {
    pub fn parse(s: &str) -> Option<Version> {
        match Version::from_str(s) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            Version::V0 => "V0",
            Version::V1 => "V1",
            Version::V2 => "V2",
        }
    }
}


impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Version, ()> {
        match s {
            "V0" => Ok(Version::V0),
            "V1" => Ok(Version::V1),
            "V2" => Ok(Version::V2),
            _ => Err(()),
        }
    }
}


impl From<&Version> for &'static str {
    fn from(v: &Version) -> &'static str {
        match v {
            Version::V0 => "V0",
            Version::V1 => "V1",
            Version::V2 => "V2",
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.into())
    }
}