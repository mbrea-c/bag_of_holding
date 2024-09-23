use std::{io::Write, thread};

use bevy::prelude::*;
use bevy_mod_debugdump::{render_graph, render_graph_dot};
use tempfile::NamedTempFile;

pub fn show_render_graph(app: &App) {
    let dot = render_graph_dot(app, &render_graph::Settings::default());
    dot_to_pdf(dot);
}

pub fn dot_to_pdf(dot: String) {
    let mut dot_file = NamedTempFile::new().unwrap();
    dot_file.write_all(dot.as_bytes()).unwrap();
    let out = std::process::Command::new("dot")
        .arg("-Tpdf")
        .arg(dot_file.path())
        .output()
        .unwrap();
    let mut out_file = NamedTempFile::new().unwrap();
    out_file.write_all(&out.stdout).unwrap();
    thread::spawn(move || {
        std::process::Command::new("zathura")
            .arg(out_file.path())
            .output()
            .unwrap();
        println!("Dotfile {:?}", dot_file.path());
        println!("Outfile {:?}", out_file.path());
    });
}
