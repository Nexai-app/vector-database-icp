extern  crate nalgebra as na;
pub mod config;
pub mod database;
pub mod management;
pub mod company;
pub mod migration;

use std::cell::RefCell;
use company::comp::{CompanyCollection, Company};
use config::EMBEDDING_LENGTH;
use database::index::Vector;
use ic_cdk::{update, query, init, post_upgrade, pre_upgrade, storage,};
use candid::{candid_method, export_service, Principal};
use instant_distance::Search;
use management::AccessControl;
use migration::migration::CompanyCollectionMigration;

thread_local! {
    static ACL: RefCell<AccessControl> = RefCell::new(AccessControl::new());

    static COMP: RefCell<CompanyCollection> = RefCell::new(CompanyCollection::new());
}

#[candid_method(init)]
#[init]
async fn init() {
    let caller = ic_cdk::caller();
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.set_owner(caller);
        acl.add_manager(caller);
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    let acl = ACL.with(|a| {
        a.replace(AccessControl::default())
    });

    let comp = COMP.with(|c| {
        c.replace(CompanyCollection::new())
    });

    let comp_migrate = CompanyCollectionMigration::from(comp);

    storage::stable_save((acl, comp_migrate,)).expect("should save acl and comp to stable storage");
}

// #[post_upgrade]
// fn post_upgrade() {
//     let (acl, comp_migrate): (AccessControl, CompanyCollectionMigration) = storage::stable_restore().expect("restore company collection and acl should work");

//     let comp: CompanyCollection = comp_migrate.into();

//     ACL.with(|a| {
//         a.replace(acl);
//     });

//     COMP.with(|c| {
//         c.replace(comp);
//     });
// }

// APIs for vector database business

/// for company user to register
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

/// get similar `limit` numbers of records([(similarity:f64, question-answer-pair:string)]) from vector database
/// or throws an error(String) 
#[candid_method(query)]
#[query]
fn get_similar(id: u32, q: Vec<f64>, limit: i32) -> Result<Vec<(f64, String)>, String> {
    if q.len() != EMBEDDING_LENGTH {
        return Err(String::from("query malformed"))
    }

    COMP.with(|comp| {
        let comps = comp.borrow();

        match comps.get(&id) {
            Some(c) => {
                let mut search = Search::default();
                let key = Vector::from(q);
                Ok(c.db.query(&key, &mut search, limit))
            },
            None => Err(String::from("No such comp"))
        }

    })
}

/// append keys(embeddings) and values(question-answer-pairs) into database
/// it either returns Ok() or throw an error(Unprivileged)
#[candid_method(update)]
#[update]
fn append_keys_values(id: u32, keys: Vec<Vec<f64>>, values: Vec<String>) -> Result<(), String> {
    // let caller = ic_cdk::caller();
    // if !caller_same_with_comp_owner(&caller, &id) && !is_manager(&caller) {
    //     return Err(String::from("caller not owner of company or not manager"))
    // }

    if keys.len() != values.len() {
        return Err(String::from("keys length is not euqal to values"));
    }


    COMP.with(|comp| {
        let mut comps = comp.borrow_mut();   
        match comps.get_mut(&id) {
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

/// build index for uploaded keys(embeddings) and values(question-answers-pairs)
/// this is done manually, and function `append_keys_values` doesn't do it automatically since the function call is expensive
#[candid_method(update)]
#[update]
fn build_index(id: u32) -> Result<(), String> {
    // let caller = ic_cdk::caller();
    // if !caller_same_with_comp_owner(&caller, &id) && !is_manager(&caller) {
    //     return Err(String::from("caller not owner of company or not manager"))
    // }

    COMP.with(|comp| {
        let mut comps = comp.borrow_mut();
        match comps.get_mut(&id) {
            Some(c) => {
                let db = &mut c.db;
                db.build_index();
                Ok(())
            },
            None => Err(String::from("No such comp"))
        }
    })
}

// Manage functions
/// add a manager
#[candid_method(update)]
#[update(guard = "only_owner")]
fn add_manager(manager: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.add_manager(manager)
    })
}

/// remove a manager
#[candid_method(update)]
#[update(guard = "only_owner")]
fn remove_manager(manager: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.remove_manager(&manager)
    })
}

/// add a accesser to allow access, only valid when vdb setting `access_list_enabled` to be true
#[candid_method(update)]
#[update(guard = "only_manager")]
fn add_accesser(accesser: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.add_accesser(accesser)
    })
}

/// remove an accesser
#[candid_method(update)]
#[update(guard = "only_manager")]
fn remove_accesser(accesser: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.remove_accesser(&accesser)
    })
}


// buggy codes
// #[update(guard = "only_manager")]
#[candid_method(query)]
#[query]
fn states() -> Option<AccessControl> {
    ACL.with(|acl| {
        let acl = acl.borrow();

        Some((*acl).clone())
    })
}

/// set flag `access_list_enabled`
#[candid_method(update)]
#[update(guard = "only_manager")]
fn set_acl_enabled(enable: bool) -> Result<(), String> {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.access_list_enabled = enable;
    });

    Ok(())
}


// Candid
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}


// Access Control helper functions
fn only_owner() -> Result<(), String> {
    let caller = ic_cdk::caller();
    ACL.with(|acl| {
        let acl = acl.borrow();

        if acl.is_owner(caller) {
            Ok(())
        } else {
            Err(String::from("Not owner"))
        }
    })
}

fn only_manager() -> Result<(), String> {
    let caller = ic_cdk::caller();
    ACL.with(|acl| {
        let acl = acl.borrow();

        if acl.allow_manage(&caller) {
            Ok(())
        } else {
            Err(String::from("Not manager"))
        }
    })
}

fn only_allowed_accesser() -> Result<(), String> {
    let caller = ic_cdk::caller();
    ACL.with(|acl| {
        let acl = acl.borrow();

        if !acl.access_list_enabled || acl.allow_access(&caller) {
            Ok(())
        } else {
            Err(String::from("Contact admin to gain access"))
        }
    })
}

fn is_manager(p: &Principal) -> bool {
    ACL.with(|acl| {
        let acl = acl.borrow();
        acl.allow_manage(p)
    })
}

fn caller_same_with_comp_owner(caller: &Principal,  comp_id: &u32) -> bool {
    let mut allow = true;

    ACL.with(|acl| {
        let acl = acl.borrow();
        allow = allow && acl.allow_access(caller)
    });

    COMP.with(|comp| {
        let comp = comp.borrow();
        match comp.get(comp_id) {
            Some(c) => {
                allow = allow && (c.owner == *caller);
            },
            None => {
                allow = false;
            }
        }
    });

    return allow;
}