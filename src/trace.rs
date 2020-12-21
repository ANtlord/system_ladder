use std::fs;
use std::io;
use std::io::Write;

pub struct Trace {
    dump_to: fs::File,
}

impl Trace {
    pub fn new(path: &str) -> io::Result<Self> {
        if let Err(e) = fs::remove_file(path) {
            if let io::ErrorKind::NotFound = e.kind() {
                return Err(e);
            }
        }
        let dump_to = fs::OpenOptions::new().write(true).create(true).open(path)?;
        Ok(Self { dump_to })
    }

    pub fn object<'a>(&'a mut self, key: &str) -> Tracable<'a> {
        self.dump_to.write_all(format!("{}\n", key).as_bytes()).unwrap();
        self.dump_to.flush().unwrap();
        Tracable {
            trace: self,
        }
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        self.dump_to.write_all(format!("-> {}\n", data).as_bytes())?;
        self.dump_to.flush()
    }
}

pub struct Tracable<'a> {
    trace: &'a mut Trace,
}

impl<'a> Tracable<'a> {
    pub fn add<T: AsRef<str>>(&mut self, text: T) -> io::Result<()> {
        self.trace.write(text.as_ref())
    }
}
