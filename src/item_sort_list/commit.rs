use std::{
    fs::{copy, create_dir_all, remove_file, rename},
    io::{Error, ErrorKind},
    path::Path,
};

use chrono::NaiveDateTime;

use super::{file_item, CommitMethod, ItemList};

/// Trait to encapsulate commit file IO operations
pub trait CommitIO {
    fn copy(&self, src: &Path, dest: &Path) -> Result<(), Error>;
    fn remove_file(&self, path: &Path) -> Result<(), Error>;
    fn r#move(&self, src: &Path, dest: &Path) -> Result<(), Error>;
    fn create_dir_all(&self, path: &Path) -> Result<(), Error>;
}

/// Struct with implementation for std::fs implementation of CommitIO
pub struct FileCommitIO;

impl FileCommitIO {
    fn assert_not_exists(&self, path: &Path) -> Result<(), Error> {
        if path.exists() {
            let e = Error::new(
                ErrorKind::AlreadyExists,
                format!("Destination file already exists: {}", path.display()),
            );
            Err(e)
        } else {
            Ok(())
        }
    }
}

impl CommitIO for FileCommitIO {
    fn copy(&self, src: &Path, dest: &Path) -> Result<(), Error> {
        self.assert_not_exists(dest)?;
        copy(src, dest)?;
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), Error> {
        remove_file(path)
    }

