use std::fs;
use std::path::Path;

use crate::error::{Result, TypsyError};

const HEAD_INJECT: &str = r#"
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="/style.css">
<link rel="icon" href="/favicon.ico">
"#;

const LIVE_RELOAD_SCRIPT: &str = r#"
<script>
(function() {
  const es = new EventSource('/__typsy_reload');
  es.onmessage = function(e) {
    if (e.data === 'reload') window.location.reload();
  };
  es.onerror = function() {
    setTimeout(function() { window.location.reload(); }, 1000);
  };
})();
</script>
"#;

/// Inject `<head>` meta tags (and optionally live-reload script) into an HTML file.
pub fn inject_head(html_file: &Path, dev_mode: bool) -> Result<()> {
    let content = fs::read_to_string(html_file).map_err(|e| TypsyError::Io {
        path: html_file.to_path_buf(),
        source: e,
    })?;

    let mut injection = HEAD_INJECT.to_string();
    if dev_mode {
        injection.push_str(LIVE_RELOAD_SCRIPT);
    }

    let new_content = if content.contains("<head>") {
        content.replacen("<head>", &format!("<head>{injection}"), 1)
    } else if content.contains("<HEAD>") {
        content.replacen("<HEAD>", &format!("<HEAD>{injection}"), 1)
    } else {
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head>{injection}</head><body>{content}</body></html>"
        )
    };

    fs::write(html_file, new_content).map_err(|e| TypsyError::Io {
        path: html_file.to_path_buf(),
        source: e,
    })?;

    Ok(())
}
