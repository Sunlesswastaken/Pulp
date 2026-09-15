//! PDF compression backends.
//!
//! The primary backend shells out to Ghostscript (`gs`), which produces
//! the strongest size reductions without reimplementing a codec. A
//! secondary path may recompress page streams in-process with `lopdf`.
//!
//! Backends land in a later phase.

// qpdf is also an option for a lighter "just recompress streams" pass; it
// is auto-detected at runtime if present.