use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Komunitas,
    CatatanKomunitas,
    CatatanSaksi,
    Siaga,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Komunitas => "komunitas",
            Self::CatatanKomunitas => "catatan_komunitas",
            Self::CatatanSaksi => "catatan_saksi",
            Self::Siaga => "siaga",
        }
    }
}
