pub enum Version {
    V0,
    V1,
    V2,
}

impl Version {
    pub fn from_str(s: &str) -> Option<Version> {
        match s {
            "V0" => Some(Version::V0),
            "V1" => Some(Version::V1),
            "V2" => Some(Version::V2),
            _ => None, // Any other string results in None
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
