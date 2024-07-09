# Forge Move

`forge_move` is utility for moving websites from one server to another. I developed this for Laravel Forge servers, however it will work with any linux server. It will detect the type of website and backup both the database and website files and copy them to the destination server and restore the DB on that server.

## Requirements

- Rust 1.74 or newer
- `mysqldump`
- `gzip`
- Password-less SSH access configured between servers

### Rust toolchain

To install the rust toolchain, run the following command
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installation

To install `forge_move`, ensure you have the Rust toolchain installed, then run:

```sh
cargo install forge_move
```

## Usage

To use `forge_move`, run the following command:

```sh
forge_backup [OPTIONS]
```

### Options
- --dir <WEBSITE_FOLDER>: Specify where the website files are.
- --server <DEST_SERVER>: Specify the destination server.
- --target <TARGET_FOLDER>: Specify the target folder on the new server.
- -h, --help: Print help information.
- -V, --version: Print version information.


### Example

```sh
forge_move --dir . --server new-server --target /home/new_user/some-site.com.au
```

## Contribution Guidelines

We welcome contributions. Please follow these guidelines:

1. Fork the repository and clone your fork.
2. Create new branch for your feature or bugfix.
3. Make your changes and test thoroughly.
4. Commit your changes with clear and descriptive commit messages.
5. Push your branch to your forkl.
6. Open a pull request and describe your changes.

## License

This project is licensed under the MIT License.

## Issues and Feature Requests

If you encounter any issues or have feature requests, please submit them on our GitHub Issues page.

## Contact

For any questions or support, please feel free to reach out via GitHub.

Thank you for using `forge_move`!



