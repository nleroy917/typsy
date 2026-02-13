# typsy
typsy is a static-site generator for the typst typesetting system. It takes typst files as input and produces a static website as output, injecting any necessary HTML, CSS, and JavaScript to make the site functional.

This is very experimental and for my personal use, so expect things to break and be very rough around the edges. If you want to contribute, please do!

## Installation
Currently, the only way to install `typsy` is to build it from source. You can do this with Cargo:

```bash
cargo install --git https://github.com/nleroy917/typsy.git
```

## Quickstart

To create a new typsy site, run the following command in your terminal:

```bash
typsy new my-site
```

This will create a new directory called `my-site` with the following structure:

```
my-site/
├── content/
│   └── index.typst
├── static/
|   └── styles.css
```

The `content` directory is where you will put your typst files. The `static` directory is where you can put any static assets (like CSS, JavaScript, images, etc.) that you want to include in your site.

To build your site, run the following command in your terminal:

```bash
typsy build my-site
```

This will generate a static website in the `my-site/dist` directory. You can then serve this directory with any static file server (like `python -m http.server` or `serve`).

## Development
To start a development server that watches for changes and rebuilds the site automatically, run the following command:

```bash
typsy dev
```