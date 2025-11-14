// Wrapper library that re-exports the Template derive macro

// Import the proc macro crate
extern crate template_proc_macro;

// Re-export the Template derive macro
pub use template_proc_macro::Template;
