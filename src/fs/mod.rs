use libc;

use std::fs;
use std::path;
use std::mem;
use std::ffi::CString;
use std::error::Error;
use std::io;
use std::os::unix::fs::MetadataExt;

static FTW_ACTIONRETVAL: Flag = 1;
static FTW_CHDIR: Flag = 2;
static FTW_DEPTH: Flag = 4;
static FTW_MOUNT: Flag = 8;
static FTW_PHYS: Flag = 16;

const DEFALT_NFTW: NftwResult = Ok(CallbackResult::Continue);

type Func = Fn(&io::Result<fs::Metadata>, &TypeFlag, &Ftw) -> NftwResult;
type NftwResult = Result<CallbackResult, NftwErr>;
type Flag = u8;

struct Ftw {
    base: u32,
    level: u32,
}

enum NftwErr {
    IO(io::Error),
    Stop,
}

impl From<io::Error> for NftwErr {
    fn from(e: io::Error) -> Self {
        NftwErr::IO(e)
    }
}

enum TypeFlag {
    File, // FTW_F
    Directory, // FTW_D
    UnreadableDirectory, // FTW_DNR
    PostReadableDirectory, // FTW_DP
    MetadataFailed, // FTW_NS
    Symlink, // FTW_SL
    SymlinkAbsentFile, // FTW_SLN
}

enum CallbackResult {
    Continue,
    SkipSiblings,
    SkipSubTree,
}

fn nftw(dirname: &str, action: &Func, nopenfd: i32, flags: Flag) -> NftwResult
{
    let mut ftw = Ftw{base: 0, level: 0};
    _nftw(path::Path::new(dirname), action, nopenfd, flags, &mut ftw)
}

fn metadata_ok(
    meta_ok: fs::Metadata,
    from_path: &path::Path,
    flags: Flag
) -> (io::Result<fs::Metadata>, TypeFlag) {
    let flag = if meta_ok.is_file() {
        TypeFlag::File
    } else if meta_ok.is_dir() {
        match from_path.read_dir() {
            Ok(_) if flags & FTW_DEPTH != 0 => TypeFlag::PostReadableDirectory,
            Ok(_) => TypeFlag::Directory,
            Err(_) => TypeFlag::UnreadableDirectory,
        }
    } else {
        TypeFlag::File
    };
    (Ok(meta_ok), flag)
}

fn metadata_err(
    from_path: &path::Path,
    flags: Flag,
) -> (io::Result<fs::Metadata>, TypeFlag) {
    let meta = from_path.symlink_metadata();
    let flag = match meta {
        Ok(_) if flags & FTW_PHYS != 0 => TypeFlag::Symlink,
        Ok(_) => TypeFlag::SymlinkAbsentFile,
        Err(_) => TypeFlag::MetadataFailed,
    };
    (meta, flag)
}

fn resolve_meta_and_type_flag(
    from_path: &path::Path, flags: Flag,
) -> (io::Result<fs::Metadata>, TypeFlag) {
    match from_path.metadata() {
        Ok(x) => metadata_ok(x, from_path, flags),
        Err(_) => metadata_err(from_path, flags),
    }
}

fn read_dir(from_path: &path::Path, action: &Func, nopenfd: i32, flags: Flag, ftw: &mut Ftw) -> NftwResult {
    for entry in from_path.read_dir()? {
        let path = entry?.path().to_owned();

        ftw.level += 1;
        let res = _nftw(&path, action, nopenfd, flags, ftw)?;
        ftw.level -= 1;

        match res {
            CallbackResult::SkipSiblings => break,
            _ => continue,
        }
    }
    DEFALT_NFTW
}

fn normalize(res: CallbackResult, flags: Flag) -> NftwResult {
    if flags | FTW_ACTIONRETVAL == 0 {
        match res {
            CallbackResult::Continue => Ok(res),
            _ => Err(NftwErr::Stop),
        }
    } else {
        Ok(res)
    }
}

fn handle_dir_content(
    current_item_path: &path::Path,
    action: &Func,
    nopenfd: i32,
    flags: Flag,
    ftw: &mut Ftw,
    type_flag: &TypeFlag,
) -> NftwResult {
    match type_flag {
        TypeFlag::SymlinkAbsentFile => {
            let path_buf = fs::read_link(current_item_path)?;
            if path_buf.metadata()?.is_dir() {
                read_dir(path_buf.as_path(), action, nopenfd, flags, ftw)?;
            }
            DEFALT_NFTW
        },
        TypeFlag::Directory | TypeFlag::PostReadableDirectory => read_dir(
            current_item_path, action, nopenfd, flags, ftw
        ),
        _ => DEFALT_NFTW,
    }
}

fn _nftw(current_item_path: &path::Path, action: &Func, nopenfd: i32, flags: Flag, ftw: &mut Ftw) -> NftwResult {
    let (out_meta, type_flag) = resolve_meta_and_type_flag(current_item_path, flags);
    let callback_result = if FTW_DEPTH & flags == 0 {
        action(&out_meta, &type_flag, ftw).and_then(|x| normalize(x, flags))?
    } else {
        CallbackResult::Continue
    };

    match callback_result {
        CallbackResult::SkipSubTree => return Ok(CallbackResult::Continue),
        _ => (),
    }

    if out_meta.is_ok() {
        handle_dir_content(current_item_path, action, nopenfd, flags, ftw, &type_flag)?;
    }

    // read_entry(&out_meta, &current_item_path, flags, &type_flag);
    if FTW_DEPTH & flags != 0 {
        action(&out_meta, &type_flag, ftw).and_then(|x| normalize(x, flags))
    } else {
        Ok(callback_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir {
        path: String,
    }

    enum TempPath {
        String(String),
        Pathbuf(path::PathBuf),
    }

    impl TempDir {
        fn new(path: TempPath) -> Self {
            let path = match path {
                TempPath::String(x) => x,
                TempPath::Pathbuf(x) => x.to_str().unwrap().to_owned(),
            };
            fs::create_dir(&path).unwrap_or_else(
                |e| if e.kind() != io::ErrorKind::AlreadyExists {
                    panic!(e);
                }
            );
            Self{path: path.to_owned()}
        }

        fn path_buf(&self) -> path::PathBuf {
            path::Path::new(&self.path).to_path_buf()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            fs::remove_dir(&self.path).unwrap();
        }
    }

    #[test]
    fn test_pathbuf() {
        let root = TempDir::new(TempPath::String("/tmp/nftw_test".to_owned()));
        let mut deep1_pathbuf = root.path_buf();
        deep1_pathbuf.push("deep1");
        let deep1 = TempDir::new(TempPath::Pathbuf(deep1_pathbuf));
    }

    // #[test]
    // fn test_nftw() {
    //     let dir_path = path::Path::new("test/right");
    //     match dir_path.read_dir() {
    //         Ok(x) => println!("success read"),
    //         Err(x) => println!("fail read"),
    //     }

    //     let dir_path = path::Path::new("test/wrong");
    //     match dir_path.read_dir() {
    //         Ok(x) => println!("success read"),
    //         Err(x) => println!("fail read"),
    //     }

    //     let dir_path = path::Path::new("test/link");
    //     match dir_path.metadata() {
    //         Ok(x) => println!("success read"),
    //         Err(x) => println!("fail read. {}", match x.kind() {
    //             io::ErrorKind::NotFound => "Not found",
    //             _ => "other",
    //         })
    //     }

    //     let dir_path = path::Path::new("test/file");
    //     match dir_path.metadata() {
    //         Ok(x) => println!("success read"),
    //         Err(x) => println!("fail read"),
    //     }
    // }
}
