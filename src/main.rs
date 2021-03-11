use csv;
use std::borrow::BorrowMut;
use std::collections::{HashSet, HashMap};
use std::fs::File as FileReader;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;
use std::path::Path;
use std::process::exit;
use std::env;

enum FileType {
    Text,
    Csv,
}

struct File {
    path: String,
    file_type: FileType,
}

impl File {
    pub fn new(s: &str) -> Self {
        let path = Path::new(s);

        if !path.is_file() {
            panic!(format!("{} is not a file", path.to_str().unwrap()));
        }

        let file_type = match path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase() {
            ext if ext == "txt" => FileType::Text,
            ext if ext == "csv" => FileType::Csv,
            ext => panic!(
                format!(
                    "Not authorized '{}' extension file.\nExtensions allowed : txt, csv",
                    ext.as_str()
                )
            ),
        };

        File {
            path: s.parse().unwrap(),
            file_type,
        }
    }
}

struct Reader {
    files: Vec<File>,
    result: Vec<HashSet<String>>,
    options: Option<HashMap<String, String>>,
}

impl Reader {
    pub fn new(mut args: Vec<String>) -> Self {
        let mut files = Vec::new();

        let mut options = HashMap::new();
        for arg in args.drain(..) {
            let arg_cloned = arg.clone();

            if arg_cloned.starts_with("--") && arg_cloned.contains(&String::from("=")) {
                if let [option_name, value] = arg
                    .split("=")
                    .borrow_mut()
                    .map(|item| String::from(item))
                    .collect::<Vec<String>>()
                    .as_slice() {
                    options.insert(option_name.clone(), value.clone());
                }

                continue;
            }
            let file = File::new(&*arg);

            files.push(file);
        }

        Reader {
            options: Some(options),
            files,
            result: Vec::new(),
        }
    }

    fn read(&mut self) -> () {
        for file in self.files.iter_mut() {
            let file_result = match file.file_type {
                FileType::Csv => { Reader::read_csv(file, self.options.as_ref()) }
                FileType::Text => { Reader::read_txt(file) }
            };

            self.result.push(HashSet::from_iter(file_result.into_iter()));
        }
    }

    fn read_csv(file: &File, options: Option<&HashMap<String, String>>) -> Vec<String> {
        let header = match options {
            Some(options) => {
                if options.contains_key("--header") {
                    options.get("--header").unwrap().as_str()
                } else {
                    ""
                }
            }
            None => "",
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(!header.is_empty())
            .from_path(&file.path)
            .unwrap();

        let mut result_file: Vec<String> = Vec::new();
        let mut index = Some(0);
        for (i, result) in reader.records().enumerate() {
            let line = result
                .unwrap()
                .iter()
                .map(|item| String::from(item))
                .collect::<Vec<String>>();

            if i == 0 && !header.is_empty() && line.contains(&String::from(header)) {
                index = line.iter().position(|r| r == header);
                continue;
            }
            result_file.push(
                line[index.unwrap()].clone()
            );
        }

        result_file
    }

    fn read_txt(file: &File) -> Vec<String> {
        let file_handler = FileReader::open(&file.path).unwrap();

        let result: Vec<String> = BufReader::new(file_handler)
            .lines()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|item| item.unwrap())
            .collect();

        result
    }

    pub fn display_diff(&mut self) {
        self.read();

        self.result[0]
            .difference(&self.result[1])
            .into_iter()
            .for_each(|item| { println!("{:?}", item) });
    }
}

fn main() {
    let args: Vec<String> = env::args()
        .skip(1)
        .collect::<Vec<String>>();

    if args.len() == 0 || args.first().unwrap() == "--help" {
        println!("{}", format!("Only .csv and .txt files allowed
Usage:
    diff-list --help
    diff-list <path-1> <path-2>
    diff-list --header=<header-name> (For csv header only)"
        ));
        exit(0);
    }

    let mut reader = Reader::new(args);
    reader.display_diff()
}