    fn r#move(&self, src: &Path, dest: &Path) -> Result<(), Error> {
        self.assert_not_exists(dest)?;
        match rename(src, dest) {
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

/// Commits an item list taking the take_over flag into account to a new directory.
/// The progress is reported by calling a callback function with the file that is currently processed.
pub fn commit<T>(
    item_list: &ItemList,
    path: &str,
    commit_method: CommitMethod,
    commit_io: &T,
    progress_callback: impl Fn(String),
) where
    T: CommitIO,
{
    if commit_method != CommitMethod::Delete {
        let path = Path::new(path);
        prepare_path(path, commit_io);

        for item in &item_list.items {
            if item.get_take_over() {
                let full_path = path.join(get_sub_path(item_list, item));
                prepare_path(&full_path, commit_io);
                let source = item.get_path();
                let target = full_path.join(source.file_name().unwrap());
                progress_callback(format!("{:?} -> {:?}", source, target));

                if commit_method == CommitMethod::Copy {
                    match commit_io.copy(source, &target) {
                        Ok(_) => (),
                        Err(e) => progress_callback(format!("Error copying {}: {}", item, e)),
                    }
                } else {
                    match commit_io.r#move(source, &target) {
                        Ok(_) => (),
                        Err(e) => progress_callback(format!("Error moving {}: {}", item, e)),
                    }
                };
            } else if commit_method == CommitMethod::MoveAndDelete {
                let source = item.get_path();
                progress_callback(format!("Delete {:?}", source));
                match commit_io.remove_file(source) {
                    Ok(_) => (),
                    Err(e) => progress_callback(format!("Error deleting {}: {}", item, e)),
                }
            }
        }
    } else {
        for item in &item_list.items {
            if !item.get_take_over() {
                let source = item.get_path();
                progress_callback(format!("Delete {:?}", source));
                match commit_io.remove_file(source) {
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
/// If it is not part of an event, its sub path is the file item's timestamp in year-month.
fn get_sub_path(item_list: &ItemList, item: &file_item::FileItem) -> String {
    let event = item_list.get_event(item);
    if let Some(event) = event {
        if event.start_date != event.end_date {
            return format!(
                "{} - {} {}",
                event.start_date.format("%Y-%m-%d"),
                event.end_date.format("%Y-%m-%d"),
                event.name
            );
        } else {
            return format!("{} {}", event.start_date.format("%Y-%m-%d"), event.name);
        }
    }
    NaiveDateTime::from_timestamp(item.get_timestamp(), 0)
        .format("%Y-%m")
        .to_string()
}

/// Prepares the path by creating it if it does not exist
fn prepare_path<T>(path: &Path, commit_io: &T)
where
    T: CommitIO,
{
    if !path.exists() {
        match commit_io.create_dir_all(path) {
            Ok(_) => (),
            Err(e) => println!("Error creating path {}: {}", e, path.display()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::item_sort_list::commit::CommitIO;
    use crate::item_sort_list::{commit::get_sub_path, Event, FileItem, ItemList};
    use std::cell::RefCell;
    use std::path::PathBuf;

    struct TestCommitIO {
        pub copies: RefCell<Vec<(PathBuf, PathBuf)>>,
        pub renames: RefCell<Vec<(PathBuf, PathBuf)>>,
        pub removes: RefCell<Vec<PathBuf>>,
        pub creates: RefCell<Vec<PathBuf>>,
    }

    impl TestCommitIO {
        pub fn new() -> Self {
            TestCommitIO {
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

    impl CommitIO for TestCommitIO {
        fn copy(&self, src: &Path, dest: &Path) -> Result<(), Error> {
            self.copies
                .borrow_mut()
                .push((src.to_path_buf(), dest.to_path_buf()));
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<(), Error> {
            self.removes.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn r#move(&self, src: &Path, dest: &Path) -> Result<(), Error> {
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
                    start_date: NaiveDate::from_ymd(2021, 9, 14),
                    end_date: NaiveDate::from_ymd(2021, 9, 14),
                },
                Event {
                    name: String::from("Test2"),
                    start_date: NaiveDate::from_ymd(2021, 9, 20),
                    end_date: NaiveDate::from_ymd(2021, 9, 21),
                },
                Event {
                    name: String::from("Test3"),
                    start_date: NaiveDate::from_ymd(2021, 9, 24),
                    end_date: NaiveDate::from_ymd(2021, 9, 27),
                },
            ],
            path: String::from(""),
        };
        let test_cases = [
            ("2021-09-14 00:00", "2021-09-14 Test1"),
            ("2021-09-13 23:59", "2021-09"),
            ("2021-09-15 00:00", "2021-09"),
            ("2021-09-20 00:00", "2021-09-20 - 2021-09-21 Test2"),
            ("2021-09-21 16:00", "2021-09-20 - 2021-09-21 Test2"),
            ("2021-09-25 16:00", "2021-09-24 - 2021-09-27 Test3"),
        ];

        for (input, result) in test_cases {
            assert_eq!(
                get_sub_path(
                    &item_list,
                    &FileItem::dummy(
                        "",
                        NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M")
                            .unwrap()
                            .timestamp(),
                        false
                    )
                ),
                result
            );
        }
    }

    #[test]
    fn test_commit_methods() {
        let item_list = ItemList {
            items: vec![
                FileItem::dummy("test/test1", 0, true),
                FileItem::dummy("test/test2", 0, false),
            ],
            events: vec![],
            path: String::from(""),
        };
        let mut commit_io = TestCommitIO::new();

        commit(
            &item_list,
            "target",
            CommitMethod::Delete,
            &commit_io,
            |_: String| {},
        );
        assert_eq!(commit_io.copies.borrow().len(), 0);
        assert_eq!(commit_io.creates.borrow().len(), 0);
        assert_eq!(commit_io.renames.borrow().len(), 0);
        assert_eq!(commit_io.removes.borrow().len(), 1);
        assert_eq!(
            commit_io.removes.borrow()[0].to_str().unwrap(),
            "test/test2"
        );

        commit_io.reset();
        commit(
            &item_list,
            "target",
            CommitMethod::Copy,
            &commit_io,
            |_: String| {},
        );
        assert_eq!(commit_io.copies.borrow().len(), 1);
        assert_eq!(
            commit_io.copies.borrow()[0].0.to_str().unwrap(),
            "test/test1"
        );
        assert_eq!(
            commit_io.copies.borrow()[0].1.to_str().unwrap(),
            "target/1970-01/test1"
        );
        assert_eq!(commit_io.creates.borrow().len(), 1);
        assert_eq!(
            commit_io.creates.borrow()[0].to_str().unwrap(),
            "target/1970-01"
        );
        assert_eq!(commit_io.renames.borrow().len(), 0);
        assert_eq!(commit_io.removes.borrow().len(), 0);

        commit_io.reset();
        commit(
            &item_list,
            "target",
            CommitMethod::Move,
            &commit_io,
            |_: String| {},
        );
        assert_eq!(commit_io.copies.borrow().len(), 0);
        assert_eq!(commit_io.creates.borrow().len(), 1);
        assert_eq!(
            commit_io.creates.borrow()[0].to_str().unwrap(),
            "target/1970-01"
        );
        assert_eq!(commit_io.renames.borrow().len(), 1);
        assert_eq!(
            commit_io.renames.borrow()[0].0.to_str().unwrap(),
            "test/test1"
        );
        assert_eq!(
            commit_io.renames.borrow()[0].1.to_str().unwrap(),
            "target/1970-01/test1"
        );
        assert_eq!(commit_io.removes.borrow().len(), 0);

        commit_io.reset();
        commit(
            &item_list,
            "target",
            CommitMethod::MoveAndDelete,
            &commit_io,
            |_: String| {},
        );
        assert_eq!(commit_io.copies.borrow().len(), 0);
        assert_eq!(commit_io.creates.borrow().len(), 1);
        assert_eq!(
            commit_io.creates.borrow()[0].to_str().unwrap(),
            "target/1970-01"
        );
        assert_eq!(commit_io.renames.borrow().len(), 1);
        assert_eq!(
            commit_io.renames.borrow()[0].0.to_str().unwrap(),
            "test/test1"
        );
        assert_eq!(
            commit_io.renames.borrow()[0].1.to_str().unwrap(),
            "target/1970-01/test1"
        );
        assert_eq!(commit_io.removes.borrow().len(), 1);
        assert_eq!(
            commit_io.removes.borrow()[0].to_str().unwrap(),
            "test/test2"
        );
    }
}
