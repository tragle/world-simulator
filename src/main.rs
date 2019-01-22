extern crate actix_web;

use actix_web::{http, server, App, HttpRequest, HttpResponse};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, SeekFrom};
use std::sync::{mpsc, Arc, Mutex};
use std::{env, fs, thread};

struct AppState {
    channel: mpsc::Sender<usize>,
    word: Arc<Mutex<String>>,
}

fn index(req: &HttpRequest<AppState>) -> HttpResponse {
    let channel = &req.state().channel;
    channel.send(1).unwrap();
    let word = &req.state().word.lock().unwrap().clone();
    let style = "body { 
        font-family: 'Charter', Palatino, serif; 
        font-size: 72px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        margin: 0;
    }";
    let html = format!(
        "<html>
            <head>
                <title>world simulator</title>
                <style>{}</style>
            </head>
        <body>
            {}
        </body>",
        style, word
    );
    HttpResponse::Ok().body(html)
}

fn main() -> io::Result<()> {
    let mutex = Arc::new(Mutex::new("".to_owned()));
    let mutex_copy = mutex.clone();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let _r = rx.recv().unwrap();
        let mut contents = vec![];
        {
            let source = File::open("world.txt").expect("No world");
            let mut file_reader = BufReader::new(&source);
            let mut line_buf = String::new();
            let _ = file_reader.read_line(&mut line_buf);
            let len = &line_buf.len();

            if line_buf.is_empty() {
                fs::remove_file("world.txt").expect("Can't remove world");
                panic!();
            }

            *(mutex_copy.lock().unwrap()) = line_buf;

            file_reader
                .seek(SeekFrom::Start(*len as u64))
                .expect("Can't seek world");
            file_reader
                .read_to_end(&mut contents)
                .expect("Can't read world");
        }
        let mut destination = File::create("world.txt").expect("Can't create world");
        let _ = destination.write(&contents);
    });

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT required");

    server::new(move || {
        let sender = mpsc::Sender::clone(&tx);
        App::with_state(AppState {
            channel: sender,
            word: mutex.clone(),
        })
        .resource("/", |r| r.method(http::Method::GET).f(index))
    })
    .bind(("0.0.0.0", port))
    .expect("Could not bind to port")
    .run();

    Ok(())
}
