use std::thread;
use std::time::Duration;
use std::process::{
    Command,
    Stdio
};

use std::io::Read;

use gtk;
use gtk::glib;
use gtk::gdk_pixbuf;
use gtk::prelude::*;

use async_channel;
use async_channel::Receiver;

use futures::executor::block_on;

pub fn exec_once(command: String) -> String {
    let mut shell = Command::new("sh");
    let cmd = shell.arg("-c").arg(&command);
    let out = match cmd.output() {
        Ok(output) => output,
        Err(error) => {
            panic!("failed to run command: {:?}", error)
        }
    };
    
    let text = String::from_utf8_lossy(&out.stdout).replace("\n", "");

    text
}

#[allow(dead_code)]
pub fn poll_label(
    command: String,
    timeout: u64
    ) -> gtk::Label {
    
    let label = gtk::Label::new(Some(""));
    let label_clone = label.clone();

    let callback = move || {
        let text = exec_once(command.clone());
        label_clone.set_text(&text);
        
        glib::ControlFlow::Continue 
    };

    callback(); // init

    let sec = Duration::new(timeout, 0);
    glib::timeout_add_local(sec, callback);

    label
}

#[allow(dead_code)]
pub fn listen_label(
    command: String
    ) -> gtk::Label {

    let label = gtk::Label::new(Some(""));

    let process = match Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn() {

        Ok(child) => child,
        Err(error) => {
            panic!("failed to run command: {:?}", error)
        }
    };

    let mut stdout = match process.stdout {
        Some(c_stdout) => c_stdout,
        None => {
            panic!("failed to get a handle for the child's stdout")
        }
    };

    let (s, r) = async_channel::unbounded();

    thread::spawn(move || {
        block_on(async move {
            loop {
                let mut buf = vec![0; 128];
                let _ = stdout.read(&mut buf);
                let text = String::from_utf8_lossy(&buf)
                    .replace("\0", "")
                    .replace("\n", "");

                let _ = s.send(text.to_string()).await;
            }
        });
    });

    let label_clone = label.clone();

    glib::MainContext::default().spawn_local(async move {
        while let Ok(text) = r.recv().await {
            label_clone.set_text(&text);
        }
    });

    label
}

#[allow(dead_code)]
pub fn listen_var(
    command: String
    ) -> Receiver<String> {

    let process = match Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn() {

        Ok(child) => child,
        Err(error) => {
            panic!("failed to run command: {:?}", error)
        }
    };

    let mut stdout = match process.stdout {
        Some(c_stdout) => c_stdout,
        None => {
            panic!("failed to get a handle for the child's stdout")
        }
    };

    let (sender, receiver) = async_channel::unbounded();

    thread::spawn(move || {
        block_on(async move {
            loop {
                let mut buf = vec![0; 128];
                let _ = stdout.read(&mut buf);
                let text = String::from_utf8_lossy(&buf)
                    .replace("\0", "")
                    .replace("\n", "");

                let _ = sender.send(text.to_string()).await;
            }
        });
    });

    receiver
}

#[allow(dead_code)]
pub fn pfp_box(path: &str, w: i32, h: i32) -> gtk::Image {
    let pfp_pb = gdk_pixbuf::Pixbuf::from_file(
        path
    ).expect("image file does not found");
    
    let pfp_box = match pfp_pb.scale_simple(
        w, h, gdk_pixbuf::InterpType::Bilinear
    ) {
   
        Some(pb) => gtk::Image::builder()
            .pixbuf(&pb)
            .build(),

        None => gtk::Image::new()
    };

    pfp_box
}

#[allow(dead_code)]
pub enum KatanaType {
    A, B,
    D, C
}

