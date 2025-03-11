use std::fmt::Debug;
use proc_macro2::Span;
use proc_macro2::TokenStream;

// =============
// === Level ===
// =============

#[derive(Clone, Copy, Debug)]
pub(crate) enum Level {
    Warning,
    Error,
}

impl Level {
    pub const WARNING_PREFIX: &'static str = "[WARNING]";
    pub const ERROR_PREFIX: &'static str = "[ERROR]";

    #[cfg(not(nightly))]
    fn prefix(&self) -> &str {
        match self {
            Level::Warning => Self::WARNING_PREFIX,
            Level::Error => Self::ERROR_PREFIX,
        }
    }
}

#[cfg(nightly)]
impl From<Level> for proc_macro::Level {
    fn from(level: Level) -> Self {
        match level {
            Level::Warning => proc_macro::Level::Warning,
            Level::Error => proc_macro::Level::Error,
        }
    }
}

pub(crate) fn print(level: Level, message: &str) {
    #[cfg(nightly)] {
        let span = proc_macro::Span::call_site();
        proc_macro::Diagnostic::spanned(span, level.into(), message).emit();
    }
    #[cfg(not(nightly))] {
        println!("{} {message}", level.prefix());
    }
}

macro_rules! debug         { ($($ts:tt)*) => { if DEBUG { println!( $($ts)* )}  }; }
macro_rules! print_warning { ($($ts:tt)*) => { print (Level::Warning, &format!( $($ts)* )); }; }
macro_rules! print_error   { ($($ts:tt)*) => { print (Level::Error,   &format!( $($ts)* )); }; }
pub(crate) use debug;
pub(crate) use print_warning;
pub(crate) use print_error;

// ==============
// === Errors ===
// ==============

pub(crate) type Result<T=(), E=Issue> = std::result::Result<T, E>;

pub(crate) struct Issue {
    pub level: Level,
    pub span: Option<Span>,
    pub message: String,
    pub context: Option<Box<Issue>>,
}

impl Issue {
    pub fn msg(level: Level, span: Option<Span>, message: String) -> Self {
        Self { level, span, message, context: None }
    }

    pub fn context(mut self, f: impl FnOnce() -> Issue) -> Self {
        self.context = Some(Box::new(f()));
        self
    }

    pub fn message_with_cause(&self) -> String {
        match &self.context {
            None => self.message.clone(),
            Some(context) =>
                format!("{}\nCaused by: {}", self.message, context.message_with_cause()),
        }
    }

    #[cfg(nightly)]
    pub fn emit(&self) {
        // SAFETY: This unwrap is safe in proc macros.
        let span = self.span.unwrap_or_else(Span::call_site).unwrap();
        let level = self.level.into();
        let message = self.message_with_cause();
        proc_macro::Diagnostic::spanned(span, level, message).emit();
    }

    // This is a hack to make compile errors with spans on stable.
    // Source: https://stackoverflow.com/questions/54392702/how-to-report-errors-in-a-procedural-macro-using-the-quote-macro
    pub fn compile_error(&self) -> TokenStream {
        let span = self.span.unwrap_or_else(Span::call_site);
        let message = self.message_with_cause();
        quote::quote_spanned! { span => compile_error!(#message) }
    }
}

impl<E: Debug> From<E> for Issue {
    fn from(e: E) -> Self {
        Self::msg(Level::Error, None, format!("{e:?}"))
    }
}

macro_rules! issue   {
    ($l:expr,          $s:literal           ) => { Issue::msg($l, None,     format!($s)) };
    ($l:expr,          $s:expr              ) => { Issue::msg($l, None,     format!("{}", $s)) };
    ($l:expr,          $s:literal, $($t:tt)*) => { Issue::msg($l, None,     format!($s, $($t)*)) };
    ($l:expr, $e:expr, $s:literal           ) => { Issue::msg($l, Some($e), format!($s)) };
    ($l:expr, $e:expr, $s:expr              ) => { Issue::msg($l, Some($e), format!("{}", $s)) };
    ($l:expr, $e:expr, $s:literal, $($t:tt)*) => { Issue::msg($l, Some($e), format!($s, $($t)*)) };
    ($l:expr, $e:expr,             $($t:tt)*) => { Issue::msg($l, Some($e), format!($($t)*)) };
    ($l:expr,                      $($t:tt)*) => { Issue::msg($l, None,     format!($($t)*)) };
}

macro_rules! error   { ($($ts:tt)*) => { issue! { Level::Error,   $($ts)* }}; }
macro_rules! warning { ($($ts:tt)*) => { issue! { Level::Warning, $($ts)* }}; }
macro_rules! err     { ($($ts:tt)*) => { Err(error!($($ts)*)) }; }
pub(crate) use issue;
pub(crate) use error;
pub(crate) use warning;
pub(crate) use err;

// ===============
// === Context ===
// ===============

pub(crate) trait Context<T, I> {
    fn context(self, issue: I) -> Result<T>;
}

impl<T, I> Context<T, I> for Result<T, Issue> where
I: FnOnce() -> Issue {
    fn context(self, issue: I) -> Result<T> {
        self.map_err(|e| e.context(issue))
    }
}

impl<T> Context<T, &'static str> for Result<T, Issue> {
    fn context(self, issue: &'static str) -> Result<T> {
        self.context(|| error!("{}", issue))
    }
}

impl<T, E: Debug, I> Context<T, I> for Result<T, E> where
I: FnOnce() -> Issue {
    fn context(self, issue: I) -> Result<T> {
        self.map_err(|e| Issue::from(e)).context(issue)
    }
}

impl<T, E: Debug> Context<T, &'static str> for Result<T, E> {
    fn context(self, issue: &'static str) -> Result<T> {
        self.context(|| error!("{}", issue))
    }
}

impl<T, I> Context<T, I> for Option<T> where
I: FnOnce() -> Issue {
    fn context(self, issue: I) -> Result<T> {
        self.ok_or_else(issue)
    }
}

impl<T> Context<T, &'static str> for Option<T> {
    fn context(self, issue: &'static str) -> Result<T> {
        self.context(|| error!("{}", issue))
    }
}

// ==============
// === Unwrap ===
// ==============

pub(crate) trait Unwrap {
    fn unwrap_or_compile_error(self) -> TokenStream;
}

impl Unwrap for Result<TokenStream, Issue> {
    fn unwrap_or_compile_error(self) -> TokenStream {
        self.unwrap_or_else(|e| e.compile_error())
    }
}
