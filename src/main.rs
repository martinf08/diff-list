use csv::ReaderBuilder;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};
use structopt::StructOpt;

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
    path_buffer: &PathBuf,
    header: &Option<String>,
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
        FileType::Csv => Ok(read_csv(&path_buffer, &header)?),
    }
}

fn read_csv(path_buffer: &PathBuf, header: &Option<String>) -> Result<HashSet<String>, io::Error> {
    let skip_first = match &header {
        Some(_header) => false,
        None => true,
    };

    let mut reader = ReaderBuilder::new()
        .has_headers(skip_first)
        .from_path(path_buffer.as_path())?;

    let mut index: Option<usize> = Some(0);
    if let Some(search) = &header {
        if let Some(found_index) = reader
            .deserialize::<Vec<String>>()
            .next()
            .unwrap()?
            .iter()
            .position(|v: &String| v == search)
        {
            index = Some(found_index);
        }
    }

    let mut data: HashSet<String> = HashSet::new();
    for result in reader.deserialize() {
        let record: Vec<String> = result?;

        if let Some(index) = index {
            let values: HashSet<String> = HashSet::from_iter(
                record
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i == index)
                    .map(|(_, v)| v)
                    .cloned(),
            );

            data.extend(values)
        }
    }

    Ok(data)
}

fn write_file(values: Vec<String>, path_buffer: &PathBuf) -> Result<(), io::Error> {
    let mut file = File::create(path_buffer.as_path())?;
    for value in values {
        write!(file, "{}", value)?;
    }

    Ok(())
}

fn output_result(values: Vec<String>, path_buffer: &Option<PathBuf>) -> Result<(), io::Error> {
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
            FileType::Text => write_file(values, path_buf),
            _ => return output_stdout(values),
        },
        None => return output_stdout(values),
    }?;

    Ok(())
}

fn get_result(
    primary_handle: JoinHandle<Result<HashSet<String>, io::Error>>,
    secondary_handle: JoinHandle<Result<HashSet<String>, io::Error>>,
) -> Result<Vec<String>, io::Error> {
    let primary_data = primary_handle.join().unwrap().unwrap();
    let secondary_data = secondary_handle.join().unwrap().unwrap();

    return Ok(primary_data
        .difference(&secondary_data)
        .into_iter()
        .cloned()
        .collect());
}

fn main() {
    let args: Arc<Mutex<Opt>> = Arc::new(Mutex::new(Opt::from_args()));
    let local_args = Arc::clone(&args);

    let primary_handle =
        thread::spawn(move || -> Result<HashSet<String, RandomState>, io::Error> {
            let args = local_args.lock().expect("Unable to lock args in thread");
            return Ok(read_from_path(
                &args.primary_source_file,
                &args.primary_header,
            )?);
        });

    let local_args = Arc::clone(&args);
    let secondary_handle =
        thread::spawn(move || -> Result<HashSet<String, RandomState>, io::Error> {
            let args = local_args.lock().expect("Unable to lock args in thread");
            return Ok(read_from_path(
                &args.secondary_source_file,
                &args.secondary_header,
            )?);
        });

    if let Ok(result) = get_result(primary_handle, secondary_handle) {
        output_result(
            result,
            &args
                .clone()
                .lock()
                .expect("Unable to lock value to output")
                .target,
        )
        .expect("Unable to output the result");
    }
}
