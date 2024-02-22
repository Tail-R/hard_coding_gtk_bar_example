use crate::widget_factory;

use regex::Regex;

use gtk;
use gtk::glib;
use gtk::prelude::*;

use gtk::RevealerTransitionType as AnimType;

// listen commands
const CMD_METADATA: &str = "playerctl --follow metadata";
const CMD_STATUS: &str = "playerctl --follow status";
const CMD_TITLE: &str = "playerctl --follow metadata --format '{{title}}'";
// const CMD_ARTIST: &str = "playerctl --follow metadata --format '{{artist}}'";

// one shot
const CMD_PLAYER: &str = "playerctl -l";
const CMD_TOGGLE: &str = "playerctl play-pause &>/dev/null";

pub struct MprisServer {
    // icon_play: String,
    // icon_pause: String
}

impl MprisServer {
    pub fn get_toggle_widget() -> gtk::Box {
        let toggle_widget = gtk::Box::builder()
            .name("mpris_toggle_widget")
            .homogeneous(true) 
            .build();

        let button = gtk::Button::new();
        button.set_label("󰐊");
        toggle_widget.add(&button);

        button.connect_clicked(|_: &gtk::Button| {
            widget_factory::exec_once(CMD_TOGGLE.to_string());
        });

        let r = widget_factory::listen_var(CMD_STATUS.to_string());

        let button_clone = button.clone();

        // callback
        glib::MainContext::default().spawn_local(async move {
            while let Ok(text) = r.recv().await {
                if text == "Playing".to_string() {
                    button_clone.set_label("󰏤"); 
                } else {
                    button_clone.set_label("󰐊");
                }
            }
        });

        toggle_widget
    }

    pub fn get_player_widget() -> gtk::Box {
        let player_widget = gtk::Box::builder()
            .name("mpris_player_widget")
            .width_request(100)
            .homogeneous(true) 
            .build();

        let label = gtk::Label::builder()
            .label("no media")
            .build();
        
        player_widget.add(&label);
        
        let r = widget_factory::listen_var(CMD_METADATA.to_string());

        let re_noise = Regex::new(r"\.instance_.*").unwrap(); 
 
        // callback
        glib::MainContext::default().spawn_local(async move {
            while let Ok(_) = r.recv().await {
                let raw_player_name = widget_factory::exec_once(CMD_PLAYER.to_string());
               
                if raw_player_name != "" {  
                    let player_name = re_noise.replace(&raw_player_name, "").to_string();
                    label.set_text(&player_name);
                } else {
                    label.set_text("no media");
                }
            }
        });

        player_widget
    }

    pub fn get_title_widget() -> gtk::Revealer {
        let title_widget = gtk::Revealer::builder()
            .name("mpris_title_widget")
            .reveal_child(false)
            .transition_duration(300)
            .transition_type(AnimType::SlideLeft)
            .build();

        let label = gtk::Label::new(Some(""));
        
        title_widget.add(&label);

        let r = widget_factory::listen_var(CMD_TITLE.to_string());

        let title_widget_clone = title_widget.clone();
        let label_clone = label.clone();

        // callback
        glib::MainContext::default().spawn_local(async move {
            while let Ok(text) = r.recv().await {
                if text != "" {

                    // is it stupid ?
                    #[allow(unused_assignments)]
                    let mut sized_text = "";

                    if text.len() > 30 {
                        sized_text = &text.get(0..29).unwrap_or("failed");
                    } else {
                        sized_text = &text;
                    }

                    label_clone.set_text(&sized_text);
                    title_widget_clone.set_reveal_child(true);
                } else {
                    title_widget_clone.set_reveal_child(false);
                }
            }
        });

        title_widget
    }

    #[allow(dead_code)]
    pub fn get_custom_widget() -> gtk::Box {
        let custom_widget = gtk::Box::builder()
            .name("mpris_custom_widget")
            .homogeneous(false)
            .valign(gtk::Align::Center)
            .spacing(10)
            .orientation(gtk::Orientation::Horizontal)
            .build();

        custom_widget.add(&Self::get_toggle_widget());
        custom_widget.add(&Self::get_title_widget());
        custom_widget.add(&Self::get_player_widget());
    
        custom_widget    
    }
}
