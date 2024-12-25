use indicatif::ProgressStyle;

pub fn spinner() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {spinner}{msg}")
        .unwrap()
        .tick_strings(&["  ", "🔴", "  "])
}
