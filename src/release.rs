use serde_json;

#[derive(Serialize, Debug, Default)]
pub struct Release {
    pub tag_name: String,
    pub target_commitsh: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
}

// Builder pattern
impl Release {
    pub fn new() -> Release {
        Release {
            body: "".to_string(),
            name: "".to_string(),
            tag_name: "".to_string(),
            target_commitsh: "master".to_string(),
            draft: false,
            prerelease: false,
        }
    }

    pub fn name(mut self, name: String) -> Release {
        self.name = name;
        self
    }

    pub fn body(mut self, body: String) -> Release {
        self.body = body;
        self
    }

    pub fn tag_name(mut self, tag_name: String) -> Release {
        self.tag_name = tag_name;
        self
    }

    pub fn target_commitsh(mut self, target_commitsh: String) -> Release {
        self.target_commitsh = target_commitsh;
        self
    }

    pub fn prerelease(mut self, prerelease: bool) -> Release {
        self.prerelease = prerelease;
        self
    }

    pub fn draft(mut self, draft: bool) -> Release {
        self.draft = draft;
        self
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}


#[derive(Deserialize, Debug)]
pub struct ReleaseResponse {
    pub url: String,
    pub html_url: String,
    pub upload_url: String,
}
