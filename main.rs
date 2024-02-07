use std::thread;
use std::time::Duration;
use std::process::{
    Command,
    Stdio
};

use std::io::Read;

use gio::prelude::*;
use gtk::prelude::*;

use glib::MainContext;
use glib::ControlFlow;

use glib::source::timeout_add_local;

use gtk::{
    Application,
    ApplicationWindow,
    Box,
    Orientation,
    Button,
    Label
};

use gtk_layer_shell::{
    Edge,
    Layer,
    LayerShell
};

use async_channel;
use futures::executor::block_on;

#[allow(dead_code)]
fn typeis<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}

fn listen_label(command: String) -> Label {
    let label = Label::new(None);
    let label_clone = label.clone();
    
    let cmd = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to execute command");

    let mut stdout = cmd.stdout.expect("failed to get stdout handle");
    let mut buf = vec![0; 128];

    // a channel to communicate between shell's stdout and gtk label
    let (tx, rx) = async_channel::unbounded();
    
    // listening stdout and send the result
    thread::spawn(move || {
        block_on(async move {
            loop {
                let _ = stdout.read(&mut buf);
                let text = String::from_utf8_lossy(&buf)
                    .replace("\0", "")
                    .replace("\n", "");
            
                let _ = tx.send(text.to_string()).await;
            }
        });
    });
     
    // resieve a sent result and set that to label
    MainContext::default().spawn_local(async move {
        while let Ok(text) = rx.recv().await {
            label_clone.set_text(&text);
        }
    });
 
    label
}

fn poll_label(command: String, timeout: u64) -> Label {
    let label = Label::new(None);
    let label_clone = label.clone();

    let callback = move || {
        let mut shell = Command::new("sh");
        let cmd = shell.arg("-c").arg(&command);
        let out = cmd.output().expect("failed to execute command");
        let res = String::from_utf8_lossy(&out.stdout).replace("\n", "");

        label_clone.set_text(&res);
        ControlFlow::Continue
    };

    let sec = Duration::new(timeout, 0);
    timeout_add_local(sec, callback);

    label
}

fn activate(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .build();

    win.init_layer_shell();
    win.set_layer(Layer::Overlay);
    win.auto_exclusive_zone_enable();

    let anchors = [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, true),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        win.set_anchor(anchor, state);
    }

    // layout
    let hbox = Box::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .build();

    let date = poll_label("date +'%H:%M %b %d'".to_string(), 1);
    let lsn = listen_label("./command_1".to_string());
    let bat = poll_label("acpi -b | cut -d ',' -f 2".to_string(), 1);

    hbox.add(&date);
    hbox.add(&lsn);
    hbox.add(&bat);

    win.add(&hbox);    
    
    win.show_all()
}

fn main() {
    let application = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}
