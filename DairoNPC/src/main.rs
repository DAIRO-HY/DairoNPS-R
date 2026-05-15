#![allow(warnings)]

fn main() {

    println!("{}", lib_npc::application::NPC_CONNECT_MSG.lock().unwrap());
    println!("{}", lib_npc::application::NPC_CONNECT_MSG.lock().unwrap());

    lib_npc::start(None);
}
