/* 
List of bugs that i need to fix uwu*

CPU usage increases significantly
when i passed an invalid command.

add a json configurations support

*/
mod services;
use services::hyprland::HyprServer;
use services::mpris::MprisServer;
use crate::services::widget_factory;

// use std::fs;

use gtk::Orientation::{
    Horizontal,
    Vertical
};

use gtk::prelude::*;
use gtk::glib;
use gtk::gdk;
use gdk::EventMask;

use gtk_layer_shell::{
    Edge,
    Layer,
    LayerShell
};

// use serde::{
//     Serialize,
//     Deserialize
// };

#[allow(dead_code)]
fn who_are_you<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}

fn right_bar(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::builder()
        .application(app)
        .build();
    
    win.init_layer_shell();
    win.set_layer(Layer::Background);
    // win.auto_exclusive_zone_enable();
    win.set_exclusive_zone(20);

    win.set_anchor(Edge::Top, true);
    win.set_anchor(Edge::Right, true);
    win.set_anchor(Edge::Bottom, true);
    win.set_anchor(Edge::Left, false);

    let right_bar = gtk::Box::builder()
        .orientation(Horizontal)
        .homogeneous(false)
        .width_request(40)
        .build();

    let main_box = gtk::Box::builder()
        .hexpand(true)
        .build();

    main_box.set_widget_name("main_box");
    
    let decors = gtk::Box::builder()
        .orientation(Vertical)
        .homogeneous(false)
        .build();
 
    decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::B,
        (245.0, 245.0, 255.0, 255.0)
    ));
    
    decors.add(&gtk::Box::builder()
        .orientation(Vertical)
        .vexpand(true)
        .build());
    
    decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::C,
        (245.0, 245.0, 255.0, 255.0)
    ));
     
    right_bar.add(&decors);
    right_bar.add(&main_box);

    win.add(&right_bar);
    win.show_all();
}

fn left_bar(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::builder()
        .application(app)
        .build();
    
    win.init_layer_shell();
    win.set_layer(Layer::Background);
    // win.auto_exclusive_zone_enable();
    win.set_exclusive_zone(20);

    win.set_anchor(Edge::Top, true);
    win.set_anchor(Edge::Right, false);
    win.set_anchor(Edge::Bottom, true);
    win.set_anchor(Edge::Left, true);

    let left_bar = gtk::Box::builder()
        .orientation(Horizontal)
        .homogeneous(false)
        .width_request(40)
        .build();

    let main_box = gtk::Box::builder()
        .hexpand(true)
        .build();
    
    main_box.set_widget_name("main_box");

    let decors = gtk::Box::builder()
        .orientation(Vertical)
        .homogeneous(false)
        .build();
 
    decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::A,
        (245.0, 245.0, 255.0, 255.0)
    ));
    
    decors.add(&gtk::Box::builder()
        .orientation(Vertical)
        .vexpand(true)
        .build());
    
    decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::D,
        (245.0, 245.0, 255.0, 255.0)
    ));

    left_bar.add(&main_box);
    left_bar.add(&decors);

    win.add(&left_bar);
    win.show_all();
}

fn bottom_bar(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::builder()
        .application(app)
        .build();
    
    win.init_layer_shell();
    win.set_layer(Layer::Background);
    win.auto_exclusive_zone_enable();

    win.set_anchor(Edge::Top, false);
    win.set_anchor(Edge::Right, true);
    win.set_anchor(Edge::Bottom, true);
    win.set_anchor(Edge::Left, true);

    let bottom_bar = gtk::EventBox::builder()
        .height_request(20)
        .build();

    bottom_bar.set_events(EventMask::BUTTON_PRESS_MASK);

    let revealer = gtk::Revealer::builder()
        .reveal_child(false)
        .transition_duration(300)
        .transition_type(gtk::RevealerTransitionType::SlideUp)
        .build();
    
    let clients_list = gtk::Box::builder()
        .height_request(64)
        .homogeneous(true)
        .build();
    
    clients_list.add(&HyprServer::get_c_widget());
    revealer.add(&clients_list);
    
    let revealer_clone = revealer.clone();
    
    bottom_bar.connect_closure(
        // "enter_notify_event",
        "button_press_event",
        false,
        glib::closure_local!(move |_: gtk::EventBox, _: gdk::Event| {
            
            if revealer_clone.is_child_revealed() {
                revealer_clone.set_reveal_child(false);
            } else {
                revealer_clone.set_reveal_child(true);
            }

            true 
        })
    );
    
    let decors = gtk::Box::builder()
        .orientation(Horizontal)
        .homogeneous(false)
        .build();

    let main_box = gtk::Box::builder()
        .orientation(Vertical)
        .homogeneous(false)
        .build();

    main_box.set_widget_name("main_box");
 
    decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::D,
        (0.0, 0.0, 0.0, 255.0)
    ));

    decors.add(&gtk::Box::builder().hexpand(true).build());
    
    decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::C,
        (0.0, 0.0, 0.0, 255.0)
    ));

    main_box.add(&revealer);
    main_box.add(&decors);

    bottom_bar.add(&main_box);

    win.add(&bottom_bar);
    win.show_all();
}

