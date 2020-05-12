# bookmark

Save bookmarks from the command line.

You'll probably need to have SQLite installed. Bookmarks are saved to `~/.bookmark/bookmarks.db`.

If you're on Windows, bookmarks will be saved to `$APPDATA/.bookmark/bookmarks.db`. I don't regularly test against Windows, but hopefully you should be able to build and run this.

## Usage

```sh
git clone https://github.com/nchlswhttkr/bookmark/
cargo install --path bookmark

# Add a bookmark
bookmark add https://github.com/

# Add a bookmark with a name and several tags
bookmark add https://www.youtube.com/watch?v=ElPkT5Qvzw8 --name "Baka Mitai Cover" --tags music,gaming

# List bookmark
bookmark list

# List bookmarks with a certain tag
bookmark list --tagged music

# Delete a bookmark by its URL or its ID
bookmark delete https://github.com/
bookmark delete 2
```
