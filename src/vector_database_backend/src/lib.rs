extern crate nalgebra as na;
pub mod company;
pub mod config;
pub mod database;
pub mod management;
pub mod message;
pub mod migration;

use std::time::{Duration, Instant, SystemTime};
use std::thread::sleep;
use std::cell::RefCell;
use company::comp::{CompanyCollection, Company};
use message::msg::{Msg, MessageEntry, ConnectionEntry};
use config::EMBEDDING_LENGTH;
use database::index::Vector;
use ic_cdk::storage;
use ic_cdk_macros::{init, pre_upgrade, query, update};
use instant_distance::Search;
use management::AccessControl;
use message::msg::{ConnectionEntry, MessageEntry, Msg};
use migration::migration::CompanyCollectionMigration;
use std::cell::RefCell;

////////////////////OPENAI/////////////////////////
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
};
// use ic_cdk_macros::{self};
use openai::chat_completion::{self, ChatCompletionRequest};
use openai::common::GPT3_5_TURBO;
///////////////////////////////////////////////////
mod openai;

thread_local! {
    static ACL: RefCell<AccessControl> = RefCell::new(AccessControl::new());
    static MSG : RefCell<Msg> = RefCell::new(Msg::new());
    static COMP: RefCell<CompanyCollection> = RefCell::new(CompanyCollection::new());
}

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
    let acl = ACL.with(|a| a.replace(AccessControl::default()));

    let comp = COMP.with(|c| c.replace(CompanyCollection::new()));

    let comp_migrate = CompanyCollectionMigration::from(comp);

    storage::stable_save((acl, comp_migrate)).expect("should save acl and comp to stable storage");
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

#[update]
fn register(description: String) -> Result<u32, String> {
    COMP.with(|comp| {
        let mut comps = comp.borrow_mut();
        let owner = ic_cdk::caller();
        let c = Company::new(owner, description);
        Ok(comps.register(c))
    })
}

#[candid_method(update)]
#[update]
fn send_message(account : String, body : String, time : i64) -> Result<Option<()>, String> {
    let caller : Principal = ic_cdk::caller();
    let now = Instant::now();
    let now_now = SystemTime::now().clone();
    MSG.with(|msg| {
        let mut msg = msg.borrow_mut();
        let main_caller = caller.to_text();
        // Ok(msg.send_message(account, main_caller, body, time))
        msg.send_message(account.clone(), main_caller.clone(), body.clone(), time)
            .map(|_| Some(()))
            .ok_or_else(|| "Something went wrong, Failed to send message".to_string())
    })
}




#[candid_method(query)]
#[query]
fn get_messages(account: String) -> Vec<MessageEntry> {
    let caller = ic_cdk::caller();
    MSG.with(|msg| {
        let msg = msg.borrow();
        return msg.get_messages(account, caller.to_text());
    })
}

#[candid_method(update)]
#[update]
async fn set_connection_completed(connection_id : usize) -> bool {
    // let caller = ic_cdk::caller();
    MSG.with(|msg| {
        return msg.borrow_mut().set_connection_completed(connection_id)
    })
}

#[candid_method(query)]
#[query]
async fn is_complete(account : String) -> bool {
    let caller = ic_cdk::caller();
    MSG.with(|msg| {
        return msg.borrow().is_complete(caller.to_text(), account)
    })
}

#[query]
async fn get_all_connections(caller: String) -> Vec<ConnectionEntry> {
    MSG.with(|msg| msg.borrow().get_all_connections(caller))
}

/// get similar `limit` numbers of records([(similarity:f64, question-answer-pair:string)]) from vector database
/// or throws an error(String)
#[query]
fn get_similar(id: u32, raw_q: String, q: Vec<f64>, limit: i32) -> Result<String, String> {
    let mut result = String::from("");
    if q.len() != EMBEDDING_LENGTH {
        return Err(String::from("query malformed"));
    }

    COMP.with(|comp| {
        let comps = comp.borrow();

        match comps.get(&id) {
            Some(c) => {
                let mut search = Search::default();
                let key = Vector::from(q);
                // Ok(c.db.query(&key, &mut search, limit))
                let res = c.db.query(&key, &mut search, limit);
                if let Some(first_element) = res.get(0) {
                    let correctness = first_element.0;
                    // let context = String::from(&finfirst_element.1);
                    if correctness < 0.6 {
                        result = format!("hh");
                        // Ok(result);
                        // Ok(String::from("bla bla bal"));
                    }
                    else {
                            /// if corredtness better
                            /// chatgpt comes it
                            /// and 
                        let qa = ("Hi, This is the question: {} \n This is the answer: {}", raw_q);
                        let template =  String::from("find a solution to this question ");
                        // println!("the Question is {}", raw_q);
                        // println!("the Answer is {}", context);
                        println!("the Template is {}", template);
                        result = format!("hh");
                    }

                    // let content =  "Hi chatgpt i want to do something for me" + " Here is the dodcunsas";
                    Ok::<std::string::String, String>(result);
                } else {
                    Ok::<std::string::String, String>(result);
                }

                Ok(String::from("No value for your question"))
            }
            None => Err(String::from("No such comp")),
        }
    })
}

