/// Maps a (category, year) pair to a generation bucket.
///
/// The bucket is a small integer (1, 2, 3 …) for models with known generation
/// data, or the year itself for unknown models. Because real model years are
/// always > 1000 and generation numbers are always < 100, the two namespaces
/// never overlap in a HashMap key.
///
/// Add a new arm here whenever a new model category is added to main.rs.
pub fn generation_bucket(category: &str, year: u16) -> u16 {
    match category {
        // Yamaha MT-09 / MT-09 SP
        //   Gen 1 — 2013–2016  RN29  847cc, original CP3
        //   Gen 2 — 2017–2020  RN43  847cc, new frame, quickshifter std
        //   Gen 3 — 2021–2023  RN69  890cc, all-new frame + electronics
        //   Gen 4 — 2024+      RN86  new chassis, TFT dash
        "MT09" => match year {
            2013..=2016 => 1,
            2017..=2020 => 2,
            2021..=2023 => 3,
            2024.. => 4,
            _ => year,
        },

        // Yamaha Ténéré 700 / World Raid
        //   Gen 1 — 2019–2021  CP2 launch, Euro 4
        //   Gen 2 — 2022–2024  Euro 5, World Raid variant added
        //   Gen 3 — 2025+      updated suspension + connectivity
        "Tenere_700" => match year {
            2019..=2021 => 1,
            2022..=2024 => 2,
            2025.. => 3,
            _ => year,
        },

        // Unknown model — fall back to year so scoring still works
        _ => year,
    }
}
