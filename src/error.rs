use std::path::PathBuf;

use thiserror::Error;
use typst::diag::SourceDiagnostic;

#[derive(Error, Debug)]
pub enum TypsyError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("typst compilation failed for {path}:\n{}", format_diagnostics(diagnostics))]
    TypstCompile {
        path: PathBuf,
        diagnostics: Vec<String>,
    },

    #[error("HTML export failed for {path}:\n{}", format_diagnostics(diagnostics))]
    HtmlExport {
        path: PathBuf,
        diagnostics: Vec<String>,
    },

    #[error("no content/ directory found")]
    NoContentDir,
}

pub type Result<T> = std::result::Result<T, TypsyError>;

fn format_diagnostics(diagnostics: &[String]) -> String {
    diagnostics
        .iter()
        .map(|d| format!("  - {d}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert typst SourceDiagnostics into plain strings for storage in error variants.
pub fn diagnostics_to_strings(diags: &[SourceDiagnostic]) -> Vec<String> {
    diags
        .iter()
        .map(|d| {
            let mut msg = d.message.to_string();
            for hint in &d.hints {
                msg.push_str(&format!(" (hint: {hint})"));
            }
            msg
        })
        .collect()
}