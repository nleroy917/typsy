use std::fs;
use std::path::Path;

use crate::error::{Result, TypsyError};

const CONTENT: &str = r#"
= Welcome to Typsy!
This is your first page. Edit `content/index.typ` to change this content.

== Adding More Pages
To add more pages, create new `.typ` files in the `content/` directory. For example, `content/about.typ` will be available at `/about.html`.

== Styling
You can edit the default styles in `static/style.css`. This file is served as-is, so you can use any CSS you like.

== Development Mode
When you run the dev server (`typsy dev`), it will automatically reload the page when you make changes to any `.typ` files. Just save your changes and see them reflected in the browser!
"#;

const STYLE: &str = r#"
body {
    font-family: system-ui, sans-serif;
    max-width: 42rem;
    margin: 3rem auto;
    padding: 0 1rem;
    line-height: 1.6;
    color: #333;
}
h1, h2, h3 {
    font-weight: 600;
    color: #111;
}
a {
    color: #0077cc;
    text-decoration: none;
}
a:hover {
    text-decoration: underline;
}
"#;

pub fn init_new_typsy_project(project_dir: &Path) -> Result<()> {
    // Create the project directory
    fs::create_dir_all(project_dir).map_err(|e| TypsyError::Io {
        path: project_dir.to_path_buf(),
        source: e,
    })?;

    // Create content directory and index.typ
    let content_dir = project_dir.join("content");
    fs::create_dir_all(&content_dir).map_err(|e| TypsyError::Io {
        path: content_dir.to_path_buf(),
        source: e,
    })?;

    // Create a default index.typ with welcome content
    let index_file = content_dir.join("index.typ");
    fs::write(&index_file, CONTENT).map_err(|e| TypsyError::Io {
        path: index_file.to_path_buf(),
        source: e,
    })?;

    // Create static directory and default style.css
    let static_dir = project_dir.join("static");
    fs::create_dir_all(&static_dir).map_err(|e| TypsyError::Io {
        path: static_dir.to_path_buf(),
        source: e,
    })?;

    // Create a default style.css with basic styles
    let style_file = static_dir.join("style.css");
    fs::write(&style_file, STYLE).map_err(|e| TypsyError::Io {
        path: style_file.to_path_buf(),
        source: e,
    })?;

    Ok(())
}