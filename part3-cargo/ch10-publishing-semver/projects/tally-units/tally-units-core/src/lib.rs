//! 길이 단위를 미터 기준으로 바꿉니다.

/// 지원하는 길이 단위
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Unit {
    Millimeter,
    Centimeter,
    Meter,
    Kilometer,
}

impl Unit {
    /// 단위 기호(`"mm"` 등)로 단위를 찾습니다.
    ///
    /// ```
    /// use tally_units_core::Unit;
    /// assert_eq!(Unit::parse("km"), Some(Unit::Kilometer));
    /// assert_eq!(Unit::parse("mi"), None);
    /// ```
    pub fn parse(symbol: &str) -> Option<Unit> {
        match symbol {
            "mm" => Some(Unit::Millimeter),
            "cm" => Some(Unit::Centimeter),
            "m" => Some(Unit::Meter),
            "km" => Some(Unit::Kilometer),
            _ => None,
        }
    }

    /// 1 단위가 몇 미터인지
    pub fn meters(self) -> f64 {
        match self {
            Unit::Millimeter => 0.001,
            Unit::Centimeter => 0.01,
            Unit::Meter => 1.0,
            Unit::Kilometer => 1000.0,
        }
    }
}

/// `value`를 `from` 단위에서 `to` 단위로 바꿉니다.
///
/// ```
/// use tally_units_core::{convert, Unit};
/// assert_eq!(convert(2.5, Unit::Kilometer, Unit::Meter), 2500.0);
/// ```
pub fn convert(value: f64, from: Unit, to: Unit) -> f64 {
    value * from.meters() / to.meters()
}
