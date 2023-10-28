use scrypto::prelude::*;

// function to generate the url for the image
pub fn generate_trophy_url(
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

// function to generate the url for the image
pub fn generate_membership_url(
    base_path: String,
    donated: Decimal,
    created: String,
    user_slug: String,
) -> String {
    format!(
        "{}/nft/membership/{}?donated={}&created={}",
        base_path, user_slug, donated, created
    )
}

// function to generate the url for the image
pub fn generate_creator_url(base_path: String, donated: Decimal, created: String) -> String {
    format!(
        "{}/nft/creator?donated={}&created={}",
        base_path, donated, created
    )
}

// function to generate the created string with a date format
pub fn generate_created_string(time: UtcDateTime) -> String {
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

// parse_created_string is a function that makes created string into a UtcDateTime
pub fn parse_created_string(input: String) -> UtcDateTime {
    let mut split = input.split('-');
    let year = split.next().unwrap();
    let month = split.next().unwrap();
    let day = split.next().unwrap();
    let year_int = year.parse::<u32>().unwrap();
    let month_int = month.parse::<u8>().unwrap();
    let day_int = day.parse::<u8>().unwrap();
    UtcDateTime::new(year_int, month_int, day_int, 0, 0, 0).unwrap()
}
