use std::env;
// use std::thread;
use std::io::Read;
use std::path::Path;

use std::os::unix::net::UnixStream;

fn main() {
    let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("failed to get hyprland instance signature");
    
    let sock_file = "/tmp/hypr/".to_string()
        + &signature
        + "/.socket2.sock";

    let s = Path::new(&sock_file);

    let mut socket = match UnixStream::connect(s) {
        Ok(sock) => sock,
        Err(e) => {
            println!("couldn't connect: {e:?}");
            return
        }
    };
    
    let mut buf = vec![0; 1024];
    
    loop {
        let _ = socket.read(&mut buf);
        println!("{}", &String::from_utf8_lossy(&buf));
    }
}
