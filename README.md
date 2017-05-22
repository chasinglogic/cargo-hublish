# cargo-hublish
[![Apache 2.0 License](https://img.shields.io/badge/license-Apache%202.0-ff69b4.svg)](https://github.com/chasinglogic/cargo-hublish/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/cargo-hublish.svg)](https://crates.io/crates/cargo-hublish)
[![Crates.io](https://img.shields.io/crates/d/cargo-hublish.svg)](https://crates.io/crates/cargo-hublish)

Automatically publish your Rust projects to Github releases.

## Installation

You can install cargo-hublish using cargo itself with the following one-liner

```
cargo install cargo-hublish
```

## Why

I like to write tools, specifically CLI tools. Publishing those tools
to Github is a great way to distribute the project. However creating
Github releases is fairly tedious so I decided to automate it using
metadata from the Cargo.toml

## Contributing

I'm always happy to accept pull requests for any features you would
like to see added, a few I personally would like to see added are:

- Automate cross compiling the project and put the resulting builds in
  the release using the upload_url
- Integration tests (not sure what the best way to do this is
  personally given it's so dependent on the Github API. If you have
  ideas please send them my way!)
-

If you're not sure if your feature is a good fit or not, just submit a
Github issue asking for comments before you start working on it!

As always follow
the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html),
not only is it the nice thing to do it's one of the reasons I
personally get so excited about Rust.

If something you add isn't covered by an existing integration test,
please please please write one for your thing.

Please submit all pull requests to the develop branch.

## Building from source

Just like any Rust project you can simply build with ```cargo build```
however I've included a make file which adds some niceties for
performing certain commands. Specifically when running tests you
should always use ```make test``` since it will clean up and
regenerate the cargo project used for integration testing.

## Usage

```
Publish Rust projects to Github Releases

Usage: cargo hublish [options]

Options:
	-h, --help          Show this help message.
	-n, --name NAME     Name of the release. Defaults to package name +
						version number as defined in Cargo.toml. Example:
						cargo-hublish v0.1.0
	-t, --tag TAG_NAME  Name of the git tag for the release, if not set
						defaults to version number as defined in Cargo.toml.
	-c, --commit COMMIT SHA of the commit the tag should point to, defaults to
						HEAD of master
	-f, --file FILE     A file which contains the markdown for the body
						(description) of the release
	-m, --message MESSAGE
						The body of the release (description)
	-d, --draft         Set whether this is a draft release defaults to false
	-p, --prerelease    Set whether this is a prerelease defaults to false
		--url URL       URL for the github API request. cargo-hublish attempts
						to find this based on the origin url of the git repo.
						If you're using a different remote such as 'github'
						then use the --remote flag to set that name, otherwise
						set the full api url with this flag.
	-r, --remote REMOTE Remote name to use when generating API endpoint.
						Defaults to origin.
	-u, --username USERNAME
						Your github username. If not provided you will be
						prompted.
	-p, --password PASSWORD
						Your github password. If not provided you will be
						prompted.

```

You can run `cargo hublish` without any flags and it will generate a
Github release with reasonable defaults. It will always prompt you
with the generated release so you can make sure it all looks good
before sending off to Github. If you have not set the environment
variable `$GITHUB_API_TOKEN` and have not passed in the `--username`
and `--password` flags you will be prompted to login before the
request is sent off. I highly recommend generating an API token and
setting the environment variable `GITHUB_API_TOKEN` to it. When this
is done `cargo hublish` will automatically read that and authenticate
the request using it. This prevents you having to store your plain
text password and/or logging in every time.
