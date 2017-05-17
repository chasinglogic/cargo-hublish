pub mod release;
pub mod utils;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate hyper;
extern crate getopts;

use std::process;
use std::path::PathBuf;
use utils::get_project_root;
use utils::read_file;
use hyper::Client;
use release::Release;
use getopts::Options;
use std::env;

#[derive(Deserialize, Debug)]
struct Cargo {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    authors: Vec<String>,
}

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    // opts.optflag("g", "git", "Create the appropriate tags using git")
    opts.optflag("h", "help", "Show this help message.");
    opts.reqopt("n", "name", "Name of the release.", "NAME");
    opts.optopt("t", "tag", "Name of the git tag for the release, \
                             if not set defaults to `NAME`.",
                "TAG_NAME");
    opts.optopt("c", "commit", "SHA of the commit the tag should \
                                point to, defaults to HEAD of master",
                "COMMIT");
    opts.optopt("f", "file", "A file which contains the markdown for \
                              the body (description) of the release",
                "FILE");
    opts.optopt("m", "message", "The body of the release (description)",
                "MESSAGE");
    opts.optflag("d", "draft", "Set whether this is a draft release \
                                defaults to false");
    opts.optflag("p", "prerelease", "Set whether this is a prerelease \
                                defaults to false");

    let matches = match opts.parse(&args) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    if matches.opt_present("help") {
        println!("Help!");
        process::exit(0)
    }

    let rel_name = matches.opt_str("name").unwrap();

    let tag_name = if matches.opt_present("tag") {
        matches.opt_str("tag_name")
            .expect("tag_name requires an argument")
    } else {
        rel_name.clone()
    };

    let target_commit = if matches.opt_present("commit") {
        matches.opt_str("commit")
            .expect("commit requires an argument")
    } else {
        "master".to_string()
    };

    let body = if matches.opt_present("message") {
        matches.opt_str("message").unwrap().to_string()
    } else if matches.opt_present("file") {
        let p = PathBuf::from(matches.opt_str("file").unwrap());
        read_file(&p)
    } else {
        "".to_string()
    };

    let project_root = get_project_root();
    if project_root.is_none() {
        println!("Command must be run inside a cargo project.");
        process::exit(1)
    }

    let mut manifest = project_root.unwrap();
    manifest.push("Cargo.toml");
    let content = read_file(&manifest);

    let cfg: Cargo = toml::from_str(&content).unwrap();
    println!("cfg: {:?}", cfg);

    let rel = Release::new()
        .name(rel_name)
        .tag_name(tag_name)
        .body(body)
        .target_commitsh(target_commit)
        .prerelease(matches.opt_present("prerelease"))
        .draft(matches.opt_present("draft"));
    let json = serde_json::to_string(&rel).unwrap();
    println!("json: {}", json)


    let client = Client::new();
    let res = client.post("https://api.github.com/chasinglogic/")
}