const TOP_BAR_HEIGHT: i32 = 74;

const LEFT_BOX_WIDTH: i32 = 600;
const RIGHT_BOX_WIDTH: i32 = 260;

fn top_bar(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::builder()
        .application(app)
        .build();

    win.init_layer_shell();
    win.set_layer(Layer::Background);
    win.auto_exclusive_zone_enable();

    win.set_anchor(Edge::Top, true);
    win.set_anchor(Edge::Right, true);
    win.set_anchor(Edge::Bottom, false);
    win.set_anchor(Edge::Left, true);

    let top_bar = gtk::Box::builder()
        .orientation(Horizontal)
        .homogeneous(false)
        .height_request(TOP_BAR_HEIGHT)
        .build();

    let h_box = gtk::Box::builder()
        .orientation(Horizontal)
        .homogeneous(false)
        .build();
    
    let l_box = gtk::Box::builder()
        .orientation(Horizontal)
        .width_request(LEFT_BOX_WIDTH)
        .homogeneous(false)
        .build();

    let r_box = gtk::Box::builder()
        .orientation(Horizontal)
        // .width_request(RIGHT_BOX_WIDTH)
        .homogeneous(false)
        .build();

    let c_box = gtk::Box::builder()
        .orientation(Vertical)
        .homogeneous(false)
        .hexpand(true)
        .build();
    
    h_box.set_widget_name("h_box"); 
    l_box.set_widget_name("l_box");
    r_box.set_widget_name("r_box");
    c_box.set_widget_name("c_box");

    top_bar.add(&widget_factory::cairo_katana_corner(
        20, TOP_BAR_HEIGHT,
        widget_factory::KatanaType::A,
        (245.0, 245.0, 255.0, 255.0),
        (0.0, 0.0, 0.0, 255.0)
    ));

    top_bar.add(&h_box);
     
    top_bar.add(&widget_factory::cairo_katana_corner(
        20, TOP_BAR_HEIGHT,
        widget_factory::KatanaType::B,
        (245.0, 245.0, 255.0, 255.0),
        (0.0, 0.0, 0.0, 255.0)
    ));
    
    h_box.add(&l_box);
    
    h_box.add(&widget_factory::cairo_katana_corner(
        20, TOP_BAR_HEIGHT,
        widget_factory::KatanaType::C,
        (245.0, 245.0, 255.0, 255.0),
        (0.0, 0.0, 0.0, 0.0)
    ));
    
    h_box.add(&c_box);
    
    h_box.add(&widget_factory::cairo_katana_corner(
        20, TOP_BAR_HEIGHT,
        widget_factory::KatanaType::D,
        (245.0, 245.0, 255.0, 255.0),
        (0.0, 0.0, 0.0, 0.0)
    ));
    
    h_box.add(&r_box);

    let main_box = gtk::Box::builder()
        .height_request(20)
        .build();
    
    main_box.set_widget_name("main_box");

    let c_box_decors = gtk::Box::builder()
        .build();
 
    c_box_decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::A,
        (245.0, 245.0, 255.0, 255.0)
    ));
    
    c_box_decors.add(&gtk::Box::builder()
        .orientation(Horizontal)
        .hexpand(true)
        .build());
    
    c_box_decors.add(&widget_factory::cairo_reverse_katana_corner(
        20,
        widget_factory::KatanaType::B,
        (245.0, 245.0, 255.0, 255.0)
    ));

    // modules
    let date = widget_factory::poll_label("date +'%H:%M %b %d'".to_string(), 30);
    let bat = widget_factory::poll_label("acpi -b | cut -d ',' -f 2".to_string(), 120);
 
    l_box.add(&widget_factory::pfp_box(
        "/home/tailr/Pictures/artworks/sroll.jpg",
        40, 40
    ));
    l_box.add(&MprisServer::get_custom_widget()); 
    // l_box.add(&HyprServer::get_ws_widget());
    // l_box.add(&HyprServer::get_ac_widget());

    r_box.add(&date);
    r_box.add(&bat);

    c_box.add(&main_box);
    c_box.add(&c_box_decors);
    
    win.add(&top_bar);
    win.show_all();
}

fn activate(app: &gtk::Application) {
    top_bar(app); 
    bottom_bar(app);
    left_bar(app);
    right_bar(app);
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    let style = include_bytes!("style.css");

    provider.load_from_data(style)
        .expect("failed to load css");

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("failed to init provider"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION 
    );
}

fn main() {
    // let conf_file = fs::read_to_string("./config.json")
    //     .expect("failed to read \"config.json\"");

    // let conf_json: serde_json::Value
    //     = serde_json::from_str(&conf_file).unwrap();
 
    let application = gtk::Application::builder()
        .application_id("org.TopBar")
        .build();
     
    application.connect_activate(|app| {
        load_css();
        activate(app);
    });
     
    application.run();
}
