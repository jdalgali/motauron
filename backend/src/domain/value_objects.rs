pub fn generation_bucket(category: &str, year: u16) -> u16 {
    match category {
        "mt-09" => match year {
            2013..=2016 => 1,
            2017..=2020 => 2,
            2021..=2023 => 3,
            2024.. => 4,
            _ => year,
        },
        "tenere-700" => match year {
            2019..=2021 => 1,
            2022..=2024 => 2,
            2025.. => 3,
            _ => year,
        },
        _ => year,
    }
}
