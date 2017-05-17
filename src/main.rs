pub mod release;
pub mod utils;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate hyper;
extern crate getopts;
extern crate git2;

use std::process;
use std::io::Read;
use std::path::PathBuf;
use utils::get_project_root;
use utils::read_file;
use hyper::Client;
use hyper::status::StatusCode;
use release::Release;
use release::ReleaseResponse;
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
    opts.optopt("n", "name", "Name of the release. Defaults to \
                              package name + version number as \
                              defined in Cargo.toml. Example: \
                              cargo-hublish v0.1.0",
                "NAME");
    opts.optopt("t", "tag", "Name of the git tag for the release, \
                             if not set defaults to version number as \
                             defined in Cargo.toml.",
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
    opts.optopt("u", "url", "URL for the github API request. \
                              cargo-hublish attempts to find this \
                              based on the origin url of the git repo. \
                              If you're using a different remote such \
                              as 'github' then use the --remote flag \
                              to set that name, otherwise set the \
                              full api url with this flag.",
                 "URL");
    opts.optopt("r", "remote", "Remote name to use when generating \
                                 API endpoint. Defaults to origin.",
                 "REMOTE");

    let m = match opts.parse(&args) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    if m.opt_present("help") {
        print!("{}", opts.usage("Usage: cargo hublish [options]"));
        process::exit(0)
    }

    let project_root = match get_project_root() {
        Some(r) => r,
        None => {
            println!("Command must be run inside a cargo project.");
            process::exit(1)
        }
    };

    let mut manifest = project_root.clone();
    manifest.push("Cargo.toml");
    let content = read_file(&manifest);
    let cfg: Cargo = toml::from_str(&content).unwrap();
    println!("cfg: {:?}", cfg);

    let rel_name = if m.opt_present("name") {
        m.opt_str("name")
            .expect("name requires an argument")
    } else {
        format!("{} v{}", cfg.package.name,
                cfg.package.version.clone())
    };

    let tag_name = if m.opt_present("tag") {
        m.opt_str("tag_name")
            .expect("tag_name requires an argument")
    } else {
        cfg.package.version
    };

    let target_commit = if m.opt_present("commit") {
        m.opt_str("commit")
            .expect("commit requires an argument")
    } else {
        "master".to_string()
    };

    let body = if m.opt_present("message") {
        m.opt_str("message").unwrap().to_string()
    } else if m.opt_present("file") {
        let p = PathBuf::from(m.opt_str("file").unwrap());
        read_file(&p)
    } else {
        "".to_string()
    };


    let rel = Release::new()
        .name(rel_name)
        .tag_name(tag_name)
        .body(body)
        .target_commitsh(target_commit)
        .prerelease(m.opt_present("prerelease"))
        .draft(m.opt_present("draft"));

    let json = serde_json::to_string(&rel).unwrap();
    println!("json: {}", json);

    let repo = match git2::Repository::open(project_root) {
        Ok(r) => r,
        Err(e) => {
            println!("Error opening repo: {}", e);
            process::exit(1)
        }
    };

    let remote_name = m.opt_str("remote").unwrap_or("origin".to_string());
    let origin = match repo.find_remote(&remote_name) {
        Ok(o) => o,
        Err(e) => {
            println!("Remote origin not set {}", e);
            process::exit(1)
        }
    };

    let url = if m.opt_present("url") {
        m.opt_str("url")
            .expect("url requires an argument")
    } else {
        let u = origin.url().expect("Unexpected error.");
        let mut spl: Vec<&str> = u.split('/').collect();
        let mut base = "https://api.github.com/".to_string();
        let repo_name = spl.pop().unwrap();
        let username = spl.pop().unwrap();
        let endpoint = format!("{}/{}/releases", username, repo_name);
        base.push_str(&endpoint);
        base
    };

    println!("url: {}", url);

    let client = Client::new();
    let mut res = client.post(&url)
        .body(&json)
        .send()
        .expect("Error calling github API.");

    let mut buf = String::new();
    res.read_to_string(&mut buf)
        .expect("Unable to read Github response.");

    if res.status >= StatusCode::MultipleChoices {
        println!("ERROR Status: {} Response: {}", res.status, buf);
        process::exit(1)
    }

    let rr: ReleaseResponse = serde_json::from_str(&buf).unwrap();
    println!("Successfully created release!
View it here: {}", rr.html_url);
}
