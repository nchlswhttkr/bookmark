# bookmark

Save bookmarks from the command line. Still a WIP!

You'll probably need to have SQLite installed. Bookmarks are saved to `~/.bookmark/bookmarks.db` (or `$APPDATA/.bookmark/bookmarks.db` on Windows).

The CLI is still likely to change, but your bookmarks will carry over between upgrades. I use this myself, so I want to avoid breaking changes where possible!

## Usage

```sh
git clone https://github.com/nchlswhttkr/bookmark/
cargo install --path bookmark

# Add a bookmark (you'll be prompted for an optional name and tags)
bookmark add https://github.com/

# Add a bookmark with a name and several tags
bookmark add https://www.youtube.com/watch?v=ElPkT5Qvzw8 --name "Baka Mitai Cover" --tags music,gaming

# List bookmark
bookmark list

# List bookmarks with a certain tag
bookmark list --tagged music

# Open a bookmark in your default browser
bookmark open 1

# Delete a bookmark by its URL or its ID
bookmark delete https://github.com/
bookmark delete 2
```
