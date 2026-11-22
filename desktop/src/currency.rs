//! Supported currencies (same list as the original app).

pub struct Currency {
    pub code: &'static str,
    pub symbol: &'static str,
    pub label: &'static str,
}

pub const CURRENCIES: &[Currency] = &[
    Currency { code: "USD", symbol: "$", label: "USD - US Dollar ($)" },
    Currency { code: "EUR", symbol: "€", label: "EUR - Euro (€)" },
    Currency { code: "GBP", symbol: "£", label: "GBP - British Pound (£)" },
    Currency { code: "CAD", symbol: "$", label: "CAD - Canadian Dollar ($)" },
    Currency { code: "AUD", symbol: "$", label: "AUD - Australian Dollar ($)" },
    Currency { code: "BDT", symbol: "৳", label: "BDT - Bangladeshi Taka (৳)" },
    Currency { code: "INR", symbol: "₹", label: "INR - Indian Rupee (₹)" },
    Currency { code: "JPY", symbol: "¥", label: "JPY - Japanese Yen (¥)" },
    Currency { code: "CNY", symbol: "¥", label: "CNY - Chinese Yuan (¥)" },
    Currency { code: "SGD", symbol: "$", label: "SGD - Singapore Dollar ($)" },
    Currency { code: "AED", symbol: "AED", label: "AED - UAE Dirham (AED)" },
    Currency { code: "SAR", symbol: "SAR", label: "SAR - Saudi Riyal (SAR)" },
    Currency { code: "CHF", symbol: "CHF", label: "CHF - Swiss Franc (CHF)" },
];

pub fn symbol_for(code: &str) -> Option<&'static str> {
    CURRENCIES.iter().find(|c| c.code == code).map(|c| c.symbol)
}
