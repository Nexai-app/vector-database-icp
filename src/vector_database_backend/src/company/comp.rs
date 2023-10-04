use std::{collections::{BTreeMap, HashMap}, hash::Hash};

use candid::{Principal, CandidType, };

use crate::database::db::Database;

pub struct Company {
    pub owner: Principal,
    pub description: String,
    pub db: Database
}

impl Company {
    pub fn new(owner: Principal, description: String) -> Self {
        Company { owner, description, db: Database::new(vec![], vec![]) } 
    }

    pub fn update_description(&mut self, description: String) {
        self.description = description
    }
}

pub struct CompanyCollection {
    pub owner_mapping: HashMap<Principal, u32>,
    pub companies: BTreeMap<u32, Company>,
    pub counter: u32,
}

impl CompanyCollection {
    pub fn new() -> Self {
        CompanyCollection { owner_mapping: HashMap::new(), companies: BTreeMap::default(), counter: 0 }
    }

    pub fn register(&mut self, comp: Company) -> u32 {
        self.owner_mapping.insert(comp.owner.clone(), self.counter-1);
        self.companies.insert(self.counter, comp);
        self.counter += 1;

        self.counter - 1 
    }

    pub fn get(&self, id: &u32) -> Option<&Company> {
        self.companies.get(id)
    }

    pub fn get_mut(&mut self, id: &u32) -> Option<&mut Company> {
        self.companies.get_mut(id)
    }

    pub fn get_id_by_principal(&self, p: &Principal) -> Option<&u32> {
        self.owner_mapping.get(p)
    }
}