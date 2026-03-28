//! Lightweight scope and block timing.
//!
//! All output goes to stderr so stdout stays clean for program output.
//! Use `2> file.txt` to capture timing separately.
//!
//! # Examples
//!
//! **Drop guard** — times the enclosing scope (function, block, loop body):
//!
//! ```ignore
//! fn main() {
//!     timed!("fn main");
//!     // ... entire function timed, prints on drop ...
//! }
//! ```
//!
//! **Block form** — times a section, variables escape into caller scope:
//!
//! ```ignore
//! timed!("parsing args", {
//!     let cli = Cli::parse();
//! });
//! // cli is visible here
//! ```

pub struct Timed {
    label: &'static str,
    start: std::time::Instant,
    silent: bool,
}

impl Timed {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: std::time::Instant::now(),
            silent: false,
        }
    }

    pub fn get(&mut self) -> std::time::Duration {
        self.silent = true;
        self.start.elapsed()
    }

    pub fn eprintln(&mut self) {
        let elapsed = self.get();
        eprintln!("{} {:?}", self.label, elapsed);
    }
}

impl Drop for Timed {
    fn drop(&mut self) {
        if !self.silent {
            self.eprintln();
        }
    }
}

macro_rules! timed {
    ($label:expr) => {
        let _timer = $crate::timed::Timed::new($label);
    };
    ($label:expr, { $($body:tt)* }) => {
        let mut _timer = $crate::timed::Timed::new($label);
        $($body)*
        _timer.eprintln();
    };
}

pub(crate) use timed;
