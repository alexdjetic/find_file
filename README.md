# Find File

Find File is a command-line utility written in Rust that allows you to search for files in specified directories based on various criteria.

## Features

- Search for files in multiple directories
- Filter files by name using regex patterns
- Exclude files based on patterns
- Option to include hidden files in the search
- Search for content within files
- Display detailed search parameters (optional)

## Installation

1. Ensure you have Rust installed on your system. If not, install it from [https://www.rust-lang.org/](https://www.rust-lang.org/).

2. Clone this repository:
   ```bash
   git clone https://github.com/alexdjetic/find_file.git
   cd find_file
   ```

3. Build the project:
   ```bash
   git clone https://github.com/alexdjetic/find_file.git
   cd find_file
   cargo build --release
   sudo cp target/release/find_file /usr/local/bin/
   chmod +x /usr/local/bin/find_file
   ```

4. The executable will be in `target/release/find_file`.

## Usage

To use Find File, run the executable with the appropriate arguments. Here are the basic syntax and options:

```bash
find_file [options] <directory> <regex_pattern>
```

### Options

- `-d, --directory <directory>`: Specify the directory to search in.
- `-f, --filter <filter_pattern>`: Specify the filter pattern to filter file names.
- `-e, --exclude <exclude_pattern>`: Specify the pattern to exclude files.
- `-a, --include-hidden`: Include hidden files in the search.
- `-c, --content <content>`: Search for content within files.
- `-p, --parameter-show`: Display detailed search parameters.
- `-h, --help`: Display help information.
- `-v, --version`: Display version information.

### Examples

1. Search for files in the current directory with names matching the regex pattern `.*\.txt`:

```bash
./target/release/find_file . ".*\.txt"
```

2. Search for files in the `/home/user/documents` directory with names matching the regex pattern `.*\.txt` and exclude files with names matching the pattern `ignore_.*`:

```bash
./target/release/find_file /home/user/documents ".*\.txt" "ignore_.*"
```

3. Search for files in the `/home/user/documents` directory with names matching the regex pattern `.*\.txt` and include hidden files:

```bash
./target/release/find_file /home/user/documents ".*\.txt" -i
```

4. Search for files in the `/home/user/documents` directory with names matching the regex pattern `.*\.txt` and search for the content "important":

```bash
./target/release/find_file /home/user/documents ".*\.txt" -c "important"
```

5. Search for files in the `/home/user/documents` directory with names matching the regex pattern `.*\.txt` and display detailed search parameters:

```bash
./target/release/find_file /home/user/documents ".*\.txt" -v
```
