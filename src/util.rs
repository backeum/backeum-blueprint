use scrypto::prelude::*;

// function to generate the url for the image
pub fn generate_url(
    base_path: String,
    donated: Decimal,
    created: String,
    collection_id: String,
) -> String {
    format!(
        "{}/nft/collection/{}?donated={}&created={}",
        base_path, collection_id, donated, created
    )
}

// function to generate the created string with a date format
pub fn generate_created_string() -> String {
    let time = UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap();
    let mut month = time.month().to_string();
    match time.month() {
        1 => month = "01".to_owned(),
        2 => month = "02".to_owned(),
        3 => month = "03".to_owned(),
        4 => month = "04".to_owned(),
        5 => month = "05".to_owned(),
        6 => month = "06".to_owned(),
        7 => month = "07".to_owned(),
        8 => month = "08".to_owned(),
        9 => month = "09".to_owned(),
        _ => {}
    }
    let mut day = time.day_of_month().to_string();
    match time.day_of_month() {
        1 => day = "01".to_owned(),
        2 => day = "02".to_owned(),
        3 => day = "03".to_owned(),
        4 => day = "04".to_owned(),
        5 => day = "05".to_owned(),
        6 => day = "06".to_owned(),
        7 => day = "07".to_owned(),
        8 => day = "08".to_owned(),
        9 => day = "09".to_owned(),
        _ => {}
    }
    format!("{}-{}-{}", time.year(), month, day)
}
