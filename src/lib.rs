#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Debug)]
struct HashValue<'a>(pub &'a serde_json::Value);

impl Eq for HashValue<'_> {}

impl std::hash::Hash for HashValue<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use serde_json::Value::*;
        use std::hash::Hasher;
        match self.0 {
            Null => state.write_u32(3_221_225_473), // chosen randomly
            Bool(ref b) => b.hash(state),
            Number(ref n) => {
                if let Some(x) = n.as_u64() {
                    x.hash(state);
                } else if let Some(x) = n.as_i64() {
                    x.hash(state);
                } else if let Some(x) = n.as_f64() {
                    // `f64` does not implement `Hash`. However, floats in JSON are guaranteed to be
                    // finite, so we can use the `Hash` implementation in the `ordered-float` crate.
                    ordered_float::NotNan::new(x).unwrap().hash(state);
                }
            }
            String(ref s) => s.hash(state),
            Array(ref v) => {
                for x in v {
                    HashValue(x).hash(state);
                }
            }
            Object(ref map) => {
                let mut hash = 0;
                for (k, v) in map {
                    // We have no way of building a new hasher of type `H`, so we
                    // hardcode using the default hasher of a hash map.
                    let mut item_hasher = std::collections::hash_map::DefaultHasher::new();
                    k.hash(&mut item_hasher);
                    HashValue(v).hash(&mut item_hasher);
                    hash ^= item_hasher.finish();
                }
                state.write_u64(hash);
            }
        }
    }
}


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
            let mut vv = std::collections::HashSet::new();
            let mut candidates = vec![];
            for (i, v) in a.iter().enumerate(){
                if !vv.insert(HashValue(v)){
                    candidates.push(i);
                }
            }
            for i in candidates{
                a.remove(i);
            }
            for v in a.iter_mut(){
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

        let test_str3 = r###"[
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Delivery",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "url",
"tlp": "white",
"url": "http://zyzkikpfewuf.ru/XpqA02Df.exe",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Delivery",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "url",
"tlp": "white",
"url": "http://zyzkikpfewuf.ru/eSttPnHsmB.exe",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Delivery",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "url",
"tlp": "white",
"url": "http://zyzkikpfewuf.ru/esttpnhsmb.exe",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Delivery",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "url",
"tlp": "white",
"url": "http://zyzkikpfewuf.ru/hour84a6d9k.dotm",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Delivery",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "url",
"tlp": "white",
"url": "http://zyzkikpfewuf.ru/hour84a6d9k.exe",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Delivery",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "url",
"tlp": "white",
"url": "http://zyzkikpfewuf.ru/xpqa02df.exe",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "C2 Communication",
"malware_families": [
"Arkei"

],
"md5": "1e9311c594d49feba530c3ce815dfd2d",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "domain",
"tlp": "white",
"url": "https://rwwmefkauiaa.ru/",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Installation",
"malware_families": [
"Arkei"

],
"md5": "8e967ff97e36388934c5b2e7d63d714e",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "filehash",
"tlp": "white",
"url": "https://rwwmefkauiaa.ru/",
"threat_type": "Tandem Espionage"

},
{
"adversary": "",
"category": "Malware",
"description": "Some time ago, researchers discovered an interesting campaign distributing malicious documents. Which used the download chain as well as legitimate payload hosting services.",
"domain": "zyzkikpfewuf.ru",
"submission_time": "2022-05-27 08:50:17 UTC",
"intel": "Threat",
"ip_address": "162.33.179.235",
"killchain": "Installation",
"malware_families": [
"Arkei"

],
"md5": "8e967ff97e36388934c5b2e7d63d714e",
"sha256": "b9a1ac0335226386029bb3b6f9f3b9114bde55c7ea9f4fdcdccc02593208bdfd",
"source": "alienvault",
"indicator": "filehash",
"tlp": "white",
"url": "https://rwwmefkauiaa.ru/",
"threat_type": "Tandem Espionage"

}

]"###;

        let mut val1: serde_json::Value = serde_json::from_str(test_str).unwrap();
        let mut val2: serde_json::Value = serde_json::from_str(test_str2).unwrap();
        let mut val3: serde_json::Value = serde_json::from_str(test_str3).unwrap();

        super::filter_nulls(&mut val1).unwrap();
        println!("{:#?}", val1);

        super::filter_nulls(&mut val2).unwrap();
        println!("{:#?}", val2);

        super::filter_nulls(&mut val3).unwrap();
        println!("{:#?}", val3);
    }

}