#[allow(dead_code)]
pub fn cairo_reverse_katana_corner(
    w: i32,
    shape: KatanaType,
    fg: (f64, f64, f64, f64)

) -> gtk::DrawingArea {  
    let (fg_r, fg_g, fg_b, fg_a) = fg;

    let cairo_box = gtk::DrawingArea::builder()
        .width_request(w)
        .height_request(w)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    
    cairo_box.connect_draw(move |_, cr| {
        let two_pi = 6.28;
        
        cr.set_source_rgba(fg_r / 255.0, fg_g / 255.0, fg_b / 255.0, fg_a / 255.0);
       
        match shape {
            KatanaType::A => {
                cr.arc(w.into(), w.into(), w.into(),
                    0.0, two_pi);
                
                cr.move_to(w.into(), 0.0);
                cr.line_to(0.0, 0.0); 
                cr.line_to(0.0, w.into()); 
                cr.line_to(w.into(), w.into()); 

                cr.clip();
                
                cr.rectangle(0.0, 0.0,
                    (w * 2).into(),
                    (w * 2).into());
            },
            KatanaType::B => {
                cr.arc(0.0, w.into(), w.into(),
                    0.0, two_pi);
                
                cr.move_to(w.into(), 0.0);
                cr.line_to(0.0, 0.0); 
                cr.line_to(0.0, w.into()); 
                cr.line_to(w.into(), w.into()); 

                cr.clip();
                
                cr.rectangle(0.0, 0.0,
                    (w * 2).into(),
                    (w * 2).into());
            
            },
            KatanaType::C => {
                cr.arc(0.0, 0.0, w.into(),
                    0.0, two_pi);
                
                cr.move_to(w.into(), 0.0);
                cr.line_to(0.0, 0.0); 
                cr.line_to(0.0, w.into()); 
                cr.line_to(w.into(), w.into()); 

                cr.clip();
                
                cr.rectangle(0.0, 0.0,
                    (w * 2).into(),
                    (w * 2).into());
            
            },
            KatanaType::D => {
                cr.arc(w.into(), 0.0, w.into(),
                    0.0, two_pi);
                
                cr.move_to(w.into(), 0.0);
                cr.line_to(0.0, 0.0); 
                cr.line_to(0.0, w.into()); 
                cr.line_to(w.into(), w.into()); 

                cr.clip();
                
                cr.rectangle(0.0, 0.0,
                    (w * 2).into(),
                    (w * 2).into());
            
            }
        };
        
        cr.fill().expect("invalid cairo surface state");

        gtk::glib::Propagation::Proceed    
    });

    cairo_box
}

#[allow(dead_code)]
pub fn cairo_katana_corner(
    w: i32,
    h: i32,
    shape: KatanaType,
    fg: (f64, f64, f64, f64),
    bg: (f64, f64, f64, f64)

) -> gtk::DrawingArea {
    if w > h {
        panic!("invalid parameters");
    }

    let (fg_r, fg_g, fg_b, fg_a) = fg;
    let (bg_r, bg_g, bg_b, bg_a) = bg;

    let cairo_box = gtk::DrawingArea::builder()
        .width_request(w)
        .build();
    
    cairo_box.connect_draw(move |_, cr| {
        let two_pi = 6.28;
        
        cr.set_source_rgba(bg_r / 255.0, bg_g / 255.0, bg_b / 255.0, bg_a / 255.0);
        let _ = cr.paint();

        cr.set_source_rgba(fg_r / 255.0, fg_g / 255.0, fg_b / 255.0, fg_a / 255.0);

        match shape {
            KatanaType::A => {
                cr.arc(w.into(), w.into(), w.into(), 0.0, two_pi);
                cr.rectangle(0.0, w.into(), w.into(), (h - w).into());
            },
            KatanaType::B => {
                cr.arc(0.0, w.into(), w.into(), 0.0, two_pi);
                cr.rectangle(0.0, w.into(), w.into(), (h - w).into());
            },
            KatanaType::C => {
                cr.arc(0.0, (h - w).into(), w.into(), 0.0, two_pi);
                cr.rectangle(0.0, 0.0, w.into(), (h - w).into());
            },
            KatanaType::D => {
                cr.arc(w.into(), (h - w).into(), w.into(), 0.0, two_pi);
                cr.rectangle(0.0, 0.0, w.into(), (h - w).into());
            }
        };

        cr.fill().expect("invalid cairo surface state");

        gtk::glib::Propagation::Proceed    
    });

    cairo_box
}
