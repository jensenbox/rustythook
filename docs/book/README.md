# RustyHook Documentation

This directory contains the source files for the RustyHook documentation website, built with [mdBook](https://rust-lang.github.io/mdBook/).

## Local Development

### Prerequisites

- [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html) installed on your system

```sh
cargo install mdbook
```

### Building the Documentation

To build the documentation:

```sh
cd docs/book
mdbook build
```

This will generate the HTML files in the `book` directory.

### Previewing the Documentation

To preview the documentation with live reloading:

```sh
cd docs/book
mdbook serve --open
```

This will start a local web server and open the documentation in your default web browser. The documentation will automatically reload when you make changes to the source files.

## Documentation Structure

- `src/`: Contains the Markdown source files
  - `SUMMARY.md`: Defines the table of contents
  - `*.md`: Individual documentation pages
- `book.toml`: Configuration file for mdBook
- `theme/`: Custom styling and JavaScript (optional)

## Contributing to the Documentation

1. Make your changes to the Markdown files in the `src/` directory
2. Preview your changes locally using `mdbook serve --open`
3. Commit your changes and create a pull request

## Deployment

The documentation is automatically built and deployed to GitHub Pages when changes are pushed to the main branch. The GitHub Actions workflow is defined in `.github/workflows/docs.yml`.

## Additional Resources

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [Markdown Guide](https://www.markdownguide.org/)