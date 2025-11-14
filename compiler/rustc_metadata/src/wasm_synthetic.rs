//! Synthetic metadata generation for WASM proc macros
//!
//! This module creates minimal metadata for WASM proc macro crates,
//! allowing them to be registered in the CStore and used like native proc macros.
//!
//! Instead of generating metadata from scratch (which requires complex encoding),
//! we use a pre-compiled template from a minimal proc-macro crate and adapt it
//! for the WASM proc macro.

use rustc_middle::ty::TyCtxt;
use rustc_session::cstore::CrateSource;
use rustc_session::search_paths::PathKind;
use rustc_span::symbol::Symbol;
use std::path::Path;

use crate::rmeta::*;
use crate::creader::Library;

/// Creates a synthetic `Library` for a WASM proc macro crate
///
/// This creates minimal metadata by using a pre-compiled template.
pub fn create_wasm_proc_macro_library<'tcx>(
    _tcx: TyCtxt<'tcx>,
    _crate_name: Symbol,
    wasm_path: &Path,
    _proc_macros: &[proc_macro::bridge::client::ProcMacro],
) -> Library {
    eprintln!("[WASM_SYNTHETIC] Loading template metadata for WASM proc macro");

    // Load the template metadata from a pre-compiled proc-macro crate
    // This template was created by compiling examine_proc_macro.rs
    let template_bytes = include_bytes!("../../../proc_macro_template.rmeta");

    eprintln!("[WASM_SYNTHETIC] Template size: {} bytes", template_bytes.len());

    // Patch the template to mark it as NOT a proc-macro crate
    // This prevents rustc from trying to load proc macros from the metadata,
    // since we're passing them directly via pre_loaded_proc_macros
    let metadata_vec = template_bytes.to_vec();

    // The CrateHeader.is_proc_macro_crate field is at a specific offset
    // We need to find it and set it to false (0x00)
    // Looking at the structure: it comes after triple, hash, and name in the CrateHeader
    // Since finding the exact offset is complex, we'll search for the pattern
    // For now, let's try a simple approach: the template has is_proc_macro_crate=true (0x01)
    // somewhere in the CrateRoot/CrateHeader. We'll patch it to false (0x00).

    // Actually, let's just use the template as-is for now and rely on pre_loaded_proc_macros
    // The key is that our modified register_crate checks pre_loaded first, so it won't
    // try to dlsym even if is_proc_macro_crate is true

    // Create the metadata blob
    use rustc_data_structures::owned_slice::slice_owned;
    use std::ops::Deref;

    let metadata_blob = match MetadataBlob::new(slice_owned(metadata_vec, Deref::deref)) {
        Ok(blob) => {
            eprintln!("[WASM_SYNTHETIC] Successfully created MetadataBlob from template");
            blob
        }
        Err(()) => {
            panic!("Failed to create MetadataBlob from template - template may be invalid");
        }
    };

    let source = CrateSource {
        dylib: Some((wasm_path.to_path_buf(), PathKind::All)),
        rlib: None,
        rmeta: None,
    };

    Library {
        source,
        metadata: metadata_blob,
    }
}
