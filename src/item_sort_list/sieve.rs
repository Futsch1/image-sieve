use std::{
    fs::{copy, create_dir_all, metadata, remove_file, rename, File},
    io::{Error, ErrorKind, Read},
    path::{Path, PathBuf},
};

use chrono::Datelike;

use super::{file_item, timestamp_to_string, DirectoryNames, Format, ItemList, SieveMethod};

/// Trait to encapsulate sieve file IO operations
pub trait SieveIO {
    fn copy(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error>;
    fn remove_file(&self, path: &Path) -> Result<(), Error>;
    fn r#move(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error>;
    fn create_dir_all(&self, path: &Path) -> Result<(), Error>;
}

/// Struct with implementation for std::fs implementation of SieveIO
pub struct FileSieveIO;

impl FileSieveIO {
    fn different(&self, f1: &Path, f2: &Path) -> Result<bool, Error> {
        if metadata(f1)?.len() == metadata(f2)?.len() {
            let mut content1 = vec![];
            let mut fh1 = File::open(f1)?;
            fh1.read_to_end(&mut content1)?;
            let mut content2 = vec![];
            let mut fh2 = File::open(f2)?;
            fh2.read_to_end(&mut content2)?;
            Ok(content1 != content2)
        } else {
            Ok(true)
        }
    }

    fn check_target(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error> {
        if dest.exists() {
            if self.different(src, dest)? {
                let mut new_file_name = dest.file_stem().unwrap().to_os_string();
                new_file_name.push("_.");
                new_file_name.push(dest.extension().unwrap());
                let parent = dest.parent().unwrap().to_path_buf();
                dest.clear();
                dest.push(parent);
                dest.push(new_file_name);

                self.check_target(src, dest)
            } else {
                let e = Error::new(
                    ErrorKind::AlreadyExists,
                    format!("Destination file already exists: {}", dest.display()),
                );
                Err(e)
            }
        } else {
            Ok(())
        }
    }
}

impl SieveIO for FileSieveIO {
    fn copy(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error> {
        self.check_target(src, dest)?;
        copy(src, dest)?;
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), Error> {
        remove_file(path)
    }

    fn r#move(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error> {
        self.check_target(src, dest)?;
        match rename(src, dest.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.copy(src, dest)?;
                self.remove_file(src)
            }
        }
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), Error> {
        create_dir_all(path)
    }
}

/// Sieves an item list taking the take_over flag into account to a new directory.
/// The progress is reported by calling a callback function with the file that is currently processed.
pub fn sieve<T>(
    item_list: &ItemList,
    path: &Path,
    sieve_method: SieveMethod,
    sieve_directory_names: DirectoryNames,
    sieve_io: &T,
    progress_callback: impl Fn(String),
) where
    T: SieveIO,
{
    if sieve_method != SieveMethod::Delete {
        prepare_path(path, sieve_io);

        for item in &item_list.items {
            if item.get_take_over() {
                let sub_path: PathBuf = get_sub_path(item_list, item, &sieve_directory_names)
                    .iter()
                    .collect();
                let full_path = path.join(sub_path);
                prepare_path(&full_path, sieve_io);
                let source = &item.path;
                let mut target = full_path.join(source.file_name().unwrap());

                if sieve_method == SieveMethod::Copy {
                    match sieve_io.copy(source, &mut target) {
                        Ok(_) => (),
                        Err(e) => progress_callback(format!("Error copying {}: {}", item, e)),
                    }
                } else {
                    match sieve_io.r#move(source, &mut target) {
                        Ok(_) => (),
                        Err(e) => progress_callback(format!("Error moving {}: {}", item, e)),
                    }
                };
                progress_callback(format!("{:?} -> {:?}", source, target));
            } else if sieve_method == SieveMethod::MoveAndDelete {
                let source = &item.path;
                progress_callback(format!("Delete {:?}", source));
                match sieve_io.remove_file(source) {
                    Ok(_) => (),
                    Err(e) => progress_callback(format!("Error deleting {}: {}", item, e)),
                }
            }
        }
    } else {
        for item in &item_list.items {
            if !item.get_take_over() {
                let source = &item.path;
                progress_callback(format!("Delete {:?}", source));
                match sieve_io.remove_file(source) {
                    Ok(_) => (),
                    Err(e) => progress_callback(format!("Error deleting {:?}: {}", item, e)),
                }
            }
        }
    }

