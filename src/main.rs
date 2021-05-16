use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use csv::ReaderBuilder;

#[derive(Debug, StructOpt)]
#[structopt(name = "dff")]
struct Opt {
    #[structopt(parse(from_os_str))]
    primary_source_file: PathBuf,

    #[structopt(name = "primary-header", short = "p")]
    primary_header: Option<String>,

    #[structopt(parse(from_os_str))]
    secondary_source_file: PathBuf,

    #[structopt(name = "secondary-header", short = "s")]
    secondary_header: Option<String>,

    target: Option<PathBuf>,
}

enum FileType {
    Text,
    Csv,
}

impl FromStr for FileType {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "txt" => Ok(FileType::Text),
            "csv" => Ok(FileType::Csv),
            _ => panic!("File extension not found. Available are 'txt' and 'csv'"),
        }
    }
}

trait FileTypeExt {
    fn get_filetype(&self) -> Result<FileType, io::Error>;
}

impl FileTypeExt for PathBuf {
    fn get_filetype(&self) -> Result<FileType, io::Error> {
        Ok(FileType::from(
            self.as_path()
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .parse()?,
        ))
    }
}

fn read_from_path(
    path_buffer: PathBuf,
    header: Option<String>,
) -> Result<HashSet<String>, io::Error> {
    match path_buffer.get_filetype()? {
        FileType::Text => {
            let file = File::open(path_buffer.as_path())?;
            let buffer = BufReader::new(file);
            let data: HashSet<String> = buffer
                .lines()
                .map(|l| l.expect("Could not parse line"))
                .collect();

            Ok(data)
        }
        FileType::Csv => Ok(read_csv(path_buffer, header)?)
    }
}

fn read_csv(
    path_buffer: PathBuf,
    header: Option<String>,
) -> Result<HashSet<String>, io::Error> {

    let skip_first = match header {
        Some(_header) => false,
        None => true,
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(skip_first)
        .from_path(path_buffer.as_path())?;

    for result in reader.deserialize() {
        let record: Vec<String> = result?;
        println!("{:?}", record);
    }

    Ok(HashSet::new())
}

fn write_target(values: Vec<String>, path_buffer: PathBuf) -> Result<(), io::Error> {
    let mut file = File::create(path_buffer.as_path())?;
    for value in values {
        write!(file, "{}", value)?;
    }

    Ok(())
}

fn output_result(values: Vec<String>, path_buffer: Option<PathBuf>) -> Result<(), io::Error> {
    if values.is_empty() {
        return Ok(());
    }

    let output_stdout = move |values: Vec<String>| -> Result<(), io::Error> {
        for value in values {
            println!("{}", value);
        }

        Ok(())
    };

    match path_buffer {
        Some(path_buf) => match path_buf.get_filetype()? {
            FileType::Text => write_target(values, path_buf),
            _ => return output_stdout(values),
        },
        None => return output_stdout(values),
    }?;

    Ok(())
}

fn main() {
    let args: Opt = Opt::from_args();

    let primary_data = read_from_path(args.primary_source_file, args.primary_header)
        .expect("Unable to parse file of primary source");
    let secondary_data = read_from_path(args.secondary_source_file, args.secondary_header)
        .expect("Unable to parse file of secondary source");

    let result: Vec<String> = primary_data
        .difference(&secondary_data)
        .into_iter()
        .cloned()
        .collect();

    output_result(result, args.target).expect("Unable to output the result");
}
