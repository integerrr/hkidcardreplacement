use chrono::NaiveDate;
use log::{error, info};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fmt::Debug, time::Duration};

const THIRTY_SECONDS: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Res {
    data: Vec<Data>,
    office: Vec<Office>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data {
    date: String,

    #[serde(rename = "quotaR")]
    #[serde(deserialize_with = "bool_from_str")]
    quota_r: bool,

    #[serde(rename = "officeId")]
    office_id: String,

    #[serde(deserialize_with = "bool_from_str")]
    #[serde(rename = "quotaK")]
    quota_k: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Office {
    #[serde(rename = "officeId")]
    office_id: String,
    eng: Eng,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Eng {
    district: String,
}

fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "quota-y" | "quota-g" => Ok(true),
        "quota-non" | "quota-r" => Ok(false),
        _ => Err(serde::de::Error::custom("Unknown quota string: {s:?}")),
    }
}

fn generate_office_id_map(v: Vec<Office>) -> HashMap<String, String> {
    v.into_iter()
        .map(|o| (o.office_id, o.eng.district))
        .collect()
}

fn main() -> color_eyre::Result<()> {
    env_logger::init();

    loop {
        std::thread::sleep(THIRTY_SECONDS);

        info!("Polling API..");
        let r = reqwest::blocking::get(
            "https://eservices.es2.immd.gov.hk/surgecontrolgate/ticket/getSituation",
        );
        let res = match r {
            Ok(resp) => resp,
            Err(e) => {
                error!("Error: {e}");
                continue;
            }
        };

        info!("{}", &res.status());
        match res.error_for_status_ref() {
            Ok(_res) => (),
            Err(_e) => continue,
        }

        let json = res.text()?;
        let r: Res = serde_json::from_str(&json)?;
        let id_map = generate_office_id_map(r.office.clone());

        let start_date_before_japan = NaiveDate::from_ymd_opt(2023, 6, 13).expect("hardcoded");
        let end_date_before_japan = NaiveDate::from_ymd_opt(2023, 6, 17).expect("hardcoded");
        let start_date_after_japan = NaiveDate::from_ymd_opt(2023, 6, 24).expect("hardcoded");
        let end_date_after_japan = NaiveDate::from_ymd_opt(2023, 7, 1).expect("hardcoded");
        let date_format = "%m/%d/%Y";

        for d in r.data {
            let naive_date = NaiveDate::parse_from_str(&d.date, date_format)?;
            let condition = (naive_date >= start_date_before_japan
                && naive_date <= end_date_before_japan
                && (d.quota_r || d.quota_k))
                || (naive_date >= start_date_after_japan
                    && naive_date <= end_date_after_japan
                    && (d.quota_r || d.quota_k));

            if !condition {
                continue;
            }
            println!("**********************************************");
            println!("**********************************************");
            println!("BOOKING AVAILABLE DIU DIU DIU DIU DIU DIU");
            println!("Date: {}", naive_date);
            println!(
                "Office: {}",
                id_map
                    .get(&d.office_id)
                    .cloned()
                    .expect("office_id should exist")
            );
            println!("https://www.gov.hk/tc/residents/immigration/idcard/hkic/bookregidcard.htm");
            println!("**********************************************");
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}
