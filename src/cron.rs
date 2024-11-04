use chrono::{DateTime, Datelike, TimeZone, Timelike};

#[derive(Debug)]
pub struct Cron {
    minute: CronValue,
    hour: CronValue,
    day_of_month: CronValue,
    month: CronValue,
    day_of_week: CronValue,
}

impl Cron {
    pub fn parse(cron: &str) -> Result<Cron, CronError> {
        let parts: Vec<&str> = cron.split_whitespace().collect();
        let mut out = Cron {
            minute: CronValue::parse_part(parts[0])?,
            hour: CronValue::parse_part(parts[1])?,
            day_of_month: CronValue::parse_part(parts[2])?,
            month: CronValue::parse_part(parts[3])?,
            day_of_week: CronValue::parse_part(parts[4])?,
        };

        out.validate()?;

        Ok(out)
    }

    fn validate(&mut self) -> Result<(), CronError> {
        if self.minute.is_alternative()
            || self.hour.is_alternative()
            || self.day_of_month.is_alternative()
        {
            return Err(CronError::InvalidAlternative);
        }

        if self.month.is_alternative() {
            let alt = match &self.month {
                CronValue::Alternative(alt) => alt,
                _ => return Err(CronError::InvalidPart),
            };

            self.month = match alt.to_uppercase().as_str() {
                "SUN" => CronValue::Value(1),
                "MON" => CronValue::Value(2),
                "TUE" => CronValue::Value(3),
                "WED" => CronValue::Value(4),
                "THU" => CronValue::Value(5),
                "FRI" => CronValue::Value(6),
                "SAT" => CronValue::Value(7),
                _ => return Err(CronError::InvalidPart),
            }
        }

        if self.day_of_week.is_alternative() {
            let alt = match &self.month {
                CronValue::Alternative(alt) => alt,
                _ => return Err(CronError::InvalidPart),
            };

            self.month = match alt.to_uppercase().as_str() {
                "JAN" => CronValue::Value(1),
                "FEB" => CronValue::Value(2),
                "MAR" => CronValue::Value(3),
                "APR" => CronValue::Value(4),
                "MAY" => CronValue::Value(5),
                "JUN" => CronValue::Value(6),
                "JUL" => CronValue::Value(7),
                "AUG" => CronValue::Value(8),
                "SEP" => CronValue::Value(9),
                "OCT" => CronValue::Value(10),
                "NOV" => CronValue::Value(11),
                "DEC" => CronValue::Value(12),
                _ => return Err(CronError::InvalidPart),
            }
        }

        Ok(())
    }

    pub fn matches<Tz>(&self, time: &DateTime<Tz>) -> bool
    where
        Tz: TimeZone,
    {
        if self
            .day_of_week
            .matches_value(time.weekday().num_days_from_sunday())
            && self.month.matches_value(time.month())
            && self.day_of_month.matches_value(time.day())
            && self.hour.matches_value(time.hour())
            && self.minute.matches_value(time.minute())
        {
            return true;
        }
        false
    }
}

#[derive(Debug)]
pub enum CronError {
    InvalidRange(String),
    InvalidAlternative,
    InvalidPart,
}

#[derive(Debug)]
enum CronValue {
    Wildcard,
    Value(u32),
    Values(Vec<CronValue>),
    Range(u32, u32),
    Alternative(String),
}

impl CronValue {
    pub(super) fn is_alternative(&self) -> bool {
        match self {
            CronValue::Alternative(_) => true,
            _ => false,
        }
    }

    fn parse_part(part: &str) -> Result<CronValue, CronError> {
        if part.contains(",") {
            let values: Vec<CronValue> = part
                .split(",")
                .map(|v| CronValue::parse_part(v).unwrap())
                .collect();
            Ok(CronValue::Values(values))
        } else if part.contains("-") {
            let range: Vec<&str> = part.split("-").collect();
            if range.len() != 2 {
                return Err(CronError::InvalidRange(format!(
                    "Found {} values instead of 2",
                    range.len()
                )));
            }
            Ok(CronValue::Range(
                range[0].parse().unwrap(),
                range[1].parse().unwrap(),
            ))
        } else if part.contains("*") {
            Ok(CronValue::Wildcard)
        } else if part.chars().all(char::is_numeric) {
            Ok(CronValue::Value(part.parse().unwrap()))
        } else {
            Ok(CronValue::Alternative(part.to_string()))
        }
    }

    fn matches_value(&self, value: u32) -> bool {
        match self {
            CronValue::Wildcard => true,
            CronValue::Values(values) => values.iter().any(|v| v.matches_value(value)),
            CronValue::Range(start, end) => value >= *start && value <= *end,
            CronValue::Value(v) => value == *v,
            CronValue::Alternative(_) => false,
        }
    }
}
