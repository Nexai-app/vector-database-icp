use std::collections::HashSet;

use candid::types::principal::Principal;

// similar to iota
pub enum Permission {
    Unprivileged = 0,
    Read, 
    Write,
    ReadWrite,
}

impl From<u32> for Permission {
    fn from(value: u32) -> Self {
        match value {
            1 => Permission::Read, 
            2 => Permission::Write,
            3 => Permission::ReadWrite,
            _ => Permission::Unprivileged,
        } 
    }
}

pub struct AccessControl {
    read: HashSet<Principal>,
    write: HashSet<Principal>,
    read_write: HashSet<Principal>,
}

impl AccessControl {
    pub fn allow_read(&self, p: &Principal) -> bool {
        self.read.contains(p) || self.read_write.contains(p)
    }

    pub fn allow_write(&self, p: &Principal) -> bool {
        self.write.contains(p) || self.read_write.contains(p)
    }

    pub fn add_record(&mut self, permission: &Permission, p: Principal) -> Result<bool, String> {
        match permission {
            Permission::Read => Ok(self.read.insert(p)),
            Permission::Write => Ok(self.write.insert(p)),
            Permission::ReadWrite => Ok(self.read_write.insert(p)),
            _ => Err(String::from("Unpriviledged"))
        }
    }

    pub fn new() -> Self {
        AccessControl { read: HashSet::new(), write: HashSet::new(), read_write: HashSet::new() }
    }
}