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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_returns_expected_values() {
        assert_eq!(Mode::Komunitas.as_str(), "komunitas");
        assert_eq!(Mode::CatatanKomunitas.as_str(), "catatan_komunitas");
        assert_eq!(Mode::CatatanSaksi.as_str(), "catatan_saksi");
        assert_eq!(Mode::Siaga.as_str(), "siaga");
    }

    #[test]
    fn serde_roundtrips_known_values() {
        let encoded = r#""catatan_saksi""#;
        let parsed: Mode = serde_json::from_str(encoded).expect("parse known mode");
        assert_eq!(parsed, Mode::CatatanSaksi);
        let rendered = serde_json::to_string(&parsed).expect("serialize known mode");
        assert_eq!(rendered, encoded);
    }

    #[test]
    fn serde_rejects_invalid_mode() {
        let parsed = serde_json::from_str::<Mode>(r#""komunitas_x""#);
        assert!(parsed.is_err());
    }
}
