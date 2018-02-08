// Partially reflected `tar` command with these actions:
// * Create an archive from files
// * Extract an archive in a target folder
// * List the contents of a tar file

extern crate fui;

use fui::feeders::DirItems;
use fui::fields::{Autocomplete, Text};
use fui::form::FormView;
use fui::utils::cwd;
use fui::validators::{OneOf, Required};
use fui::{Fui, Value};

fn hdlr(v: Value) {
    println!("user input (from hdlr) {:?}", v);
}

fn compression_format() -> Autocomplete {
    // cloning Autocomplete is not implemented yet, so we're using clone_target for that
    // perhaps instead of cloning, simple reference would be enough but this needs research and
    // implementation, so stay tuned
    let formats = vec!["none", "gzip", "bzip2"];
    Autocomplete::new("file_to_archive", formats.clone())
        .initial("gzip")
        .validator(Required)
        .validator(OneOf(formats))
        .help("Archive format")
}

fn main() {
    Fui::new()
        .action(
            "ARCHIVE-FILES: Create an archive from files",
            FormView::new()
                .field(
                    Autocomplete::new("file-to-archive", DirItems::current_dir().files())
                        .help("Files which should be archived")
                        //TODO: .multi(true)
                        .validator(Required),
                )
                .field(
                    Text::new("target")
                        .help("Name of archive file")
                        .validator(Required),
                )
                .field(compression_format()),
            hdlr,
        )
        .action(
            "EXTRACT-TO-DIR: Extract an archive in a target folder",
            FormView::new()
                .field(
                    Autocomplete::new("archive-path", DirItems::current_dir().files())
                        .help("Path to compressed file")
                        .validator(Required),
                )
                .field(
                    Autocomplete::new("dst-dir", DirItems::current_dir().dirs())
                        .initial(cwd())
                        .help("Dir where extracted files should land"),
                )
                .field(compression_format()),
            hdlr,
        )
        .action(
            "LIST-ARCHIVE: List the contents of a tar file",
            FormView::new()
                .field(
                    Autocomplete::new("archive-file", DirItems::current_dir().files())
                        .help("Path to archive"),
                )
                .field(compression_format()),
            hdlr,
        )
        .run();
}