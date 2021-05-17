# diff-list

- Tool allowing difference between values of two files.

## Usage
### Text files
<pre>cargo -- file_1.txt file_2.txt {output_file}</pre>
- The default output is the standard output

## Run with csv
- For work with headers, options -p (for primary source) and -s (secondary) are required.
<pre>cargo run -- file_1.csv -p file_1_header file_2.csv {output_file}</pre>

## Multiple types
- Csv and text files can be combined.