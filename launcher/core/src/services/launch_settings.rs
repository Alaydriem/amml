use crate::LauncherError;

// Reads the two text settings of an instance that LeviLauncher writes: launch arguments and environment variables.
pub struct LaunchSettings;

impl LaunchSettings {
    // Split on whitespace; double quotes group words. Text next to a quote joins it. An empty `""` is an empty argument.
    pub fn args(raw: &str) -> Result<Vec<String>, LauncherError> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_token = false;
        let mut quoted = false;
        for character in raw.chars() {
            match character {
                '"' => {
                    quoted = !quoted;
                    in_token = true;
                }
                space if space.is_whitespace() && !quoted => {
                    if in_token {
                        args.push(std::mem::take(&mut current));
                        in_token = false;
                    }
                }
                other => {
                    current.push(other);
                    in_token = true;
                }
            }
        }
        if quoted {
            return Err(LauncherError::BadLaunchSetting(format!("unclosed quote in: {raw}")));
        }
        if in_token {
            args.push(current);
        }
        Ok(args)
    }

    // One `KEY=VALUE` per line, split at the first `=`; blank lines are skipped.
    pub fn env(raw: &str) -> Result<Vec<(String, String)>, LauncherError> {
        let mut pairs = Vec::new();
        for line in raw.split('\n') {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() {
                continue;
            }
            match line.split_once('=') {
                Some((key, value)) if !key.trim().is_empty() => pairs.push((key.trim().to_string(), value.to_string())),
                _ => return Err(LauncherError::BadLaunchSetting(format!("not KEY=VALUE: {line}"))),
            }
        }
        Ok(pairs)
    }
}
