#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub fn filter_nulls(val: &mut serde_json::Value) -> Result<bool>{
    match val{
        serde_json::Value::Null => {return Ok(true);}
        serde_json::Value::Bool(_) => {}
        serde_json::Value::Number(_) => {}
        serde_json::Value::String(_) => {}
        serde_json::Value::Array(a) => {
            if a.is_empty(){
                return Ok(true);
            }
            for v in a{
                filter_nulls(v)?;
            }
        }
        serde_json::Value::Object(o) => {
            if o.is_empty(){
                return Ok(true);
            }
            let mut candidates = vec![];
            for (k, v) in o.iter_mut() {
                if filter_nulls(v)?{
                    candidates.push(k.to_string());
                }
            }
            for c in candidates{
                o.remove(&c);
            }
        }
    }
    Ok(false)
}


#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let test_str = r###"{
"adversary": "Double Guns",
"category": "Malware",
"description": "Recently, our DNS data based threat monitoring system DNSmon flagged a suspicious domain pro.csocools.com. The system estimates the scale of infection may well above hundreds of thousands of users. By analyzing the related samples and C2s,\n\nWe traced its family back to the ShuangQiang (double gun) campaign, in the past, this campaign has been exposed by multiple security vendors, but it has revived and come back with new methods and great force.",
"domain": null,
"indicator": "New activity of DoubleGuns Group, control hundreds of thousands of bots via public cloud service",
"industries": null,
"intel": "Threat",
"ip_address": null,
"killchain": "Installation",
"malware_families": [
"Sakula - S0074",
"Double-Gun"

],
"md5": null,
"sha256": "c323d49f16e6ad3a8f3f1ca78249385d703db2e33722476424ac3536f7043748",
"targeted_countries": [],
"threat_type": "filehash",
"tlp": "white",
"url": null,
"submission_time": "2020-05-26 17:45:35 UTC"

}
"###;

        let test_str2 = r###"{
"adversary": "Double Guns",
"category": "Malware",
"description": "Recently, our DNS data based threat monitoring system DNSmon flagged a suspicious domain pro.csocools.com. The system estimates the scale of infection may well above hundreds of thousands of users. By analyzing the related samples and C2s,\n\nWe traced its family back to the ShuangQiang (double gun) campaign, in the past, this campaign has been exposed by multiple security vendors, but it has revived and come back with new methods and great force.",
"submission_time": "2020-05-26 17:45:35 UTC",
"indicator": "New activity of DoubleGuns Group, control hundreds of thousands of bots via public cloud service",
"targeted_countries": {},
"targeted_countriess": [],
"intel": "Threat",
"tlp": "white",
"killchain": "Installation",
"malware_families": [
"Sakula - S0074",
"Double-Gun"

],
"threat_type": "filehash",
"sha256": "c323d49f16e6ad3a8f3f1ca78249385d703db2e33722476424ac3536f7043748"

}
"###;

        let mut val1: serde_json::Value = serde_json::from_str(test_str).unwrap();
        let mut val2: serde_json::Value = serde_json::from_str(test_str2).unwrap();

        super::filter_nulls(&mut val1).unwrap();
        println!("{:#?}", val1);

        super::filter_nulls(&mut val2).unwrap();
        println!("{:#?}", val2);
    }

}
