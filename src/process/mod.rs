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
   pub name: String,
   pub pid: u32,
   pub ppid: u32,
}

fn to_int(val: &str) -> u32 {
    val.trim().parse().expect(&format!("Can't convert `{}` to int", val))
}

fn fill_process(p: &mut Process, status_file_line: &str) {
    let mut line_parts = status_file_line.split(':');
    if let Some(before_colon) = line_parts.next() {
        match before_colon {
            "Name" => p.name = line_parts.next().unwrap().trim().to_owned(),
            "Pid" => p.pid = to_int(line_parts.next().unwrap()),
            "PPid" => p.ppid = to_int(line_parts.next().unwrap()),
            _ => return,
        }
    }
}

impl Process {
    fn new(status_file_path: &str) -> Self {
        let file = fs::File::open(&status_file_path).expect(&format!("Can't open '{}'", status_file_path));
        let bufreader = BufReader::new(file);
        let mut p = Process{name: "".to_owned(), pid: 0, ppid: 0};
        bufreader.lines().filter_map(|x| x.ok()).for_each(|x| fill_process(&mut p, &x));
        p
    }
}

pub fn get_processes(uid: u32) -> impl Iterator<Item=Process> {
    let proc_dir_reading = fs::read_dir("/proc/").unwrap();
    let proc_dir_entries = proc_dir_reading.filter_map(|x| x.ok()).filter(
        move |x| is_user_folder(x, uid) && is_process_folder(x)
    );
    proc_dir_entries.map(|x| Process::new(x.path().join("status").to_str().unwrap()))
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
