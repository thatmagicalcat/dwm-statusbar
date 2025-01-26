# dwm-statusbar

`dwm-statusbar` is a customizable status bar for the dwm window manager, written in Rust.

## Features

- **Customizable Blocks**: Easily configure which information to display and how often to update it in rust.
- **Windowing Feature**: Switch between different sets of status blocks using a Unix socket.

## Usage

### Configuration

The status bar is configured in the `main.rs` file. You can define different blocks and their update intervals using the `blocks!` macro. For example:

```rust
use dwm_statusbar::*;

StatusBar::new(
    " | ", // separator
    vec![
        // blocks! {
        //    { function, update interval in ms },
        // }

        // window 1
        blocks! {
            { |_num_updates| "window 1".to_string(), 1_000 },
        },
        
        // window 2
        blocks! {
            { |_num_updates| "window 2".to_string(), 1_000 },
        },
    ],
)
```

Check `src/main.rs` for a template on how to use this for your own status bar.

### Windowing Feature

The windowing feature allows you to switch between different sets of status blocks. This can be useful if you want to display different information based on the context.

To switch windows, you can send the desired window index to the Unix socket located at `/tmp/dwm-statusbar.sock`. For example, to switch to window 1:

```sh
$ echo "1" | nc -U /tmp/dwm-statusbar.sock
```

### Running the Status Bar

To run the status bar, simply build and execute your Rust project:

```sh
$ cargo build --release
$ ./target/release/dwm-statusbar
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.