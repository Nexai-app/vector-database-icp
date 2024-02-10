use std::{collections::{HashMap, BTreeMap}, hash::Hash};

use candid::{Principal, CandidType, Deserialize};
use nalgebra::{SVector, DVector};

use crate::database::{db::Database, index::{Vector, generate_index}};
use crate::company::comp::{CompanyCollection, Company};
use crate::message::msg::{Msg, Message, MessageEntry, ConnectionEntry};
use crate::config::EMBEDDING_LENGTH;


#[derive(CandidType, Clone, Deserialize)]
pub struct DatabaseMigration {
    pub keys: Vec<Vec<f64>>,
    pub values: Vec<String>
}

impl From<Database> for DatabaseMigration {
    fn from(value: Database) -> Self {
        let mut keys = vec![];
        let mut values = vec![];

        // copy vectors
        for k in value.keys {
            let mut tmp: Vec<f64> = vec![];
            for i in k.data.iter() {
                tmp.push(*i);
            }

            keys.push(tmp);
        } 

        for v in value.values {
            values.push(v);
        }

        Self { keys, values }
    }
}

impl Into<Database> for DatabaseMigration {
    fn into(self) -> Database {
        let mut keys = vec![];
        let mut values = self.values.clone();

        for k in self.keys {
            let tmp = Vector::from(k);
        }

        Database::new(keys, values)
    }
}

#[derive(CandidType, Clone, Deserialize)]
pub struct CompanyMigration {
    pub owner: Principal,
    pub description: String,
    pub db: DatabaseMigration,
}

impl From<Company> for CompanyMigration {
    fn from(value: Company) -> Self {
        let owner = value.owner;
        let description = value.description;
        let db = DatabaseMigration::from(value.db); 

        Self { owner, description, db }
    }
}

impl Into<Company> for CompanyMigration {
    fn into(self) -> Company {
        let owner = self.owner;
        let description = self.description;
        let db = self.db.into();
        Company { owner, description, db }
    }
}

// For stabilizing the messaging feat in the vdb canister
// #[derive(Clone, Deserialize)]
// pub struct MessageCollectionMigration {
//     pub connection_id: usize,
//     pub message_id: usize,
//     pub messages: Vec<Message>,
//     pub connection_hash_map: HashMap<usize, ConnectionEntry>,
//     pub message_hash_map: HashMap<usize, MessageEntry>,
// }

// impl From<MessageCollection> for MessageCollectionMigration {
//     fn from(value : MesageCollection) -> Self {
//         connection_id = value.connection_id;
//         message_id = value.message_id;
//         messages  Vec<Message>,
//         connection_hash_map: HashMap<usize, ConnectionEntry>,
//         message_hash_map: HashMap<usize, MessageEntry>, 
//     }
// }

#[derive(CandidType, Clone, Deserialize)]
pub struct CompanyCollectionMigration {
    owner_mapping: HashMap<Principal, u32>,
    companies: BTreeMap<u32, CompanyMigration>,
    pub counter: u32,
}

impl From<CompanyCollection> for CompanyCollectionMigration {
    fn from(value: CompanyCollection) -> Self {
        let mut owner_mapping = HashMap::new(); 
        let mut companies = BTreeMap::new();
        let counter = value.counter;

        for (owner, id) in value.owner_mapping {
            owner_mapping.insert(owner, id);
        }

        for (id, comp) in value.companies {
            companies.insert(id, CompanyMigration::from(comp));
        }

        Self { owner_mapping, companies, counter }

    }
}

impl Into<CompanyCollection> for CompanyCollectionMigration {
    fn into(self) -> CompanyCollection {
        let owner_mapping = self.owner_mapping.clone();
        let mut companies = BTreeMap::new();
        let counter = self.counter;

        for (id, comp) in self.companies {
            companies.insert(id, comp.into());
        }

        CompanyCollection { owner_mapping, companies, counter}
    }
}