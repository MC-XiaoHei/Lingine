use indicatif::{ProgressBar, ProgressStyle};

pub fn create_progress_bar(len: u64, title: impl Into<String>) -> ProgressBar {
    ProgressBar::new(len)
        .with_message(title.into())
        .with_style(
        ProgressStyle::default_bar()
            .template(
                "{msg} ({pos}/{len})\n[{bar:40.cyan/blue}] {percent}% • {elapsed_precise} (ETA: {eta})",
            )
            .unwrap()
            .progress_chars("=>-"),
    )
}
