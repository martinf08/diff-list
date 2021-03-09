use csv;
use std::collections::HashSet;
use std::fs::File as FileReader;
use std::io::BufRead;
use std::iter::FromIterator;
use std::path::Path;
use std::{env, io};
use std::process::exit;

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
    result: Vec<HashSet<Vec<String>>>,
}

impl Reader {
    pub fn new(mut args: Vec<String>) -> Self <> {
        let mut files = Vec::new();

        for arg in args.drain(..) {
            let file = File::new(&*arg);

            files.push(file);
        }

        Reader {
            files,
            result: Vec::new(),
        }
    }

    pub fn read(&mut self) {
        for file in self.files.iter_mut() {
            self.result.push(match file.file_type {
                FileType::Csv => { Reader::read_csv(file) }
                FileType::Text => { Reader::read_txt(file) }
            });
        }
    }

    pub fn read_csv(file: &File) -> HashSet<Vec<String>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(&file.path)
            .unwrap();

        let mut result_file: Vec<Vec<String>> = Vec::new();
        for result in reader.records() {
            result_file.push(
                result
                    .unwrap()
                    .iter()
                    .map(|item| String::from(item))
                    .collect()
            );
        }

        HashSet::from_iter(result_file.into_iter())
    }

    pub fn read_txt(file: &File) -> HashSet<Vec<String>> {
        let file_handler = FileReader::open(&file.path).unwrap();

        let mut hash = HashSet::new();
        io::BufReader::new(file_handler)
            .lines()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|item| item.unwrap())
            .for_each(|item| {
                hash.insert(vec!(item));
                return ()
            });

        hash
    }
}

fn main() {
    let args: Vec<String> = env::args()
        .skip(1)
        .collect::<Vec<String>>();

    if args.len() > 0 && args.len() != 2 {
        panic!("Command need two files path as arguments. Extensions allowed : txt, csv")
    }

    if args.len() == 0 {
        exit(0)
    }

    let mut reader = Reader::new(args);
    reader.read();

    reader.result[0]
        .difference(&reader.result[1])
        .into_iter()
        .for_each(|item| { println!("{:?}", item) });
}
