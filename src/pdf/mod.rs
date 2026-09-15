//! PDF parsing and manipulation, built on `lopdf`.
//!
//! This module will own the document model: loading PDFs, page metadata,
//! and low-level object manipulation. It is wired in a later phase.

// `lopdf` is a dependency from the start so the build graph is settled;
// the module body lands together with the first PDF-backed operation (Info).