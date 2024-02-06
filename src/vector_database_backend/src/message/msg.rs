use std::collections::{BTreeMap, BTreeSet};

use candid::{CandidType, Decode, Encode, Principal, Deserialize};
use serde::Serialize; 
use std::cell::RefCell;

use ic_cdk::{update, query};

type MessageMap = BTreeMap<u8, Message>;
type ConnectionMap = BTreeMap<u8, Connection>;

thread_local!{
    static Message_Map : RefCell<MessageMap> = RefCell::default();
    static Connection_Map : RefCell<ConnectionMap> = RefCell::default();
}

#[derive(Clone, Debug, Default, CandidType, Serialize, Deserialize)]
// pub struct Message {
//     pub id : u128,
//     pub customer : String, // Principal type
//     pub company : String,
//     pub body : String,
//     pub time : u8,
// }

// #[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Connection {
    pub id : u128,
    pub account1 : String, // type Principal
    pub account2 : String, // type Principal
    pub created_at : u8,
    
}

use std::collections::HashMap;

pub struct Message {
    id: usize,
    customer: String, // Assuming Principal is a string in Motoko
    body: String,
    company: String, // Assuming Principal is a string in Motoko
    time: i64,
}

struct Conversation {
    conversation_id: Option<String>, // Assuming Text is an optional string in Motoko
    messages: Vec<Message>,
}

#[derive(Clone, CandidType)]
pub struct ConnectionEntry {
    id: usize,
    account1: String, // Assuming Principal is a string in Motoko
    account2: String, // Assuming Principal is a string in Motoko
    created_at: i64,
    completed : bool,
}

#[derive(CandidType)]
struct CompanyEntry {
    name : String,
    email : String,
    description : String,
    vdbId : u32,
    createdAt : i8,
}

#[derive(CandidType, Clone)]
pub struct MessageEntry {
    id: usize,
    connection_id: usize,
    sender: String, // Assuming Principal is a string in Motoko
    body: String,
    created_at: i64,
}

// #[derive(Deserialize)]
pub struct Msg {
    pub connection_id: usize,
    pub message_id: usize,
    // company_entries: Vec<(String, CompanyEntry)>,
    // conversation_entries: Vec<(String, Conversation)>, 
    // connection_entries: Vec<(usize, ConnectionEntry)>,
    // message_entries: Vec<(usize, MessageEntry)>,
    pub messages: Vec<Message>,
    pub connection_hash_map: HashMap<usize, ConnectionEntry>,
    pub message_hash_map: HashMap<usize, MessageEntry>,
}


impl Default for Msg {
    fn default() -> Self {
        Msg {
            connection_id : 0,
            message_id : 0,
            messages : Vec::default(),
            connection_hash_map : HashMap::default(),
            message_hash_map : HashMap::default()
        }
    }
}

impl Msg {

    pub fn new() -> Self {
        Msg {
            connection_id : 0,
            message_id : 0,
            messages : Vec::new(),
            connection_hash_map : HashMap::new(),
            message_hash_map : HashMap::new()
        }
    }

    pub fn send_message(&mut self, account: String, caller: String, body: String, time: i64) -> Option<()> {
        let mut sent = false;
        let size = self.connection_hash_map.len();

        if size == 0 {
            let new_connection = ConnectionEntry {
                id: self.connection_id,
                account1: account.clone(),
                account2: caller.clone(),
                created_at: time,
                completed : false,
            };
            self.connection_hash_map.insert(self.connection_id, new_connection.clone());
            self.connection_id += 1;

            let new_message = MessageEntry {
                id: self.message_id,
                connection_id: new_connection.id,
                sender: caller.clone(),
                body,
                created_at: time,
            };
            self.message_hash_map.insert(self.message_id, new_message);
            self.message_id += 1;

            sent = true;
        } else {
            let mut found_connection = None;
            for (_, j) in self.connection_hash_map.iter() {
                if (j.account1 == caller && j.account2 == account) || (j.account1 == account && j.account2 == caller) {
                    found_connection = Some(j.clone());
                    break;
                }
            }

            if let Some(connection) = found_connection {
                let new_message = MessageEntry {
                    id: self.message_id,
                    connection_id: connection.id,
                    sender: caller.clone(),
                    body,
                    created_at: time,
                };
                self.message_hash_map.insert(self.message_id, new_message);
                self.message_id += 1;
                sent = true;
            } else {
                let new_connection = ConnectionEntry {
                    id: self.connection_id,
                    account1: account.clone(),
                    account2: caller.clone(),
                    created_at: time,
                    completed : false,
                };
                self.connection_hash_map.insert(self.connection_id, new_connection.clone());
                self.connection_id += 1;

                let new_message = MessageEntry {
                    id: self.message_id,
                    connection_id: new_connection.id,
                    sender: caller.clone(),
                    body,
                    created_at: time,
                };
                self.message_hash_map.insert(self.message_id, new_message);
                self.message_id += 1;
            }
        }

        Some(())
    }
    

    pub fn get_messages(&self, account: String, caller : String) -> Vec<MessageEntry> {
        let mut msgs = Vec::new();
        for (_, j) in self.connection_hash_map.iter() {
            if (j.account1 == caller && j.account2 == account) || (j.account1 == account && j.account2 == caller) {
                for (_, l) in self.message_hash_map.iter() {
                    if l.connection_id == j.id {
                        msgs.push(l.clone());
                    }
                }
            }
        }
        msgs
    }


    pub fn check_connection(&self, account: String, caller : String) -> bool {
        for (_, j) in self.connection_hash_map.iter() {
            if (j.account1 == caller && j.account2 == account) || (j.account1 == account && j.account2 == caller) {
                return true;
            }
        }
        false
    }

    pub fn set_connection_completed(&mut self, connection_id: usize) -> bool {
        if let Some(connection) = self.connection_hash_map.get_mut(&connection_id) {
            connection.completed = true;
            return true;
        } else {
            return false; 
        }
    }


    pub fn is_complete(&self, caller : String, account : String) -> bool {
        for (_, j) in self.connection_hash_map.iter(){
            if (j.account1 == caller && j.account2 == account) || (j.account1 == account && j.account2 == caller) {
                return j.completed;
            }
        }
        false
    }

    // update func 

    pub fn get_all_connections(&self, caller : String) -> Vec<ConnectionEntry> {
        let mut buff = Vec::new();
        for (_, j) in self.connection_hash_map.iter() {
            if j.account1 == caller || j.account2 == caller {
                buff.push(j.clone());
            }
        }
        buff
    }

}


// impl Connection {

//     #[query]
//     pub fn get_all_connection() -> Vec<ConnectionMap> {
//         Connection_Map.with(|connection_map| {
//             let mut all_connections = Vec::new();
//             connection_map.borrow().iter().for_each(|connection| {
//                 all_connections.push((*connection.1).clone().try_into().unwrap())
//             });
//             return all_connections;
//         })
//     }

//     #[query]
//     pub fn get_message(name: String) -> UserProfile {
//         Message_Map.with(|message_map| {
//             Message_Map.with(|message_map| {
//                 message_map
//                     .borrow()
//                     .get(&id)
//                     .and_then(|id| message_map.borrow().get(id).cloned()).unwrap()
//             })
//         })
//     }

//     // fn checkConnection()


// }


