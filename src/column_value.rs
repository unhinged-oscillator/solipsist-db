use std::time::SystemTime;

pub enum ColumnValue {
    Integer(i64),
    Float(f64),
    String(String),
    Blob(Vec<u8>),
    Timestamp(SystemTime),
}

impl ColumnValue {
    // fn insert(&mut self, value: impl Into<ColumnValue>) {
    //     match self {
    //         ColumnValue::Integer(val) => *val = value.into().integer().unwrap(),
    //         ColumnValue::Float(val) => *val = value.into().float().unwrap(),
    //         ColumnValue::String(val) => *val = value.into().string().unwrap(),
    //         ColumnValue::Blob(blob) =>
    //         ColumnValue::Timestamp(val) => val.insert(value.into().time_series().unwrap()),
    //     }
    // }

    fn integer(self) -> Option<i64> {
        match self {
            ColumnValue::Integer(val) => Some(val),
            _ => None,
        }
    }

    fn float(self) -> Option<f64> {
        match self {
            ColumnValue::Float(val) => Some(val),
            _ => None,
        }
    }

    fn string(self) -> Option<String> {
        match self {
            ColumnValue::String(val) => Some(val),
            _ => None,
        }
    }

    fn timestamp(self) -> Option<SystemTime> {
        match self {
            ColumnValue::Timestamp(val) => Some(val),
            _ => None,
        }
    }
}

macro_rules! column_from_raw {
    ($t:ty, $member:ident) => {
        impl From<$t> for ColumnValue {
            fn from(value: $t) -> Self {
                ColumnValue::$member(value.into())
            }
        }
    };
}

column_from_raw!(i64, Integer);
column_from_raw!(i32, Integer);
column_from_raw!(f32, Float);
column_from_raw!(f64, Float);
column_from_raw!(String, String);
column_from_raw!(&str, String);
column_from_raw!(Vec<u8>, Blob);
column_from_raw!(&[u8], Blob);
// column_from_raw!(, Blob);
