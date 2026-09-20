use std::path::Path;

#[derive(Debug, Default, PartialEq)]
pub struct PackageInfo {
    pub description: String,
}

impl PackageInfo {
    pub fn from_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(content.into())
    }
}

impl<T: AsRef<str>> From<T> for PackageInfo {
    fn from(s: T) -> Self {
        let mut result = Self::default();

        for line in s.as_ref().lines() {
            let line = line.trim();
            // {$DESCRIPTION 'XXX Components'}
            if let Some(s) = line
                .strip_circumfix("{$DESCRIPTION", "}")
                .and_then(|s| s.trim().strip_circumfix("'", "'"))
            {
                result.description = s.to_string();
            }
        }

        result
    }
}

#[test]
fn parse_package_info() {
    assert_eq!(
        PackageInfo::from(" {$DESCRIPTION   'XXX'   }    "),
        PackageInfo {
            description: "XXX".to_string()
        }
    );
}
