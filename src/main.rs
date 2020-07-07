// TODO Handle errors properly
// TODO Include output, behind --verbose flag
// TODO Integration tests, ideally across different platforms in CI

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate clap;

mod models;
mod schema;

use clap::{App, AppSettings, Arg, SubCommand};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use self::diesel::prelude::*;
use self::models::*;

embed_migrations!("./migrations");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_REPOSITORY"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("database")
                .help("Overrides the default database location")
                .long("database")
                .global(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about(
                    "Add a new bookmark\n\nBy default, you'll be prompted for optional name/tags",
                )
                .arg(
                    Arg::with_name("url")
                        .help("The destination URL to bookmark")
                        .required(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .help("Tags (comma-separated) for the bookmark")
                        .long("tags")
                        .takes_value(true)
                        .use_delimiter(true),
                )
                .arg(
                    Arg::with_name("name")
                        .help("The name of the bookmark, defaults to an empty string")
                        .long("name")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List all bookmarks and their tags")
                .arg(
                    Arg::with_name("tagged")
                        .help("Only list bookmarks with a certain tag")
                        .long("tagged")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("no-pretty")
                        .help("Turns off padding for pretty output")
                        .long("no-pretty"),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete a bookmark")
                .arg(
                    Arg::with_name("target")
                        .help("The ID or URL of the bookmark to remove")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("tag")
                .about("Add a tag to an existing bookmark")
                .arg(
                    Arg::with_name("target")
                        .help("The ID or URL of the bookmark to remove")
                        .required(true),
                )
                .arg(
                    Arg::with_name("tag")
                        .help("The tag to apply to the bookmark")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("open")
                .about("Open a bookmark in your browser")
                .arg(
                    Arg::with_name("target")
                        .help("The ID or URL of the bookmark to remove")
                        .required(true),
                ),
        )
        .get_matches();
    // TODO Limited number of bookmarks listed, can be overriden with option
    // TODO Export bookmarks in a form that browsers can ingest
    // TODO Add multiple tags to a bookmark with the tag command
    // TODO Allow for filtering by multiple tags when listing bookmarks
    // TODO Add command to remove tags from a bookmark
    // TODO Allow bookmarks to be renamed/retagged

    let database: &str;
    let mut path = PathBuf::new(); // Guess Rust wants this declared here
    if let Some(value) = matches.value_of("database") {
        database = value
    } else {
        let home;
        if cfg!(windows) {
            home = env::var("APPDATA")?;
        } else {
            home = env::var("HOME")?;
        }
        path.push(&home);
        path.push(".bookmark");
        fs::create_dir_all(&path)?;
        path.push("bookmarks.db");
        database = path.to_str().unwrap();
    }

    let conn: SqliteConnection = SqliteConnection::establish(database)?;
    // https://sqlite.org/foreignkeys.html#fk_enable
    conn.execute("PRAGMA foreign_keys = ON;")?;

    embedded_migrations::run(&conn)?;

    if let Some(matches) = matches.subcommand_matches("add") {
        // TODO Run this in a transaction
        let url = matches.value_of("url").unwrap();
        // TODO Attempt to grab bookmark name from title metadata if not supplied
        let mut name = matches.value_of("name");
        let mut name_input: String = String::new(); // Could better manage lifetimes
        if name.is_none() {
            io::stdout().write(b"Name?\t")?;
            io::stdout().flush()?;
            io::stdin().read_line(&mut name_input)?;
            name = Some(name_input.trim());
        }
        let bookmark = BookmarkToInsert { url, name };
        diesel::insert_into(schema::bookmark::table)
            .values(&bookmark)
            .execute(&conn)?;
        let inserted_bookmark = schema::bookmark::table
            .filter(schema::bookmark::url.eq(url))
            .first::<Bookmark>(&conn)?;

        let mut tags: Vec<TagToInsert> = Vec::new();
        let mut tags_input: String = String::new();
        // TODO Clean up this duplicated logic
        if matches.is_present("tags") {
            for value in matches.values_of("tags").unwrap().collect::<Vec<&str>>() {
                let trimmed = value.trim();
                if trimmed.len() > 0 {
                    tags.push(TagToInsert {
                        value: String::from(trimmed),
                        bookmark_id: inserted_bookmark.id,
                    });
                } else {
                    eprintln!("Empty/whitespace tag ignored")
                }
            }
        } else {
            io::stdout().write(b"Tags?\t")?;
            io::stdout().flush()?;
            io::stdin().read_line(&mut tags_input)?;
            for tag_name in tags_input.split(',') {
                let trimmed = tag_name.trim();
                if trimmed.len() > 0 {
                    tags.push(TagToInsert {
                        value: String::from(tag_name.trim()),
                        bookmark_id: inserted_bookmark.id,
                    })
                } else {
                    eprintln!("Empty/whitespace tag ignored")
                }
            }
        }
        diesel::insert_into(schema::tag::table)
            .values(&tags)
            .execute(&conn)?;
    } else if let Some(matches) = matches.subcommand_matches("list") {
        let bookmarks = schema::bookmark::table.load::<Bookmark>(&conn)?;
        let tags = Tag::belonging_to(&bookmarks)
            .load::<Tag>(&conn)?
            .grouped_by(&bookmarks);
        let mut results: Vec<(Bookmark, Vec<Tag>)> = bookmarks.into_iter().zip(tags).collect();

        // TODO See if this can be done while querying (join bookmarks on filtered list of tags)
        if let Some(tagged) = matches.value_of("tagged") {
            results = results
                .into_iter()
                .filter(|r| match r.1.iter().find(|t| t.value == tagged) {
                    Some(_) => true,
                    None => false,
                })
                .collect();
        }

        let mut longest_url = 0;
        let mut longest_name = 0;
        if !matches.is_present("no-pretty") {
            for result in &results {
                if result.0.url.len() > longest_url {
                    longest_url = result.0.url.len();
                }
                if result.0.name.len() > longest_name {
                    longest_name = result.0.name.len();
                }
            }
        }
        for result in results {
            let taglist = result
                .1
                .iter()
                .map(|t| t.value.clone())
                .collect::<Vec<String>>()
                .join(", ");
            println!(
                "{}\t{: <longest_name$}\t{: <longest_url$}\t{}",
                result.0.id,
                result.0.name,
                result.0.url,
                taglist,
                longest_name = longest_name,
                longest_url = longest_url
            )
        }
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        let target = matches.value_of("target").unwrap();
        if let Ok(target_as_id) = i32::from_str(target) {
            diesel::delete(schema::bookmark::table.filter(schema::bookmark::id.eq(target_as_id)))
                .execute(&conn)?;
        } else {
            diesel::delete(schema::bookmark::table.filter(schema::bookmark::url.eq(target)))
                .execute(&conn)?;
        }
    } else if let Some(matches) = matches.subcommand_matches("tag") {
        let tag = matches.value_of("tag").unwrap();
        let target = matches.value_of("target").unwrap();
        let bookmark: Bookmark;
        if let Ok(target_as_id) = i32::from_str(target) {
            bookmark = schema::bookmark::table
                .filter(schema::bookmark::id.eq(target_as_id))
                .first(&conn)?;
        } else {
            bookmark = schema::bookmark::table
                .filter(schema::bookmark::url.eq(target))
                .first(&conn)?;
        }
        let tag_to_insert = TagToInsert {
            bookmark_id: bookmark.id,
            value: String::from(tag),
        };
        diesel::insert_into(schema::tag::table)
            .values(vec![tag_to_insert])
            .execute(&conn)?;
    } else if let Some(matches) = matches.subcommand_matches("open") {
        let target = matches.value_of("target").unwrap();
        let bookmark: Bookmark;
        if let Ok(target_as_id) = i32::from_str(target) {
            bookmark = schema::bookmark::table
                .filter(schema::bookmark::id.eq(target_as_id))
                .first(&conn)?;
        } else {
            bookmark = schema::bookmark::table
                .filter(schema::bookmark::url.eq(target))
                .first(&conn)?;
        }
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(vec!["/C", "start", bookmark.url.as_str()])
                .output()?;
        } else if cfg!(target_os = "macos") {
            Command::new("open").args(vec![bookmark.url]).output()?;
        } else if cfg!(target_os = "linux") {
            Command::new("xdg-open").args(vec![bookmark.url]).output()?;
        } else {
            eprintln!("Could not open bookmark, not implemented for your OS.")
        }
    }
    Ok(())
}
