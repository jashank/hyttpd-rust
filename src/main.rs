// Copyright 2015  Jashank Jeremy.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(ip_addr)]
#![feature(path_ext)]

extern crate hyper;
extern crate chrono;

use std::fs::{self, PathExt};
use std::io::{self, BufRead, Write};
use std::path;
use hyper::{header, uri, status};
use hyper::server::{Server, Request, Response};
use hyper::net::HttpListener;

const HOST: &'static str = "0.0.0.0";
const PORT: usize = 8000usize;
const SERVER_VERSION: &'static str =
    concat!("hyttpd/", env!("CARGO_PKG_VERSION"));

const ERR_BAD_REQUEST: &'static [u8] =
    b"<html><body><h1>400 Bad Request</h1></body></html>";
const ERR_NOT_FOUND: &'static [u8] =
    b"<html><body><h1>404 Not Found</h1></body></html>";
const ERR_INTERNAL_SERVER_ERROR: &'static [u8] =
    b"<html><body><h1>500 Internal Server Error</h1></body></html>";

fn log_request(req: &Request) -> () {
    let dt = chrono::Local::now();
    println!("{} [{}] {}:{} {:?}",
             req.remote_addr.ip(), dt.format("%Y-%m-%d %H:%M:%S"),
             req.version, req.method, req.uri);
}

fn request_path(req: &Request) -> Option<String> {
    match req.uri {
        uri::RequestUri::AbsolutePath(ref path) => {
            let mut path = path.clone();

            path.remove(0);
            Some(path)
        },

        uri::RequestUri::AbsoluteUri(ref uri) => {
            let mut path = uri.clone().serialize_path().unwrap();

            path.remove(0);
            Some(path)
        },

        _ => {
            return None;
        }
    }
}

fn request_dirindex(file_path_str: &String) -> (path::PathBuf, bool, bool) {
    let mut file_path = path::PathBuf::from(file_path_str);

    if file_path.is_absolute() {
        panic!("file_path '{}' is not absolute");
    }

    if file_path.file_name().is_none() {
        file_path.set_file_name("index.html");
    }

    let (exists, is_dir): (bool, bool) = {
        let path = file_path.as_path();
        (path.exists(), path.is_dir())
    };

    (file_path, exists, is_dir)
}

fn render_directory(res: Response, dirname: String) -> () {
    let path_str = format!("./{}", dirname);
    let path = path::Path::new(&path_str);

    let mut res = res.start().unwrap();
    res.write(format!("<html><head><title>Index of {}</title></head>",
                      dirname).as_bytes()).unwrap();
    res.write(format!("<body><h1>Index of {}</h1><ul>\n",
                      dirname).as_bytes()).unwrap();

    let dir = match fs::read_dir(path) {
        Ok(v)  => v,
        Err(e) => panic!("readdir {}: {}", dirname, e)
    };
    for entry in dir {
        let entry = entry.unwrap();
        let dirtag = if entry.path().is_dir() { "/" } else { "" };
        res.write(format!("<li><a href=\"{0}{1}\">{0}</a></li>\n",
                          entry.file_name().to_str().unwrap(),
                          dirtag)
                  .as_bytes()).unwrap();
    }

    res.write(b"</ul></body></html>").unwrap();
    res.end().unwrap();
}

fn request_handler(req: Request, mut res: Response) -> () {
    // Set server version in header.
    log_request(&req);
    res.headers_mut().set(header::Server(SERVER_VERSION.to_string()));

    let file_path_str: String = match request_path(&req) {
        Some(v) => v,
        None => {
            *(res.status_mut()) = status::StatusCode::BadRequest;
            println!("invalid URI");

            res.send(ERR_BAD_REQUEST).unwrap();
            return;
        }
    };
    let (file_path, exists, is_dir) = request_dirindex(&file_path_str);

    //# println!("{} {} {}", file_path_str, exists, is_dir);

    if is_dir || file_path_str == "" {
        render_directory(res, file_path_str);
        return ();
    }

    if !exists && !is_dir {
        *(res.status_mut()) = status::StatusCode::NotFound;
        println!("not found");

        res.send(ERR_NOT_FOUND).unwrap();
        return;
    }

    let f: fs::File = match fs::File::open(file_path.as_path()) {
        Ok(v)  => v,
        Err(e) => {
            *(res.status_mut()) = status::StatusCode::InternalServerError;
            println!("can't open {:?}: {}", file_path, e);

            res.send(ERR_INTERNAL_SERVER_ERROR).unwrap();
            return;
        }
    };

    {
        let mut res = res.start().unwrap();
        let mut buf = io::BufReader::new(f);
        loop {
            let consumed = match buf.fill_buf() {
                Ok(bytes) => { res.write(bytes).unwrap(); bytes.len() },
                Err(e) => panic!("i/o error in write loop: {}", e),
            };
            buf.consume(consumed);
            if consumed == 0 { break; }
        };
        res.end().unwrap();
    };
}

fn main() {
    println!("{} starting; listening on {}:{}", SERVER_VERSION, HOST, PORT);

    let svr: Server<HttpListener> = match Server::http((HOST, PORT as u16)) {
        Ok(v)  => v,
        Err(e) => panic!("couldn't listen: {}", e)
    };

    svr.handle(request_handler).unwrap();
}
