// TODO Handle errors properly
// TODO Include output, behind --verbose flag

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate clap;

mod models;
mod schema;

use clap::{App, Arg, SubCommand};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use self::diesel::prelude::*;
use self::models::*;

embed_migrations!("./migrations");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a new bookmark")
                .arg(
                    Arg::with_name("url")
                        .help("The destination URL to bookmark")
                        .required(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .help("Tags (comma-separated) for the bookmark")
                        .long("tags")
                        .short("t")
                        .takes_value(true)
                        .use_delimiter(true),
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
        .get_matches();
    // TODO Filter a list of bookmarks by tag
    // TODO Limited number of bookmarks listed, can be overriden with option
    // TODO Export bookmarks in a form that browsers can ingest
    // TODO Specify location of DB file
    // TODO Add multiple tags to a bookmark with the tag command
    // TODO Allow for filtering by multiple tags when listing bookmarks

    let database: &str;
    let mut path = PathBuf::new(); // Guess Rust wants this declared here
    if let Some(value) = matches.value_of("database") {
        database = value
    } else {
        let home = env::var("HOME")?; // TODO Make it work with Windows
        path.push(&home);
        path.push(".bookmarks");
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
        let bookmark = BookmarkToInsert {
            url,
            name: None, // TODO Get title from input/HTML
        };
        diesel::insert_into(schema::bookmark::table)
            .values(&bookmark)
            .execute(&conn)?;
        let inserted_bookmark = schema::bookmark::table
            .filter(schema::bookmark::url.eq(url))
            .first::<Bookmark>(&conn)?;

        let mut tags: Vec<TagToInsert> = Vec::new();
        if matches.is_present("tags") {
            for value in matches.values_of("tags").unwrap().collect::<Vec<&str>>() {
                let ins = TagToInsert {
                    value: String::from(value),
                    bookmark_id: inserted_bookmark.id,
                };
                tags.push(ins);
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

        // TODO Add option to choose between pretty output and parseable output
        for result in results {
            let taglist = result
                .1
                .iter()
                .map(|t| t.value.clone())
                .collect::<Vec<String>>()
                .join(", ");
            println!("{}\t{}\t{}", result.0.id, result.0.url, taglist)
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
    }
    Ok(())
}
