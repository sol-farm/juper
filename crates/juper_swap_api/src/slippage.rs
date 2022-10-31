use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Slippage {
    Zero,               // NA, but more specifically an empty string
    ZeroBip,            // 0%
    OneBip,             // 0.01%
    TwoBip,             // 0.02%
    FiveBip,            // 0.05%
    SevenFiveBip,       // 0.075%
    TenBip,             // 0.10%
    FifteenBip,         // 0.15%
    TwentyBip,          // 0.20%
    FiftyBip,           // 0.50%
    SeventyFiveBip,     // 0.75%
    OneHundredFiftyBip, // 1.50%
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FeeBps {
    Zero,               // 0%, but more specifically an empty string
    OneBip,             // 0.01%
    TenBip,             // 0.10%
    FifteenBip,         // 0.15%
    TwentyBip,          // 0.20%
    FiftyBip,           // 0.50%
    SeventyFiveBip,     // 0.75%
    OneHundredFiftyBip, // 1.50%
}

impl FeeBps {
    pub fn value(&self) -> &str {
        match self {
            Self::Zero => {
                const VALUE: &str = "";
                VALUE
            }
            Self::OneBip => {
                const VALUE: &str = "&feesBps=0.01";
                VALUE
            }
            Self::TenBip => {
                const VALUE: &str = "&feesBps=0.10";
                VALUE
            }
            Self::FifteenBip => {
                const VALUE: &str = "&feesBps=0.15";
                VALUE
            }
            Self::TwentyBip => {
                const VALUE: &str = "&feesBps=0.20";
                VALUE
            }
            Self::FiftyBip => {
                const VALUE: &str = "&feesBps=0.50";
                VALUE
            }
            Self::SeventyFiveBip => {
                const VALUE: &str = "&feesBps=0.75";
                VALUE
            }
            Self::OneHundredFiftyBip => {
                const VALUE: &str = "&feesBps=1.50";
                VALUE
            }
        }
    }
}

impl Slippage {
    pub fn value(&self) -> &str {
        match self {
            Self::Zero => {
                const VALUE: &str = "";
                VALUE
            }
            Self::ZeroBip => {
                const VALUE: &str = "&slippage=0.000001";
                VALUE
            }
            Self::OneBip => {
                const VALUE: &str = "&slippage=0.01";
                VALUE
            }
            Self::TwoBip => {
                const VALUE: &str = "&slippage=0.02";
                VALUE
            }
            Self::FiveBip => {
                const VALUE: &str = "&slippage=0.05";
                VALUE
            }
            Self::SevenFiveBip => {
                const VALUE: &str = "&slippage=0.075";
                VALUE
            }
            Self::TenBip => {
                const VALUE: &str = "&slippage=0.10";
                VALUE
            }
            Self::FifteenBip => {
                const VALUE: &str = "&slippage=0.15";
                VALUE
            }
            Self::TwentyBip => {
                const VALUE: &str = "&slippage=0.20";
                VALUE
            }
            Self::FiftyBip => {
                const VALUE: &str = "&slippage=0.50";
                VALUE
            }
            Self::SeventyFiveBip => {
                const VALUE: &str = "&slippage=0.75";
                VALUE
            }
            Self::OneHundredFiftyBip => {
                const VALUE: &str = "&slippage=1.50";
                VALUE
            }
        }
    }
}

impl Default for Slippage {
    fn default() -> Self {
        Self::TenBip
    }
}
impl Default for FeeBps {
    fn default() -> Self {
        Self::Zero
    }
}
