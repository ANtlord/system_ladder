use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::fs::MetadataExt;


fn is_user_folder(dir: &fs::DirEntry, uid: u32) -> bool {
    match dir.metadata() {
        Ok(x) => x.uid() == uid,
        Err(_) => false,
    }
}

#[derive(Debug)]
pub struct Process {
   name: String,
   pid: u32,
   ppid: u32,
}

fn to_int(val: &str) -> u32 {
    val.trim().parse().expect(&format!("Can't convert `{}` to int", val))
}

fn fill_process(p: &mut Process, data: &str) {
    let mut data_parts = data.split(':');
    if let Some(before_colon) = data_parts.next() {
        match before_colon {
            "Name" => p.name = data_parts.next().unwrap().trim().to_owned(),
            "Pid" => p.pid = to_int(data_parts.next().unwrap()),
            "PPid" => p.ppid = to_int(data_parts.next().unwrap()),
            _ => return,
        }
    }
}

impl Process {
    fn new(path: &str) -> Self {
        let file = fs::File::open(&path).expect(&format!("Can't open '{}'", path));
        let bufreader = BufReader::new(file);
        let mut p = Process{name: "".to_owned(), pid: 0, ppid: 0};
        bufreader.lines().filter_map(|x| x.ok()).for_each(|x| fill_process(&mut p, &x));
        p
    }
}

pub fn get_processes(uid: u32) {
    let proc_dir_entries = fs::read_dir("/proc/").unwrap();
    let entries = proc_dir_entries.filter_map(|x| x.ok()).filter(|x| is_user_folder(x, uid));

    for e in entries {
        println!("{}", e.path().to_str().unwrap());
    }
}

fn is_process_folder(e: &fs::DirEntry) -> bool {
    e.metadata().ok().map_or(false, |x| x.is_dir()) &&
    e.file_name().to_str().map_or(false, |x| x.parse::<u32>().is_ok())
}

pub fn get_all_processes() -> impl Iterator<Item=Process> {
    let proc_dir_entries = fs::read_dir("/proc/").unwrap();
    let process_folders = proc_dir_entries.filter_map(|x| x.ok()).filter(is_process_folder);
    process_folders.map(|x| Process::new(x.path().join("status").to_str().unwrap()))
}
