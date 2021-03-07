use csv;
use std::env;
use std::path::Path;
use std::borrow::Cow;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

#[derive(Debug)]
enum FileType {
    Text,
    Csv,
}

#[derive(Debug)]
struct File<T> {
    path: T,
    file_type: FileType,
}

impl<'a, T: ToString> File<T> {
    pub fn new(s: T) -> Self {

        let str_path: Cow<String> = Cow::Owned(s.to_string());
        let path = Path::new(str_path.as_ref());

        dbg!(&path);
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
            ext => panic!(format!("Not authorized '{}' extension file", ext.as_str())),
        };

        File {
            path: s,
            file_type,
        }
    }
}


#[derive(Debug)]
struct Reader<T> {
    files: Rc<RefCell<Vec<File<T>>>>,
}

impl <T>Reader<T> {
    pub fn new(args: Vec<String>) -> Self<> {
        let files:Rc<RefCell<Vec<File<String>>>> = Rc::new(RefCell::new(Vec::new()));

        for arg in args {
            let file: File<String> = File::new(arg);


            let mut vec: RefMut<_> = files.borrow_mut();
            vec.push(file);
        }

        Reader {
            files: Rc::new(RefCell::new(Vec::new()))
        }
    }
}

fn main() {
    let mut args: Vec<String> = env::args()
        .skip(1)
        .collect::<Vec<String>>();

    let reader: Reader<Rc<RefCell<Vec<File<String>>>>> = Reader::new(args);
    dbg!(&reader);
}
