use std::env;
use std::thread;
use std::process::Command;
use std::path::Path;
use std::io::Read;
use std::os::unix::net::UnixStream;

use regex::Regex;

use serde::{
    Serialize,
    Deserialize
};

use async_channel;
use async_channel::Sender;

use futures::executor::block_on;

use gtk::glib::MainContext;

use gtk;
use gtk::prelude::*;

use gtk::{
    Orientation,
    Align
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

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Active,
    Inactive,
    Occupied,
    Dead
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Ws {
    name: String,
    clients: Vec<String>,
    state: State
}

#[derive(Debug, Clone)]
pub struct WsInfo {
    // names: Vec<String>,
    max_index: i32,
    p_active_ws_id: i32,

    data: Vec<Ws>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct C {
    // at: [i32; 2],
    // size: [i32; 2],
    ws_id: i32,
    class: String,
    title: String
}

#[derive(Debug, Clone)]
pub struct CInfo {
    data: Vec<C>
}

// hyprland has 10 workspaces by default
const MAX_INDEX: i32 = 10;
const NAMES: [&str; 10] = ["1", "2", "3", "4", "5",
                            "6", "7", "8", "9", "10"];

pub struct HyprServer {
    // max_index: i32,
    // names: Vec<String>
}

impl HyprServer {
    #[allow(dead_code)]
    fn get_ws_info() -> WsInfo {
        /*
        spawn hyprctl as a child process. and
        parse it into a struct called "WsInfo".
        */
        
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
    
        let elems: Vec<Ws> = vec![];
        
        let mut ws_info = WsInfo {
            max_index: MAX_INDEX,
            p_active_ws_id: activeworkspace.id,
            data: elems
        };
    
        // head one is a dead variable
        let ws = Ws {
            name: String::from("dummy"),
            clients: vec![], 
            state: State::Dead
        };
        ws_info.data.push(ws);
        
        for i in 1..=ws_info.max_index {
            let ws = Ws {
                name: NAMES[i as usize - 1].to_string(),
                clients: vec![], 
                state: State::Inactive
            };
            
            ws_info.data.push(ws);
        }
    
        for c in clients {
            let id = c.workspace.id;
            
            if id > 0 {
                ws_info.data[id as usize].clients.push(c.address.replace("0x", ""));
                ws_info.data[id as usize].state = State::Occupied;
            }
        }
    
        ws_info
            .data[activeworkspace.id as usize]
            .state = State::Active;    
     
        ws_info
    }

    #[allow(dead_code)]
    pub fn summon_ws_daemon(s: Sender<WsInfo>) {
        /* 
        connect to hyprland event socket and
        return a WsInfo asynchronously.
        */ 
        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("failed to get hyprland instance signature");
        
        let sock_file = "/tmp/hypr/".to_string()
            + &signature
            + "/.socket2.sock"; // socket2.sock is a hyprland event socket
    
        let sock = Path::new(&sock_file);
    
        let mut socket = match UnixStream::connect(sock) {
            Ok(sock) => sock,
            Err(e) => {
                println!("couldn't connect: {e:?}");
                return
            }
        };
   
        let re_ws = Regex::new(r"^workspace>>[0-9]+").unwrap();
        let re_dws = Regex::new(r"^destroyworkspace>>[0-9]+").unwrap();
        
        let re_owin = Regex::new(r"^openwindow>>.+,[0-9]+,.+,.+$").unwrap();

        // the replacer to get a client's ws
        let re_owin_ws_head = Regex::new(r"^openwindow>>.{12},").unwrap();
        let re_owin_ws_tail = Regex::new(r",.+,.+$").unwrap();
        
        // the replacer to get a client's id
        let re_owin_id_head = Regex::new(r"^openwindow>>").unwrap();
        let re_owin_id_tail = Regex::new(r",[0-9]+,.+,.+$").unwrap();

        let re_cwin = Regex::new(r"^closewindow>>").unwrap();

        let mut ws_info = Self::get_ws_info();
    
        thread::spawn(move || {
            block_on(async move {
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
    
                    events.pop();
                     
                    for e in events {
                        // workspace event
                        if re_ws.is_match(e) {
                            let p = ws_info.p_active_ws_id;
                            
                            let c: i32 = e
                                .to_string()
                                .replace("workspace>>", "")
                                .parse()
                                .expect("failed to parsing workspace event");
                            
                            ws_info.data[c as usize].state = State::Active;
                            
                            if ws_info.data[p as usize].clients.len() != 0 {
                                ws_info.data[p as usize].state = State::Occupied;
                            } else {
                                ws_info.data[p as usize].state = State::Inactive;
                            }
                            
                            ws_info.p_active_ws_id = c;
                        }
    
                        // destroyworkspace event
                        else if re_dws.is_match(e) {
                            let index: i32 = e
                                .to_string()
                                .replace("destroyworkspace>>", "")
                                .parse()
                                .expect("failed to parsing destroyworkspace event");
                        
                            ws_info.data[index as usize].state = State::Inactive;
                        }

                        // openwindow event
                        else if re_owin.is_match(e) {
                            let c_ws_dirty = re_owin_ws_head.replace(e, "").to_string();
                            let c_ws = re_owin_ws_tail.replace(&c_ws_dirty, "");
 
                            let c_id_dirty = re_owin_id_head.replace(e, "").to_string();
                            let c_id = re_owin_id_tail.replace(&c_id_dirty, "");
                            
                            ws_info.data[c_ws.parse::<i32>().unwrap() as usize].clients.push(c_id.to_string());
                        }

                        // closewindow event
                        else if re_cwin.is_match(e) {
                            let c_id = re_cwin.replace(&e, "").to_string();
                        
                            for i in 0..ws_info.max_index {
                                ws_info.data[i as usize].clients.retain(|id| id != &c_id);
                            }
                        }
                    }
                    
                    let w = ws_info.clone();
                    let _ = s.send(w).await;
                }
            });
        });
    }

    #[allow(dead_code)]
    pub fn get_ws_widget() -> gtk::Box {
        let ws_widget = gtk::Box::builder()
            .name("hypr_ws_widget")
            .orientation(Orientation::Horizontal)
            .homogeneous(false)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();
         
        let (s, r) = async_channel::unbounded::<WsInfo>();

        thread::spawn(move || {
            block_on(async move {
                Self::summon_ws_daemon(s);
            }); 
        });

        let ws_info = Self::get_ws_info();
        
        // init
        for ws in ws_info.data {
            let b = gtk::Box::builder()
                .orientation(Orientation::Horizontal)
                .homogeneous(true)
                .build();

            let l = gtk::Label::new(Some(&ws.name)); 
            b.add(&l);
            
            match ws.state {
                State::Active => {
                    b.set_widget_name("ws_active");
                    ws_widget.add(&b); 
                },
                State::Occupied => {
                    b.set_widget_name("ws_occupied");
                    ws_widget.add(&b); 
                },
                State::Inactive => {
                    b.set_widget_name("ws_inactive");
                    ws_widget.add(&b);
                },
                State::Dead => {
                    ()
                }
            }
        }
 
        let ws_widget_clone = ws_widget.clone();
       
        // callback
        MainContext::default().spawn_local(async move {
            while let Ok(ws_info) = r.recv().await {
                let mut i = 1;

                for wid in ws_widget_clone.children() {
                    match ws_info.data[i].state {
                        State::Active => {
                            wid.set_widget_name("ws_active"); 
                        },
                        State::Occupied => {
                            wid.set_widget_name("ws_occupied"); 
                        },
                        State::Inactive => {
                            wid.set_widget_name("ws_inactive");
                        },
                        State::Dead => {
                            ()
                        }
                    };

                    i += 1;
                }
            }
        });

        ws_widget
    }
    
    #[allow(dead_code)]
    fn get_c_info() -> CInfo {
        /*
        spawn hyprctl as a child process. and
        parse it into a struct called "CInfo".
        */
    
        let mut h = Command::new("hyprctl");    
        
        let get_clients = h
            .arg("-j")
            .arg("clients");
        
        let stdout = get_clients
            .output()
            .expect("failed to execute command")
            .stdout;
        
        let raw_json = String::from_utf8_lossy(&stdout);
    
        let clients: Vec<Client> = serde_json::from_str(&raw_json)
            .expect("failed to deserialize");
     
        let data: Vec<C> = Vec::new();

        let mut c_info = CInfo {
            data: data
        };
        
        for c in clients {
            if c.mapped {
                let data = C {
                    ws_id: c.workspace.id,
                    class: c.class,
                    title: c.title
                };

                c_info.data.push(data)
            }
        }

        c_info
    }
    
    #[allow(dead_code)]
    pub fn summon_c_daemon(s: Sender<CInfo>) {
        /* 
        connect to hyprland event socket and
        return a CInfo asynchronously.
        */ 
        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("failed to get hyprland instance signature");
        
        let sock_file = "/tmp/hypr/".to_string()
            + &signature
            + "/.socket2.sock"; // socket2.sock is a hyprland event socket
    
        let sock = Path::new(&sock_file);
    
        let mut socket = match UnixStream::connect(sock) {
            Ok(sock) => sock,
            Err(e) => {
                println!("couldn't connect: {e:?}");
                return
            }
        };
    
        let re_owin = Regex::new(r"^openwindow>>.+,[0-9]+,.+,.+$").unwrap();
        let re_cwin = Regex::new(r"^closewindow>>([0-9]|[a-z])+$").unwrap();
        
        let mut c_info = Self::get_c_info();
    
        thread::spawn(move || {
            block_on(async move {
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
    
                    events.pop();
                     
                    for e in events {
                        // openwindow event
                        if re_owin.is_match(e) {
                            c_info = Self::get_c_info();
                        }
                        
                        // closewindow event
                        if re_cwin.is_match(e) {
                            c_info = Self::get_c_info();
                        }
                    }
                    
                    let c = c_info.clone();
                    let _ = s.send(c).await;
                }
            });
        });
    }
    
    #[allow(dead_code)]
    pub fn get_c_widget() -> gtk::Box {
        let c_widget = gtk::Box::builder()
            .name("hypr_c_widget")
            .orientation(Orientation::Horizontal)
            .homogeneous(false)
            .halign(Align::Center)
            .build();
         
        let (s, r) = async_channel::unbounded::<CInfo>();

        thread::spawn(move || {
            block_on(async move {
                Self::summon_c_daemon(s);
            }); 
        });

        let c_info = Self::get_c_info();
        
        // init
        for c in c_info.data {
            let img_box = gtk::Image::from_icon_name(
                Some(&c.class),
                gtk::IconSize::Dnd
            );

            c_widget.add(&img_box);
        }
 
        let c_widget_clone = c_widget.clone();
        
        // callback
        MainContext::default().spawn_local(async move {
            while let Ok(c_info) = r.recv().await {
                for wid in c_widget_clone.children() {
                    c_widget_clone.remove(&wid);
                }
        
                for c in c_info.data {
                    let img_box = gtk::Image::from_icon_name(
                        Some(&c.class),
                        gtk::IconSize::Dnd
                    );

                    c_widget_clone.add(&img_box);
                }

                c_widget_clone.show_all();
            }
        });
         
        c_widget
    }

    #[allow(dead_code)]
    fn get_ac_name() -> String {
        let mut h = Command::new("hyprctl");    
        
        let get_active_client = h
            .arg("-j")
            .arg("activewindow");
        
        let stdout = get_active_client
            .output()
            .expect("failed to execute command")
            .stdout;
        
        let raw_json = String::from_utf8_lossy(&stdout);
    
        let active_client: Client = match serde_json::from_str(&raw_json) {
            Ok(client) => client,
            Err(_) => return "".to_string()
        };

        active_client.title
    }

    #[allow(dead_code)]
    pub fn summon_ac_daemon(s: Sender<String>) {
        /* 
        connect to hyprland event socket and
        return an active client name asynchronously.
        */ 
        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("failed to get hyprland instance signature");
        
        let sock_file = "/tmp/hypr/".to_string()
            + &signature
            + "/.socket2.sock"; // socket2.sock is a hyprland event socket
    
        let sock = Path::new(&sock_file);
    
        let mut socket = match UnixStream::connect(sock) {
            Ok(sock) => sock,
            Err(e) => {
                println!("couldn't connect: {e:?}");
                return
            }
        };
    
        let re_awin = Regex::new(r"^activewindow>>").unwrap();
         
        thread::spawn(move || {
            block_on(async move {
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
    
                    events.pop();
                     
                    for e in events {
                        // activewindow event
                        if re_awin.is_match(e) {
                            let ac_name = Self::get_ac_name();
                            let _ = s.send(ac_name).await;
                        }
                    }
                }
            });
        });
    }

    #[allow(dead_code)]
    pub fn get_ac_widget() -> gtk::Box {
        let ac_widget = gtk::Box::builder()
            .name("hypr_ac_widget")
            .orientation(Orientation::Horizontal)
            .homogeneous(true)
            .build();
         
        let (s, r) = async_channel::unbounded::<String>();

        thread::spawn(move || {
            block_on(async move {
                Self::summon_ac_daemon(s);
            }); 
        });
 
        let ac_label = gtk::Label::new(Some(""));
 
        ac_widget.add(&ac_label);
        
        // callback
        MainContext::default().spawn_local(async move {
            while let Ok(ac_name) = r.recv().await {
                ac_label.set_label(&ac_name);
            }
        });
         
        ac_widget
    }
}
