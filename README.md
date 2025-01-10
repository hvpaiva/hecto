# hecto

**hecto** is a learning project where I follow [Philipp Flenker’s](https://www.flenker.blog) blog series on building a text editor in Rust. The goal is to deepen my understanding of low-level text manipulation, terminal I/O, raw mode, and Rust’s ownership model—while having fun creating a lightweight editor along the way.

## About

This project is based on the tutorial series:

> **hecto: Build Your Own Text Editor in Rust**  
> by Philipp Flenker  
> [https://www.flenker.blog/hecto/](https://www.flenker.blog/hecto/)

The tutorial explains how to construct a small yet functional text editor in Rust, covering aspects such as:

- Handling terminal input (key events, arrow keys, etc.)
- Managing raw mode for precise control over keyboard input
- Drawing text to the screen (clearing lines, moving the cursor)
- Basic editor architecture, from scrolling to a conceptual “Document”

**Important:** This repository is purely for **learning and experimentation**. It is not intended as a production-ready text editor and may contain unfinished features or experimental code.

## Features (So Far)

- **Terminal Abstraction Layer** using [crossterm](https://crates.io/crates/crossterm)
- **Keyboard Event Handling** to move the cursor around
- **Simple Document Model** (`Location` vs. on-screen `Position`)
- **Row Drawing** including a placeholder welcome message
- **Basic Commands** like clearing screen, quitting with `Ctrl+Q`

## Project Structure

- **`main.rs`**: Entry point; initializes the editor and handles top-level errors.
- **`editor.rs`**: Core editor logic, including the main loop and event handling.
- **`terminal.rs`**: Encapsulates terminal interactions (e.g., raw mode, clearing, cursor movement).
- **`io_provider.rs`** _(test utility)_: Supplies either `stdout()` or a mock buffer, making testing easier.

## Tests

Many functions, including those that draw to the screen, are tested by capturing terminal output in memory rather than printing to the real screen. This approach helps verify that each command writes the expected ANSI sequences without requiring a physical terminal in test environments.

## Contributing

As this is a personal learning project, contributions aren’t actively sought. However, if you spot any bugs or have suggestions, feel free to open an issue or pull request.

## License

The project does not currently specify a license. Since it’s for learning purposes, you are free to read through the code and adapt ideas in your own projects, but be mindful of any constraints you may have in a professional environment.
