# mdbook-chapter-path

![[CI status](https://ci.younata.com/teams/main/pipelines/knowledge/jobs/mdbook-chapter-path/)](https://ci.younata.com/api/v1/pipelines/knowledge/jobs/mdbook-chapter-path/badge)

[mdBook](https://github.com/rust-lang/mdbook) preprocessor for providing the path to a chapter based on its name.

## Getting Started

First, install the `mdbook-chapter-path` crate

```
cargo install mdbook-chapter-path
```

Then, add the following line to your `book.toml` file:

```toml
[preprocessor.chapter-path]
```

Once done, you can now use `{{#path_for $NAME_OF_CHAPTER}}` to insert the path (relative to `SUMMARY.md`) to that chapter.

E.g. If you have a chapter named "Whatever" located at "foo/whatever.md", the markdown `{{#path_for Whatever}}` will replace that with `/foo/whatever.md`.

This even works for anchor links, e.g. `{{#path_for Whatever#an_anchor}}` will replace that with `/foo/whatever.md#an_anchor`.

This is useful because it means the link will survive moving files around.

## Configuration

`mdbook-chapter-path` will use whatever is listed in `output.html.site-url` as the base path for all links, defaulting to '/'.

Additionally, you can enable `strict` mode. If this is set to `true`, `mdbook-chapter-path` will exit with an error whenever it encounters a duplicate chapter name. This defaults to `false`, meaning that duplicate chapter names are allowed.

⚠️ Without strict mode enabled, if you have multiple chapters with the same name (case-insensitive), then `mdbook-chapter-path` will provide the path for whichever chapter is listed last in the book.

```toml
[preprocessor.chapter-path]
strict = true
```