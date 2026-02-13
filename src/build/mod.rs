pub mod fs;
pub mod inject;

use std::path::{Path, PathBuf};

use tracing::{error, info, warn};
use walkdir::WalkDir;

use crate::error::{self, TypsyError};
use crate::render::ContentRenderer;

/// Result of a full site build.
pub struct BuildReport {
    pub successes: Vec<PathBuf>,
    pub failures: Vec<TypsyError>,
}

/// Walk up from the current directory to find a directory containing `content/`.
pub fn find_root() -> error::Result<PathBuf> {
    let mut dir = std::env::current_dir().map_err(|e| TypsyError::Io {
        path: PathBuf::from("."),
        source: e,
    })?;
    loop {
        if dir.join("content").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(TypsyError::NoContentDir);
        }
    }
}

/// Build the entire site from `content/` into `out/`.
///
/// When `dev_mode` is true, a live-reload script is injected into HTML pages.
/// Returns a `BuildReport` with successes and failures instead of panicking.
pub fn build(root: &Path, dev_mode: bool) -> BuildReport {
    info!("building site...");

    let content_dir = root.join("content");
    let static_dir = root.join("static");
    let out_dir = root.join("out");

    // Clean output directory
    if out_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&out_dir) {
            return BuildReport {
                successes: vec![],
                failures: vec![TypsyError::Io {
                    path: out_dir,
                    source: e,
                }],
            };
        }
    }
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        return BuildReport {
            successes: vec![],
            failures: vec![TypsyError::Io {
                path: out_dir,
                source: e,
            }],
        };
    }

    // Copy static files
    if static_dir.exists() {
        if let Err(e) = fs::copy_dir_recursive(&static_dir, &out_dir) {
            error!("{e}");
        } else {
            info!("copied static files");
        }
    }

    // Find all .typ files
    let typ_files: Vec<PathBuf> = WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "typ"))
        .map(|e| e.path().to_path_buf())
        .collect();

    if typ_files.is_empty() {
        info!("no .typ files found in content/");
        return BuildReport {
            successes: vec![],
            failures: vec![],
        };
    }

    let mut report = BuildReport {
        successes: vec![],
        failures: vec![],
    };

    for typ_file in &typ_files {
        let rel_path = typ_file.strip_prefix(&content_dir).unwrap();

        // _index.typ -> index.html, otherwise foo.typ -> foo.html
        let out_path = if rel_path.file_name().unwrap() == "_index.typ" {
            out_dir
                .join(rel_path.parent().unwrap_or(Path::new("")))
                .join("index.html")
        } else {
            out_dir.join(rel_path).with_extension("html")
        };

        let out_rel = out_path.strip_prefix(&out_dir).unwrap();
        info!("  {} -> {}", rel_path.display(), out_rel.display());

        match compile_file(root, typ_file, &out_path) {
            Ok(()) => {
                if let Err(e) = inject::inject_head(&out_path, dev_mode) {
                    error!("{e}");
                    report.failures.push(e);
                } else {
                    report.successes.push(out_path);
                }
            }
            Err(e) => {
                error!("{e}");
                report.failures.push(e);
            }
        }
    }

    if report.failures.is_empty() {
        info!("done! output in out/");
    } else {
        warn!(
            "build completed with {} error(s)",
            report.failures.len()
        );
    }

    report
}

/// Compile a single `.typ` file to HTML and write it to `output`.
fn compile_file(root: &Path, input: &Path, output: &Path) -> error::Result<()> {
    // Ensure output directory exists
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| TypsyError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let input_content = std::fs::read_to_string(input).map_err(|e| TypsyError::Io {
        path: input.to_path_buf(),
        source: e,
    })?;

    let renderer = ContentRenderer::new(root.to_path_buf(), input_content);
    let warned = typst::compile::<typst_html::HtmlDocument>(&renderer);

    // Log any warnings (suppress the known typst-html development notice)
    for w in &warned.warnings {
        if w.message.contains("html export is under active development") {
            continue;
        }
        if w.message.contains("was ignored during HTML export") {
            continue;
        }
        warn!("{}: {}", input.display(), w.message);
    }

    let document = warned.output.map_err(|diags| TypsyError::TypstCompile {
        path: input.to_path_buf(),
        diagnostics: error::diagnostics_to_strings(&diags),
    })?;

    let html = typst_html::html(&document).map_err(|diags| TypsyError::HtmlExport {
        path: input.to_path_buf(),
        diagnostics: error::diagnostics_to_strings(&diags),
    })?;

    std::fs::write(output, html).map_err(|e| TypsyError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;

    Ok(())
}
