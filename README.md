# bookmark

Save bookmarks from the command line.

You'll need to have SQLite3 installed. Bookmarks are saved to `~/.bookmarks/bookmarks.db`.

```sh
cargo build

target/debug/bookmark add https://nicholas.cloud/
target/debug/bookmark add https://youtu.be/L_XJ_s5IsQc --tags fusion,music
target/debug/bookmark list
# 1       https://nicholas.cloud/
# 2       https://youtu.be/L_XJ_s5IsQc    fusion, music
```