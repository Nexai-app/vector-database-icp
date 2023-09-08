extern  crate nalgebra as na;
pub mod config;
pub mod database;
pub mod management;
pub mod company;

use std::cell::RefCell;
use company::comp::{CompanyCollection, Company};
use config::EMBEDDING_LENGTH;
use database::{db::{Database}, index::Vector};
use ic_cdk::{update, query, init};
use candid::{candid_method, export_service, Principal};
use instant_distance::Search;
use management::{AccessControl, Permission};

thread_local! {
    static DB: RefCell<Database> = RefCell::new(Database::new(vec![], vec![]));
    static ACL: RefCell<AccessControl> = RefCell::new(AccessControl::new());

    static COMP: RefCell<CompanyCollection> = RefCell::new(CompanyCollection::new());
}

#[candid_method(update)]
#[update]
fn register(description: String) -> Result<u32, String> {
    COMP.with(|comp| {
        let mut comps = comp.borrow_mut();
        let owner = ic_cdk::caller();
        let c = Company::new(owner, description);
        Ok(comps.register(c))
    })
}

#[candid_method(query)]
#[query]
fn query(id: u32, q: Vec<f32>, limit: i32) -> Result<Vec<(f32, String)>, String> {
    if q.len() != EMBEDDING_LENGTH {
        return Err(String::from("query malformed"))
    }

    COMP.with(|comp| {
        let comps = comp.borrow();

        match comps.get(id) {
            Some(c) => {
                let mut search = Search::default();
                let key = Vector::from(q);
                Ok(c.db.query(&key, &mut search, limit))
            },
            None => Err(String::from("No such comp"))
        }

    })
}

#[candid_method(update)]
#[update]
fn append_keys_values(id: u32, keys: Vec<Vec<f32>>, values: Vec<String>) -> Result<(), String> {
    if keys.len() != values.len() {
        return Err(String::from("keys length is not euqal to values"));
    }

    COMP.with(|comp| {
        let mut comps = comp.borrow_mut();   
        match comps.get_mut(id) {
            Some(c) => {
                let db = &mut c.db;
                let mut points: Vec<Vector> = vec![];
                let mut _values: Vec<String> = vec![];

                for i in 0..keys.len() {
                    let key = &keys[i];
                    if key.len() !=  EMBEDDING_LENGTH {
                        continue;
                    }
                    let point = Vector::from((*key).clone());
                    points.push(point);
                    _values.push(values[i].clone());
                }

                db.append(&mut points, &mut _values)
            },
            None => Err(String::from("No such comp"))
        }
    })
}

#[candid_method(update)]
#[update]
fn build_index(id: u32) -> Result<(), String> {
    COMP.with(|comp| {
        let mut comps = comp.borrow_mut();
        match comps.get_mut(id) {
            Some(c) => {
                let db = &mut c.db;
                db.build_index();
                Ok(())
            },
            None => Err(String::from("No such comp"))
        }
    })
}

// Candid
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}