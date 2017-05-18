pub mod release;
pub mod utils;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate hyper;
extern crate hyper_native_tls;
extern crate getopts;
extern crate git2;
extern crate ioutil;
extern crate base64;

use std::process;
use base64::encode;
use std::io::Read;
use std::path::PathBuf;
use utils::get_project_root;
use ioutil::read_file;
use ioutil::prompt;
use hyper::Client;
use hyper::client;
use hyper::status::StatusCode;
use hyper::header;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
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
    opts.optopt("", "url", "URL for the github API request. \
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
    opts.optopt("u", "username", "Your github username. If not \
                                  provided you will be prompted.",
                 "USERNAME");
    opts.optopt("p", "password", "Your github password. If not \
                                  provided you will be prompted.",
                 "PASSWORD");

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
    let content = read_file(&manifest).unwrap();
    let cfg: Cargo = toml::from_str(&content).unwrap();

    let json = Release::new()
        .name(release_name(&m, &cfg))
        .tag_name(tag_name(&m, &cfg))
        .body(body(&m))
        .target_commitsh(target_commit(&m))
        .prerelease(m.opt_present("prerelease"))
        .draft(m.opt_present("draft"))
        .to_json()
        .unwrap();

    let repo = match git2::Repository::open(project_root) {
        Ok(r) => r,
        Err(e) => {
            println!("Error opening repo: {}", e);
            process::exit(1)
        }
    };

    let remote_name = m.opt_str("remote")
        .unwrap_or_else(|| "origin".to_string());

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

    println!("Release {} \
              \nAbout to Create at: {}", json, url);

    let ans = prompt("Is this correct? Y/n: ")
        .expect("Error reading from stdin.");
    if ans.to_lowercase().starts_with('n') {
        println!("Aborting release...");
        process::exit(0)
    }

    let unpw = get_username_password(&m);
    let auth = header::Authorization(format!("Basic {}", unpw));

    let mut headers = header::Headers::new();
    headers.set(auth);
    headers.set(header::UserAgent("cargo-hublish".to_string()));

    // Set up SSL support for hyper.
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);

    // Hyper hates me so this was way harder than it should have been.
    let proxy_url;
    let ssl2;
    let client = if let Ok(proxy) = env::var("HTTP_PROXY") {
        // Easier than parsing myself
        proxy_url = hyper::Url::parse(&proxy).unwrap();

        // The proxy client needs it's own ssl client. Since
        // NativeTlsClient doesn't support clone we have to make a new
        // one.
        ssl2 = NativeTlsClient::new().unwrap();

        // Build our proxy config.
        let pc = client::ProxyConfig::new(
            proxy_url.scheme().clone(),
            // This has to be valid for static lifteime, because
            // reasons.
            proxy_url.host_str()
                .unwrap()
                .to_string()
                .clone(),
            proxy_url.port().unwrap_or(80),
            connector,
            ssl2
        );

        Client::with_proxy_config(pc)
    } else {
        Client::with_connector(connector)
    };

    println!("Sending request...");
    let mut res = client.post(&url)
        .headers(headers)
        .body(&json)
        .send()
        .expect("Error calling github API.");

    println!("Reading response...");
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

fn release_name(m: &getopts::Matches, cfg: &Cargo) -> String {
    if m.opt_present("name") {
        return m.opt_str("name")
            .expect("name requires an argument")
    }

    format!("{} v{}", cfg.package.name,
            cfg.package.version.clone())
}

fn tag_name(m: &getopts::Matches, cfg: &Cargo) -> String {
    if m.opt_present("tag") {
        return m.opt_str("tag_name")
            .expect("tag_name requires an argument")
    }

    cfg.package.version.clone()
}


fn target_commit(m: &getopts::Matches) -> String {
    if m.opt_present("commit") {
        return m.opt_str("commit").unwrap()
    }

    "master".to_string()
}


fn body(m: &getopts::Matches) -> String {
    if m.opt_present("message") {
        m.opt_str("message").unwrap().to_string()
    } else if m.opt_present("file") {
        let p = PathBuf::from(m.opt_str("file").unwrap());
        read_file(&p).unwrap()
    } else {
        "".to_string()
    }
}


fn get_username_password(m: &getopts::Matches) -> String {
    let username = if m.opt_present("username") {
        m.opt_str("username")
            .expect("username requires an argument")
    } else {
        prompt("Github Username: ").unwrap()
    };

    let password = if m.opt_present("password") {
        m.opt_str("password")
            .expect("password requires an argument")
    } else {
        prompt("Github Password: ").unwrap()
    };

    let com = format!("{}:{}", username, password);
    encode(&com)
}