    progress_callback(String::from("Done"));
}

/// Gets the sub path of a file item taking the file item's timestamp and possible events into account.
/// If a fileitem is part of an event, its sub path is the event's span and name.
/// If it is not part of an event, its sub path is the file item's timestamp in the given format.
fn get_sub_path(
    item_list: &ItemList,
    item: &file_item::FileItem,
    directory_names: &DirectoryNames,
) -> Vec<String> {
    // TODO: This is a bit ugly.

    let mut directories = Vec::<String>::new();
    let event = item_list.get_event(item);
    if let Some(event) = event {
        if *directory_names == DirectoryNames::YearAndMonthInSubdirectory {
            directories.push(event.start_date.format("%Y").to_string());
            if event.start_date != event.end_date {
                directories.push(format!(
                    "{} - {} {}",
                    event.start_date.format("%m-%d"),
                    event
                        .end_date
                        .format(if event.start_date.year() != event.end_date.year() {
                            "%Y-%m-%d"
                        } else {
                            "%m-%d"
                        }),
                    event.name
                ));
            } else {
                directories.push(format!(
                    "{} {}",
                    event.start_date.format("%m-%d"),
                    event.name
                ));
            }
        } else if event.start_date != event.end_date {
            directories.push(format!(
                "{} - {} {}",
                event.start_date.format("%Y-%m-%d"),
                event.end_date.format("%Y-%m-%d"),
                event.name
            ));
        } else {
            directories.push(format!(
                "{} {}",
                event.start_date.format("%Y-%m-%d"),
                event.name
            ));
        }
    } else {
        let format = match directory_names {
            DirectoryNames::YearAndMonth => Format::YearAndMonth,
            DirectoryNames::Year => Format::Year,
            DirectoryNames::YearMonthAndDay => Format::Date,
            DirectoryNames::YearAndQuarter => Format::YearAndQuarter,
            DirectoryNames::YearAndMonthInSubdirectory => Format::Year,
        };
        directories.push(timestamp_to_string(item.get_timestamp(), format));
        if *directory_names == DirectoryNames::YearAndMonthInSubdirectory {
            directories.push(timestamp_to_string(item.get_timestamp(), Format::Month))
        }
    }

    directories
}

