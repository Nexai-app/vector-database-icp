extern  crate nalgebra as na;
pub mod config;
pub mod database;
pub mod management;

use std::cell::RefCell;
use config::EMBEDDING_LENGTH;
use database::{db::{Database}, index::Vector};
use ic_cdk::{update, query, init};
use candid::{candid_method, export_service, Principal};
use instant_distance::Search;
use management::{AccessControl, Permission};

thread_local! {
    static DB: RefCell<Database> = RefCell::new(Database::new(vec![], vec![]));
    static ACL: RefCell<AccessControl> = RefCell::new(AccessControl::new());
}

#[candid_method(init)]
#[init]
fn init() {
    let manager = ic_cdk::caller();
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        let _ = acl.add_record(&Permission::ReadWrite, manager);
    })
}

#[candid_method(query)]
#[update]
fn query(q: Vec<f32>, limit: i32) -> Result<Vec<(f32, String)>, String> {
    if q.len() != EMBEDDING_LENGTH {
        return Err(String::from("query malformed"))
    }

    let res = DB.with(|db| {
        let db = db.borrow();
        let mut search = Search::default();
        let v = Vector::from(q);

        db.query(&v, &mut search, limit)
    });


    Ok(res)
}

#[candid_method(update)]
#[update]
fn append_keys_values(keys: Vec<Vec<f32>>, values: Vec<String>) -> Result<(), String> {
    if keys.len() != values.len() {
        return Err(String::from("keys length is not euqal to values"));
    }

    let res = DB.with(|db| {
        let mut db = db.borrow_mut();
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
    });

    res
}

#[candid_method(update)]
#[update]
fn build_index() -> Result<(), String> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        db.build_index()
    });

    Ok(())
}

#[candid_method(update)]
#[update]
fn add_acl_record(permission: u32, p: Principal) -> Result<bool, String> {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.add_record(&permission.into(), p)
    })
}

// ACL
fn allow_read() -> Result<(), String> {
    let user = ic_cdk::api::caller();
    ACL.with(|acl| {
        let acl = acl.borrow();
        if acl.allow_read(&user) { Ok(()) }
        else { Err(String::from("No priviledge to read")) } 
    })
}

fn allow_write() -> Result<(), String> {
    let user = ic_cdk::api::caller();
    ACL.with(|acl| {
        let acl = acl.borrow();
        if acl.allow_write(&user) { Ok(()) }
        else { Err(String::from("No priviledge to write")) } 
    })
}

// Candid
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}