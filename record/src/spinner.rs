use indicatif::ProgressStyle;

pub fn spinner() -> ProgressStyle {
    ProgressStyle::with_template("{spinner} Recording [{elapsed:>6}] {msg}")
        .unwrap()
        .tick_strings(&["  ", "🔴", "  "])
}