/// Prepares the path by creating it if it does not exist
fn prepare_path<T>(path: &Path, sieve_io: &T)
where
    T: SieveIO,
{
    if !path.exists() {
        match sieve_io.create_dir_all(path) {
            Ok(_) => (),
            Err(e) => println!("Error creating path {}: {}", e, path.display()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::item_sort_list::sieve::SieveIO;
    use crate::item_sort_list::{sieve::get_sub_path, Event, FileItem, ItemList};
    use num_traits::FromPrimitive;
    use std::cell::RefCell;
    use std::path::PathBuf;

    struct TestSieveIO {
        pub copies: RefCell<Vec<(PathBuf, PathBuf)>>,
        pub renames: RefCell<Vec<(PathBuf, PathBuf)>>,
        pub removes: RefCell<Vec<PathBuf>>,
        pub creates: RefCell<Vec<PathBuf>>,
    }

    impl TestSieveIO {
        pub fn new() -> Self {
            TestSieveIO {
                copies: RefCell::new(vec![]),
                renames: RefCell::new(vec![]),
                removes: RefCell::new(vec![]),
                creates: RefCell::new(vec![]),
            }
        }

        pub fn reset(&mut self) {
            self.copies.get_mut().clear();
            self.renames.get_mut().clear();
            self.removes.get_mut().clear();
            self.creates.get_mut().clear();
        }
    }

    impl SieveIO for TestSieveIO {
        fn copy(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error> {
            self.copies
                .borrow_mut()
                .push((src.to_path_buf(), dest.to_path_buf()));
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<(), Error> {
            self.removes.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn r#move(&self, src: &Path, dest: &mut PathBuf) -> Result<(), Error> {
            self.renames
                .borrow_mut()
                .push((src.to_path_buf(), dest.to_path_buf()));
            Ok(())
        }

        fn create_dir_all(&self, path: &Path) -> Result<(), Error> {
            self.creates.borrow_mut().push(path.to_path_buf());
            Ok(())
        }
    }

    #[test]
    fn test_get_sub_path() {
        use chrono::NaiveDate;
        use chrono::NaiveDateTime;

        let item_list = ItemList {
            items: vec![],
            events: vec![
                Event {
                    name: String::from("Test1"),
                    start_date: NaiveDate::from_ymd_opt(2021, 9, 14).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2021, 9, 14).unwrap(),
                },
                Event {
                    name: String::from("Test2"),
                    start_date: NaiveDate::from_ymd_opt(2021, 9, 20).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2021, 9, 21).unwrap(),
                },
                Event {
                    name: String::from("Test3"),
                    start_date: NaiveDate::from_ymd_opt(2021, 9, 24).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2022, 9, 27).unwrap(),
                },
            ],
            path: PathBuf::from(""),
        };
        let test_cases = [
            (
                "2021-09-14 00:00",
                [
                    "2021-09-14 Test1",
                    "2021-09-14 Test1",
                    "2021-09-14 Test1",
                    "2021-09-14 Test1",
                    "202109-14 Test1",
                ],
            ),
            (
                "2021-09-13 23:59",
                ["2021-09", "2021", "2021-09-13", "2021-Q3", "202109"],
            ),
            (
                "2021-09-15 00:00",
                ["2021-09", "2021", "2021-09-15", "2021-Q3", "202109"],
            ),
            (
                "2021-09-20 00:00",
                [
                    "2021-09-20 - 2021-09-21 Test2",
                    "2021-09-20 - 2021-09-21 Test2",
                    "2021-09-20 - 2021-09-21 Test2",
                    "2021-09-20 - 2021-09-21 Test2",
                    "202109-20 - 09-21 Test2",
                ],
            ),
            (
                "2021-09-21 16:00",
                [
                    "2021-09-20 - 2021-09-21 Test2",
                    "2021-09-20 - 2021-09-21 Test2",
                    "2021-09-20 - 2021-09-21 Test2",
                    "2021-09-20 - 2021-09-21 Test2",
                    "202109-20 - 09-21 Test2",
                ],
            ),
            (
                "2021-09-25 16:00",
                [
                    "2021-09-24 - 2022-09-27 Test3",
                    "2021-09-24 - 2022-09-27 Test3",
                    "2021-09-24 - 2022-09-27 Test3",
                    "2021-09-24 - 2022-09-27 Test3",
                    "202109-24 - 2022-09-27 Test3",
                ],
            ),
            (
                "2020-01-13 23:59",
                ["2020-01", "2020", "2020-01-13", "2020-Q1", "202001"],
            ),
            (
                "2021-04-15 00:00",
                ["2021-04", "2021", "2021-04-15", "2021-Q2", "202104"],
            ),
        ];

        for (input, results) in test_cases {
            for (i, result) in results.into_iter().enumerate() {
                let sub_path: String = get_sub_path(
                    &item_list,
                    &FileItem::dummy(
                        "test.jpg",
                        NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M")
                            .unwrap()
                            .and_utc()
                            .timestamp(),
                        false,
                    ),
                    &FromPrimitive::from_usize(i).unwrap(),
                )
                .join("");
                assert_eq!(sub_path, result);
            }
        }
    }

    #[test]
    fn test_sieve_methods() {
        let item_list = ItemList {
            items: vec![
                FileItem::dummy("test/test1.jpg", 0, true),
                FileItem::dummy("test/test2.jpg", 0, false),
            ],
            events: vec![],
            path: PathBuf::from(""),
        };
        let mut sieve_io = TestSieveIO::new();

        sieve(
            &item_list,
            Path::new("target"),
            SieveMethod::Delete,
            DirectoryNames::YearAndMonth,
            &sieve_io,
            |_: String| {},
        );
        assert_eq!(sieve_io.copies.borrow().len(), 0);
        assert_eq!(sieve_io.creates.borrow().len(), 0);
        assert_eq!(sieve_io.renames.borrow().len(), 0);
        assert_eq!(sieve_io.removes.borrow().len(), 1);
        assert_eq!(
            sieve_io.removes.borrow()[0].to_str().unwrap(),
            "test/test2.jpg"
        );

        sieve_io.reset();
        sieve(
            &item_list,
            Path::new("target"),
            SieveMethod::Copy,
            DirectoryNames::YearAndMonth,
            &sieve_io,
            |_: String| {},
        );
        assert_eq!(sieve_io.copies.borrow().len(), 1);
        assert_eq!(
            sieve_io.copies.borrow()[0].0.to_str().unwrap(),
            "test/test1.jpg"
        );
        assert_eq!(
            sieve_io.copies.borrow()[0].1.to_str().unwrap(),
            "target/1970-01/test1.jpg"
        );
        assert_eq!(sieve_io.creates.borrow().len(), 1);
        assert_eq!(
            sieve_io.creates.borrow()[0].to_str().unwrap(),
            "target/1970-01"
        );
        assert_eq!(sieve_io.renames.borrow().len(), 0);
        assert_eq!(sieve_io.removes.borrow().len(), 0);

        sieve_io.reset();
        sieve(
            &item_list,
            Path::new("target"),
            SieveMethod::Move,
            DirectoryNames::YearAndMonth,
            &sieve_io,
            |_: String| {},
        );
        assert_eq!(sieve_io.copies.borrow().len(), 0);
        assert_eq!(sieve_io.creates.borrow().len(), 1);
        assert_eq!(
            sieve_io.creates.borrow()[0].to_str().unwrap(),
            "target/1970-01"
        );
        assert_eq!(sieve_io.renames.borrow().len(), 1);
        assert_eq!(
            sieve_io.renames.borrow()[0].0.to_str().unwrap(),
            "test/test1.jpg"
        );
        assert_eq!(
            sieve_io.renames.borrow()[0].1.to_str().unwrap(),
            "target/1970-01/test1.jpg"
        );
        assert_eq!(sieve_io.removes.borrow().len(), 0);

        sieve_io.reset();
        sieve(
            &item_list,
            Path::new("target"),
            SieveMethod::MoveAndDelete,
            DirectoryNames::YearAndMonth,
            &sieve_io,
            |_: String| {},
        );
        assert_eq!(sieve_io.copies.borrow().len(), 0);
        assert_eq!(sieve_io.creates.borrow().len(), 1);
        assert_eq!(
            sieve_io.creates.borrow()[0].to_str().unwrap(),
            "target/1970-01"
        );
        assert_eq!(sieve_io.renames.borrow().len(), 1);
        assert_eq!(
            sieve_io.renames.borrow()[0].0.to_str().unwrap(),
            "test/test1.jpg"
        );
        assert_eq!(
            sieve_io.renames.borrow()[0].1.to_str().unwrap(),
            "target/1970-01/test1.jpg"
        );
        assert_eq!(sieve_io.removes.borrow().len(), 1);
        assert_eq!(
            sieve_io.removes.borrow()[0].to_str().unwrap(),
            "test/test2.jpg"
        );
    }

    #[test]
    fn test_duplicate_files() {
        let item_list = ItemList {
            items: vec![
                FileItem::dummy("tests/test.jpg", 0, true),
                FileItem::dummy("tests/test2.JPG", 0, true),
                FileItem::dummy("tests/test3.jpg", 0, true),
                FileItem::dummy("tests/subdir/test.jpg", 0, true),
                FileItem::dummy("tests/subdir/test2.JPG", 0, true),
                FileItem::dummy("tests/subdir/test3.jpg", 0, true),
            ],
            events: vec![],
            path: PathBuf::from(""),
        };
        let file_io = FileSieveIO {};

        sieve(
            &item_list,
            Path::new("tests/target"),
            SieveMethod::Copy,
            DirectoryNames::YearAndMonth,
            &file_io,
            |_: String| {},
        );

        assert!(Path::new("tests/target/1970-01/test.jpg").exists());
        assert!(!Path::new("tests/target/1970-01/test_.jpg").exists());
        assert!(Path::new("tests/target/1970-01/test2.JPG").exists());
        assert!(Path::new("tests/target/1970-01/test2_.JPG").exists());
        assert!(Path::new("tests/target/1970-01/test3.jpg").exists());
        assert!(Path::new("tests/target/1970-01/test3_.jpg").exists());
    }
}
