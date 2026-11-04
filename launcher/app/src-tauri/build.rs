fn main() {
    // the date the build ran, as UTC YYYY-MM-DD, for the About page
    println!("cargo:rustc-env=AMML_LAUNCHER_BUILT_ON={}", BuildDate::today());
    // `cargo check` does not run the frontend build, and the Tauri context needs the folder to exist
    let _ = std::fs::create_dir_all("../build");
    println!("cargo:rerun-if-changed=build.rs");
    tauri_build::build();
}

// The calendar date of today in UTC, worked out from the system clock.
struct BuildDate;

impl BuildDate {
    fn today() -> String {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        let (year, month, day) = Self::civil(seconds / 86_400);
        format!("{year:04}-{month:02}-{day:02}")
    }

    // days since 1970-01-01 to a (year, month, day) in the proleptic Gregorian calendar
    fn civil(days: u64) -> (i64, u32, u32) {
        let shifted = days as i64 + 719_468;
        let era = shifted.div_euclid(146_097);
        let day_of_era = shifted.rem_euclid(146_097);
        let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
        let month = (if month_index < 10 { month_index + 3 } else { month_index - 9 }) as u32;
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        (year, month, day)
    }
}
