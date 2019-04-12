use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Error as IOError;
use std::io::Result as IOResult;

pub struct UserData {
    pub name: String,
    pub passwd: String,
    pub user_id: u32,
    pub group_id: u32,
    pub comment: String,
    pub pw_dir: String,
    pub pw_shell: String,
}

impl UserData{
    
}

pub enum PasswdError {
    DoesNotExist,
    IO(IOError),
    IncorrectData(String),
}

type UserDataResult = Result<UserData, PasswdError>;

fn getpwent() -> Result<impl Iterator<Item=UserDataResult>, IOError> {
    let passwd_path = "/etc/passwd";
    let fd = File::open(passwd_path)?;
    let fd_reader = BufReader::new(fd);
    let file_lines = fd_reader.lines().map(parse_line);
    Ok(file_lines)
}

fn setpwent() {
}

fn endpwent() {
}

fn parse_line(line: IOResult<String>) -> UserDataResult {
    let data = line.map_err(|x| PasswdError::IO(x))?;
    let res = {
        let data: Vec<&str> = data.split(':').take(7).collect();
        parse_user_data(data)
    };
    res.ok_or_else(|| PasswdError::IncorrectData(data))
}

fn parse_user_data(data: Vec<&str>) -> Option<UserData> {
    let user_id = data[2].parse().ok()?;
    let group_id = data[3].parse().ok()?;
    Some(UserData {
        name: data[0].to_owned(),
        passwd: data[1].to_owned(),
        user_id: user_id,
        group_id: group_id,
        comment: data[4].to_owned(),
        pw_dir: data[5].to_owned(),
        pw_shell: data[6].to_owned(),
    })
}

pub fn getpwnam(name: &str) -> UserDataResult {
    for line in getpwent().map_err(|x| PasswdError::IO(x))? {
        let user = line?;
        if user.name == name {
            return Ok(user);
        }
    }
    Err(PasswdError::DoesNotExist)
}

pub fn getpwuid(id: u32) -> UserDataResult {
    for line in getpwent().map_err(|x| PasswdError::IO(x))? {
        let user: UserData = line?;
        if user.user_id == id {
            return Ok(user);
        }
    }
    Err(PasswdError::DoesNotExist)
}


#[cfg(test)]
mod tests {
    use super::*;

}