/// append keys(embeddings) and values(question-answer-pairs) into database
/// it either returns Ok() or throw an error(Unprivileged)
#[update]
fn append_keys_values(id: u32, keys: Vec<Vec<f64>>, values: Vec<String>) -> Result<(), String> {
    // let caller = ic_cdk::caller();
    // if !caller_same_with_comp_owner(&caller, &id) && !is_manager(&caller) {
    //     return Err(String::from("caller not owner of company or not manager"))
    // }

    if keys.len() != values.len() {
        return Err(String::from("keys length is not euqal to values"));
    }

    COMP.with(|comp: &RefCell<CompanyCollection>| {
        let mut comps = comp.borrow_mut();
        match comps.get_mut(&id) {
            Some(c) => {
                let db = &mut c.db;
                let mut points: Vec<Vector> = vec![];
                let mut _values: Vec<String> = vec![];

                for i in 0..keys.len() {
                    let key = &keys[i];
                    if key.len() != EMBEDDING_LENGTH {
                        continue;
                    }
                    let point = Vector::from((*key).clone());
                    points.push(point);
                    _values.push(values[i].clone());
                }

                db.append(&mut points, &mut _values)
            }
            None => Err(String::from("No such comp")),
        }
    })
}

/// build index for uploaded keys(embeddings) and values(question-answers-pairs)
/// this is done manually, and function `append_keys_values` doesn't do it automatically since the function call is expensive
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
            }
            None => Err(String::from("No such comp")),
        }
    })
}

// Manage functions
/// add a manager
#[update(guard = "only_owner")]
fn add_manager(manager: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.add_manager(manager)
    })
}

/// remove a manager
#[update(guard = "only_owner")]
fn remove_manager(manager: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.remove_manager(&manager)
    })
}

/// add a accesser to allow access, only valid when vdb setting `access_list_enabled` to be true
#[update(guard = "only_manager")]
fn add_accesser(accesser: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.add_accesser(accesser)
    })
}

/// remove an accesser
#[update(guard = "only_manager")]
fn remove_accesser(accesser: Principal) -> bool {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.remove_accesser(&accesser)
    })
}

// buggy codes
// #[update(guard = "only_manager")]

#[query]
fn states() -> Option<AccessControl> {
    ACL.with(|acl| {
        let acl = acl.borrow();

        Some((*acl).clone())
    })
}

/// set flag `access_list_enabled`
#[update(guard = "only_manager")]
fn set_acl_enabled(enable: bool) -> Result<(), String> {
    ACL.with(|acl| {
        let mut acl = acl.borrow_mut();
        acl.access_list_enabled = enable;
    });

    Ok(())
}

//////////////////////OPENAI//////////////////////////
#[update]
async fn hello_openai(prompt : String) -> Result<String, String> {
    //2. SETUP ARGUMENTS FOR HTTP GET request

    // 2.1 Setup the URL

    let url = "https://api.openai.com/v1/chat/completions";
    let api_key = "sk-xxx";

    let request_headers = vec![
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", api_key).to_string(),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from(prompt)),
            name: None,
        }],
    );
    let json_string = serde_json::to_string(&req).unwrap();

    let json_utf8: Vec<u8> = json_string.into_bytes();
    let request_body: Option<Vec<u8>> = Some(json_utf8);

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        max_response_bytes: None, //optional for request
        method: HttpMethod::POST,
        headers: request_headers,
        body: request_body,
        transform: None,
        // transform: None, //optional for request
    };

    let result: Result<String, String> = match http_request(request, 30_603_148_400).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).expect("Transformed response is not UTF-8 encoded.");
            ic_cdk::api::print(format!("{:?}", str_body));
            if (200u32..=299u32).contains(&response.status) {
                let result: String = format!("{}. See more info of the request sent at: {}/inspect", str_body, url);

                Ok(result)
            } else {
                Err(format!("{}: {}", response.status, str_body))
            }
        }
        Err((r, m)) => {
            let message = format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");

            Err(message)
        }
    };

    result
}

// Strips all data that is not needed from the original response.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == 200_u8 {
        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", raw));
    }
    res
}
///////////////////////////////////////////////////

ic_cdk_macros::export_candid!();

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

fn caller_same_with_comp_owner(caller: &Principal, comp_id: &u32) -> bool {
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
            }
            None => {
                allow = false;
            }
        }
    });

    return allow;
}
