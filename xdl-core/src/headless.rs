//! Headless mode.
//!
//! Interactive visualizations normally launch a viewer window. In headless
//! mode they write a self-contained HTML file into the working directory
//! instead. The daemon runs XDL this way: it has no display, and a window it
//! could not show would be a silent failure.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static HEADLESS: AtomicBool = AtomicBool::new(false);
static EXPORT_SEQ: AtomicUsize = AtomicUsize::new(0);

/// Turn headless mode on or off for this process.
pub fn set_headless(on: bool) {
    HEADLESS.store(on, Ordering::SeqCst);
}

/// True when visualizations must be written to files, not shown in windows.
pub fn is_headless() -> bool {
    HEADLESS.load(Ordering::SeqCst)
}

/// Next export file name, `<stem>_<n>.<ext>`. `n` counts from 1 per process,
/// so a second export in the same run never overwrites the first.
pub fn next_export_name(stem: &str, ext: &str) -> String {
    let n = EXPORT_SEQ.fetch_add(1, Ordering::SeqCst) + 1;
    format!("{stem}_{n}.{ext}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_names_are_sequential_and_distinct() {
        let a = next_export_name("t", "html");
        let b = next_export_name("t", "html");
        assert_ne!(a, b);
        assert!(a.starts_with("t_") && a.ends_with(".html"));
    }

    #[test]
    fn headless_flag_round_trips() {
        set_headless(true);
        assert!(is_headless());
        set_headless(false);
        assert!(!is_headless());
    }
}
