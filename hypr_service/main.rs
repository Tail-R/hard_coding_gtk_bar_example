use std::env;
use std::process::Command;
use std::path::Path;
use std::io::Read;
use std::os::unix::net::UnixStream;

// use std::thread::sleep;
// use std::time::Duration;

use regex::Regex;

use serde::{
    Serialize,
    Deserialize
};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct Activeworkspace {
    id: i32,
    name: String,
    monitor: String,
    monitorID: i32,
    windows: i32,
    hasfullscreen: bool,
    lastwindow: String,
    lastwindowtitle: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    id: i32,
    name: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct Client {
    address: String,
    mapped: bool,
    hidden: bool,
    at: Vec<i32>,
    size: Vec<i32>,
    workspace: Workspace,
    floating: bool,
    monitor: i32,
    class: String,
    title: String,
    initialClass: String,
    initialTitle: String,
    pid: i32,
    xwayland: bool,
    pinned: bool,
    fullscreen: bool,
    fullscreenMode: i32,
    fakeFullscreen: bool,
    grouped: Vec<String>,
    swallowing: String,
    focusHistoryID: i32
}

#[derive(Debug)]
enum State {
    Active,
    Inactive,
    Occupied,
    Dead
}

#[derive(Debug)]
struct States {
    p_active_ws_id: i32,
    data: Vec<State>  
}
 
/*
spawn hyprctl as a child process. then
parse the result and convert it into an array.
*/
fn get_ws_states() -> States {
    let mut h0 = Command::new("hyprctl");
    let mut h1 = Command::new("hyprctl");
    
    let get_clients = h0
        .arg("-j")
        .arg("clients");

    let get_activeworkspace = h1
        .arg("-j")
        .arg("activeworkspace");
    
    let stdout0 = get_clients
        .output()
        .expect("failed to execute command")
        .stdout;

    let stdout1 = get_activeworkspace
        .output()
        .expect("failed to execute command")
        .stdout;

    let raw_json0 = String::from_utf8_lossy(&stdout0);
    let raw_json1 = String::from_utf8_lossy(&stdout1);

    let clients: Vec<Client> = serde_json::from_str(&raw_json0)
        .expect("failed to deserialize");

    let activeworkspace: Activeworkspace = serde_json::from_str(&raw_json1)
        .expect("failed to deserialize");

    let mut elems = vec![];

    // hyprland has 10 workspaces by default
    // head one is a dead variable 
    elems.push(State::Dead); 
    
    for _ in 1..=10 {
        elems.push(State::Inactive);
    }

    let mut ws_states = States{
        p_active_ws_id: activeworkspace.id,
        data: elems
    };

    for c in clients {
        let id = c.workspace.id;
        
        if id > 0 {
            ws_states.data[id as usize] = State::Occupied;
        }
    }

    ws_states.data[activeworkspace.id as usize] = State::Active;    
 
    ws_states
}

fn hypr_ws_daemon() {
    let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("failed to get hyprland instance signature");
    
    let sock_file = "/tmp/hypr/".to_string()
        + &signature
        + "/.socket2.sock"; // socket2.sock is a hyprland event socket

    let s = Path::new(&sock_file);

    let mut socket = match UnixStream::connect(s) {
        Ok(sock) => sock,
        Err(e) => {
            println!("couldn't connect: {e:?}");
            return
        }
    };
 
    let re_ws = Regex::new(r"^workspace>>[0-9]+").unwrap();
    // let re_cws = Regex::new(r"(^createworkspace>>|\ncreateworkspace)").unwrap();
    let re_dws = Regex::new(r"^destroyworkspace>>[0-9]+").unwrap();
    
    let mut ws_states = get_ws_states();
    println!("{:?}", ws_states);

    loop {
        let mut buf = vec![0; 1024];
        let _ = socket.read(&mut buf);
        
        /*
        events are comes in multiple lines
        with various pair so we need to split it
        by \n
        */
        let raw_events = String::from_utf8_lossy(&buf)
            .replace("\0", "");

        let mut events: Vec<&str> = raw_events 
            .split("\n")
            .collect();

        // remove last ""
        events.pop();
        
        // println!("{:?}", events);
        
        for e in events {

            // workspace event
            if re_ws.is_match(e) {
                let p = ws_states.p_active_ws_id;
                
                let c: i32 = e
                    .to_string()
                    .replace("workspace>>", "")
                    .parse()
                    .expect("failed to parsing workspace>>");

                ws_states.data[p as usize] = State::Occupied;
                ws_states.data[c as usize] = State::Active;
                ws_states.p_active_ws_id = c;
            }

            // destroy workspace event
            if re_dws.is_match(e) {
                let index: i32 = e
                    .to_string()
                    .replace("destroyworkspace>>", "")
                    .parse()
                    .expect("failed to parsing destroyworkspace>>");
            
                ws_states.data[index as usize] = State::Inactive;
            }
        }
        
        println!("{:?}", ws_states.data);
    }
}

fn main() {
    hypr_ws_daemon();
}

