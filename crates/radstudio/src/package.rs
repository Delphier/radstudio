use regex::Regex;
use std::path::Path;

pub struct PackageInfo {
    pub description: String,
}

impl PackageInfo {
    pub fn parse(path: impl AsRef<Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        // {$DESCRIPTION 'ExpressCharts by Developer Express Inc.'}
        let re_description = Regex::new(r"{\$DESCRIPTION '(.*)'}")?;
        let mut description = String::new();

        for line in content.lines() {
            if let Some(caps) = re_description.captures(line) {
                description = caps[1].to_string();
            }
        }

        Ok(Self { description })
    }
}
