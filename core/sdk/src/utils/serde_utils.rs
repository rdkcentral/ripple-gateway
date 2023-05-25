// If not stated otherwise in this file or this component's license file the
// following copyright and licenses apply:
//
// Copyright 2023 RDK Management
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use serde::{self, Deserialize, Deserializer};

use regex::Regex;
enum Patterns {
    Language,
    Timezone,
}

fn pattern_matches(pattern: Patterns, str: &String) -> bool {
    Regex::new(pattern.as_str()).unwrap().is_match(str.as_str())
}

impl Patterns {
    fn as_str(&self) -> &'static str {
        match self {
            Patterns::Language => "^[A-Za-z]{2}$",
            Patterns::Timezone => "^[-+_/ A-Za-z 0-9]*$",
        }
    }
}

pub mod opacity_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *value > 100 {
            Err(serde::ser::Error::custom(
                "Invalid value for Opacity. Value should be between 0 and 100 inclusive",
            ))
        } else {
            serializer.serialize_u32(*value)
        }
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = u32::deserialize(deserializer)?;
        if num > 100 {
            Err(serde::de::Error::custom(
                "Invalid value for Opacity. Value should be between 0 and 100 inclusive",
            ))
        } else {
            Ok(num)
        }
    }
}

pub mod language_code_serde {
    use super::{pattern_matches, Patterns};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(str: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if pattern_matches(Patterns::Language, str) {
            serializer.serialize_str(&str)
        } else {
            Err(serde::ser::Error::custom(
                "Language code is not of the format specified in ISO 639",
            ))
        }
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        if pattern_matches(Patterns::Language, &str) {
            Ok(str)
        } else {
            Err(serde::de::Error::custom(
                "Language code is not of the format specified in ISO 639",
            ))
        }
    }
}

pub mod optional_language_code_serde {
    use super::language_code_serde;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(data: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(str) = data {
            language_code_serde::serialize(str, serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        language_code_serde::deserialize(deserializer).map(|res| Some(res))
    }
}

pub mod optional_language_code_list_serde {
    use super::{pattern_matches, Patterns};
    use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(list_of_lang) = data {
            let mut seq = serializer.serialize_seq(Some(list_of_lang.len()))?;
            for str in list_of_lang {
                seq.serialize_element(str)?;
            }
            seq.end()
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Vec<String> = Vec::deserialize(deserializer)?;
        for elem in &s {
            if !pattern_matches(Patterns::Language, elem) {
                return Err(serde::de::Error::custom(
                    "One or more language is not of the ISO 639 format",
                ));
            }
        }
        Ok(Some(s))
    }
}

pub mod date_time_str_serde {
    use chrono::{TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S.%3fZ";

    pub fn serialize<S>(data: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formed_date_res = Utc.datetime_from_str(&data, FORMAT);
        if let Ok(_) = formed_date_res {
            serializer.serialize_str(&data)
        } else {
            Err(serde::ser::Error::custom(
                "String not convertible to date-time",
            ))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        let formed_date_res = Utc.datetime_from_str(&str, FORMAT);
        if let Ok(_) = formed_date_res {
            Ok(str)
        } else {
            Err(serde::de::Error::custom(
                "Field not in expected Date-Time (YYYY-MM-DDTHH:mm:SS.xxxZ) format",
            ))
        }
    }
}

pub mod optional_date_time_str_serde {
    use super::date_time_str_serde;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(date: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(data) = date {
            date_time_str_serde::serialize(data, serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        date_time_str_serde::deserialize(deserializer).map(|data| Some(data))
    }
}

pub mod timezone_serde {
    use super::{pattern_matches, Patterns};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(str: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if pattern_matches(Patterns::Timezone, str) {
            serializer.serialize_str(&str)
        } else {
            Err(serde::ser::Error::custom(
                "Timezone is not in a format supported by the IANA TZ database",
            ))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        if pattern_matches(Patterns::Timezone, &str) {
            Ok(str)
        } else {
            Err(serde::de::Error::custom(
                "Timezone is not in a format supported by the IANA TZ database",
            ))
        }
    }
}

pub fn progress_value_deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value: f32 = f32::deserialize(deserializer)?;
    if value < 0.0 {
        Err(serde::de::Error::custom(
            "Invalid value for progress. Minimum value should be 0.0",
        ))
    } else {
        Ok(value)
    }
}