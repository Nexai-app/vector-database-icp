use std::{collections::HashSet};

use candid::{types::principal::Principal, CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub struct AccessControl {
    owner: Principal, 
    pub access_list_enabled: bool,
    pub managers: HashSet<Principal>,
    pub accessers: HashSet<Principal>,
}

impl Default for AccessControl {
    fn default() -> Self {
        AccessControl {
            owner: Principal::anonymous(),
            access_list_enabled: true,
            managers: HashSet::default(),
            accessers: HashSet::default()
        } 
    }
}

impl AccessControl {
    pub fn new() -> Self {
        AccessControl { owner: Principal::anonymous(), access_list_enabled: true, managers: HashSet::new(), accessers: HashSet::new() }
    }

    pub fn set_owner(&mut self, owner: Principal) {
        self.owner = owner;
    }

    pub fn is_owner(&self, p: Principal) -> bool {
        self.owner == p
    }

    pub fn add_manager(&mut self, manager: Principal) -> bool  {
        self.managers.insert(manager)
    }

    pub fn add_accesser(&mut self, accesser: Principal) -> bool {
        self.accessers.insert(accesser)
    }

    pub fn allow_access(&self, p: &Principal) -> bool {
        self.accessers.contains(p)
    }
    
    pub fn allow_manage(&self, p: &Principal) -> bool {
        self.managers.contains(p)
    }

    pub fn enable_access_list(&mut self) {
        self.access_list_enabled = true;
    }

    pub fn disable_access_list(&mut self) {
        self.access_list_enabled = false;
    }

    pub fn remove_manager(&mut self, manager: &Principal) -> bool {
        self.managers.remove(manager)
    }

    pub fn remove_accesser(&mut self, accesser: &Principal) -> bool {
        self.accessers.remove(accesser)
    }

}