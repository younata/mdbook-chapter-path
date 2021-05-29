use std::collections::HashMap;
use std::path::PathBuf;

use regex::{Regex, Captures};

use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub struct PathProcessor;

#[derive(Debug, Eq, PartialEq)]
pub enum ProcessorError {
    // Tried to provide path to the given chapter, but couldn't find one.
    ChapterNotFound(String),
    // Duplicate chapter names found. Only an issue when strict mode is on.
    DuplicateChapterNames(String)
}

struct FileLink<'a> {
    name: &'a str,
    anchor: Option<&'a str>
}

struct PathProcessorOptions {
    site_path: String,
    strict_mode: bool
}

impl FileLink<'_> {
    fn from_string(string: &str) -> FileLink {
        let splitted: Vec<&str> = string.split("#").collect();

        if splitted.len() > 2 {
            panic!("Invalid link parsed: Multiple '#'s detected for {}", string);
        }
        let name = splitted[0];
        let mut anchor: Option<&str> = None;
        if splitted.len() == 2 {
            anchor = Some(splitted[1]);
        }

        FileLink { name, anchor }
    }
}

impl Preprocessor for PathProcessor {
    fn name(&self) -> &str { "chapter-path" }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let options = self.process_options(ctx);

        let known_chapters = self.chapter_names(&book, &options).unwrap();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = self.process_chapter(&chapter.content, &known_chapters, &options).unwrap();
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool { renderer == "html" }
}

impl PathProcessor {
    fn process_options(&self, ctx: &PreprocessorContext) -> PathProcessorOptions {
        // process site_path
        let mut site_path: String = "/".to_string();
        if let Some(config) = ctx.config.get("output.html") {
            if let Some(toml::value::Value::String(value)) = config.get("site-url") {
                site_path = value.to_string();
            }
        }

        if site_path.ends_with("/") == false {
            site_path.push_str("/");
        }

        let mut strict_mode = false;
        if let Some(config) = ctx.config.get_preprocessor("chapter-path") {
            if let Some(toml::value::Value::Boolean(value)) = config.get("strict") {
                strict_mode = *value;
            }
        }

        PathProcessorOptions {
            site_path,
            strict_mode
        }
    }

    fn chapter_names(&self, book: &Book, options: &PathProcessorOptions) -> Result<HashMap<String, PathBuf>, ProcessorError>{
        let mut mapping: HashMap<String, PathBuf> = HashMap::new();

        for item in book.iter() {
            if let BookItem::Chapter(chapter) = item {
                if let Option::Some(path) = &chapter.path {
                    if let Some(existing_path) = mapping.get(&chapter.name.to_lowercase()) {
                        if options.strict_mode {
                            return Err(ProcessorError::DuplicateChapterNames(chapter.name.to_lowercase()));
                        } else {
                            eprintln!("Warning: Found duplicate chapter name {} at {} (existing chapter at {})", chapter.name, path.to_str().unwrap(), existing_path.to_str().unwrap());
                        }
                    }
                    mapping.insert(chapter.name.to_lowercase(), path.to_path_buf());
                }
            }
        };
        Ok(mapping)
    }

    fn process_chapter(&self, content: &str, chapter_names: &HashMap<String, PathBuf>, options: &PathProcessorOptions) -> Result<String, ProcessorError> {
        let regex = Regex::new(r"\{\{#path_for (?P<file>.+?)}}").unwrap();

        let captures: Vec<Captures> = regex.captures_iter(&content).collect();

        let mut processed_content = String::new();

        let mut last_endpoint: usize = 0;

        for capture in captures {
            let full_match = capture.get(0).unwrap();

            if let Some(file_name) = capture.name("file") {
                let file_link = FileLink::from_string(file_name.as_str());
                if let Some(path) = chapter_names.get(&file_link.name.to_lowercase()) {
                    processed_content.push_str(&content[last_endpoint..full_match.start()]);
                    last_endpoint = full_match.end();

                    processed_content.push_str(options.site_path.as_str());
                    processed_content.push_str(path.to_str().unwrap());
                    if let Some(anchor) = file_link.anchor {
                        processed_content.push_str("#");
                        processed_content.push_str(anchor);
                    }
                } else {
                    eprintln!("Error: Found request to replace link with '{}', but no chapter with that name found.", file_link.name.to_lowercase());
                    return Err(ProcessorError::ChapterNotFound(file_link.name.to_lowercase()));
                }
            }
        }

        if content.len() > last_endpoint {
            processed_content.push_str(&content[last_endpoint..content.len()]);
        }

        Ok(processed_content)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::{PathProcessor, PathProcessorOptions};

    #[test]
    fn test_process_chapter_replaces_links_to_top_level() {
        let content = "[foo]({{#path_for Foo}})";

        let mut chapter_mapping: HashMap<String, PathBuf> = HashMap::new();
        chapter_mapping.insert("foo".to_string(), PathBuf::from("something/Foo.md"));

        let subject = PathProcessor;

        let received_chapter = subject.process_chapter(&content, &chapter_mapping, &processor_options("/")).unwrap();

        let expected_chapter = "[foo](/something/Foo.md)";

        assert_eq!(received_chapter, expected_chapter.to_string());
    }

    #[test]
    fn test_process_chapter_replaces_links_to_anchor() {
        let content = "[foo]({{#path_for Foo#bar}})";

        let mut chapter_mapping: HashMap<String, PathBuf> = HashMap::new();
        chapter_mapping.insert("foo".to_string(), PathBuf::from("something/Foo.md"));

        let subject = PathProcessor;

        let received_chapter = subject.process_chapter(&content, &chapter_mapping, &processor_options("/root/")).unwrap();

        let expected_chapter = "[foo](/root/something/Foo.md#bar)";

        assert_eq!(received_chapter, expected_chapter.to_string());
    }

    fn processor_options(site_path: &str) -> PathProcessorOptions {
        PathProcessorOptions {
            site_path: site_path.to_string(),
            strict_mode: false
        }
    }
}