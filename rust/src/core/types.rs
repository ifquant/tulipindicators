pub type Real = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorCategory {
    Overlay,
    Indicator,
    Math,
    Simple,
    Comparative,
}
