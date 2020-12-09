use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

type Year = usize;
type Color = String;
#[derive(Debug)]
enum Unit {
    Centimeter,
    Inch,
    None,
}
impl std::default::Default for Unit {
    fn default() -> Self {
        Unit::None
    }
}

impl std::str::FromStr for Unit {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unit = match s {
            "cm" => Unit::Centimeter,
            "in" => Unit::Inch,
            _ => Unit::None,
        };
        Ok(unit)
    }
}

#[derive(Debug)]
enum Property {
    Id(String),
    CountryId(String),
    BirthYear(Year),
    IssueYear(Year),
    ExpirationYear(Year),
    Height(usize, Unit),
    HairColor(Color),
    EyeColor(Color),
    EndPasseport,
}

impl std::str::FromStr for Property {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Property::EndPasseport);
        }
        let mut split = s.split(':');
        let property = match (split.next(), split.next()) {
            (Some("pid"), Some(pid)) => Property::Id(pid.to_owned()),
            (Some("cid"), Some(cid)) => Property::CountryId(cid.to_owned()),
            (Some("byr"), Some(byr)) => Property::BirthYear(
                byr.parse()
                    .expect(format!("expect '{}' to be parseable as a usize", byr).as_str()),
            ),
            (Some("iyr"), Some(iyr)) => Property::IssueYear(
                iyr.parse()
                    .expect(format!("expect '{}' to be parseable as a usize", iyr).as_str()),
            ),
            (Some("eyr"), Some(eyr)) => Property::ExpirationYear(
                eyr.parse()
                    .expect(format!("expect '{}' to be parseable as a usize", eyr).as_str()),
            ),
            (Some("hgt"), Some(hgt)) => {
                let unit: Unit = hgt[hgt.len() - 2..].parse().unwrap_or(Unit::None);
                let last_index = if let Unit::None = unit {
                    hgt.len()
                } else {
                    hgt.len() - 2
                };
                let height: usize = hgt[..last_index]
                    .parse()
                    .expect("expect height to be parseable as a usize");
                Property::Height(height, unit)
            }
            (Some("hcl"), Some(hcl)) => Property::HairColor(hcl.to_owned()),
            (Some("ecl"), Some(ecl)) => Property::EyeColor(ecl.to_owned()),
            (Some(p), Some(v)) => panic!("unknown property '{}' with value '{}'", p, v),
            (Some(p), None) => panic!("unknown property '{}' with no value", p),
            (None, Some(v)) => panic!("value '{}' has no property name", v),
            (None, None) => unreachable!(),
        };
        Ok(property)
    }
}

struct Properties<I>
where
    I: Iterator<Item = String>,
{
    stream: I,
    split: Option<Vec<String>>,
}

impl<I> Properties<I>
where
    I: Iterator<Item = String>,
{
    fn new(stream: I) -> Self {
        Self {
            stream,
            split: None,
        }
    }
}

impl<I> Iterator for Properties<I>
where
    I: Iterator<Item = String>,
{
    type Item = Property;
    fn next(&mut self) -> Option<Self::Item> {
        let property_str =
            if let Some(Some(property_str)) = self.split.as_mut().map(|split| split.pop()) {
                // Still have some element in the splitted line
                property_str
            } else {
                // No more element, need to get a new splitted line from stream
                if let Some(line) = self.stream.next() {
                    let mut split_whitespace = line
                        .split_whitespace()
                        .rev()
                        .map(std::borrow::ToOwned::to_owned)
                        .collect::<Vec<String>>();
                    if let Some(property_str) = split_whitespace.pop() {
                        self.split = Some(split_whitespace);
                        property_str
                    } else {
                        // If the new splitted line is empty, then it marks the end
                        // of a passeport
                        return Some(Property::EndPasseport);
                    }
                } else {
                    // No more line in the stream, end of the iteration
                    return None;
                }
            };
        property_str.parse().ok()
    }
}

#[derive(Default)]
struct PasseportBuilder {
    id: Option<String>,
    country_id: Option<String>,
    birth_year: Option<usize>,
    issue_year: Option<usize>,
    expiration_year: Option<usize>,
    height: Option<(usize, Unit)>,
    hair_color: Option<String>,
    eye_color: Option<String>,
}
impl PasseportBuilder {
    fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    fn country_id(mut self, country_id: String) -> Self {
        self.country_id = Some(country_id);
        self
    }
    fn birth_year(mut self, birth_year: usize) -> Self {
        self.birth_year = Some(birth_year);
        self
    }
    fn issue_year(mut self, issue_year: usize) -> Self {
        self.issue_year = Some(issue_year);
        self
    }
    fn expiration_year(mut self, expiration_year: usize) -> Self {
        self.expiration_year = Some(expiration_year);
        self
    }
    fn height(mut self, height: usize, unit: Unit) -> Self {
        self.height = Some((height, unit));
        self
    }
    fn hair_color(mut self, hair_color: String) -> Self {
        self.hair_color = Some(hair_color);
        self
    }
    fn eye_color(mut self, eye_color: String) -> Self {
        self.eye_color = Some(eye_color);
        self
    }
    fn build(self) -> Result<Passeport, String> {
        let passerport = Passeport {
            id: self.id.ok_or_else(|| "expect an 'id'".to_string())?,
            country_id: self.country_id,
            birth_year: self
                .birth_year
                .ok_or_else(|| "expect an 'birth_year'".to_string())?,
            issue_year: self
                .issue_year
                .ok_or_else(|| "expect an 'issue_year'".to_string())?,
            expiration_year: self
                .expiration_year
                .ok_or_else(|| "expect an 'expiration_year'".to_string())?,
            height: self
                .height
                .ok_or_else(|| "expect an 'height'".to_string())?,
            hair_color: self
                .hair_color
                .ok_or_else(|| "expect an 'hair_color'".to_string())?,
            eye_color: self
                .eye_color
                .ok_or_else(|| "expect an 'eye_color'".to_string())?,
        };
        Ok(passerport)
    }
}

#[derive(Debug, Default)]
struct Passeport {
    id: String,
    country_id: Option<String>,
    birth_year: usize,
    issue_year: usize,
    expiration_year: usize,
    height: (usize, Unit),
    hair_color: String,
    eye_color: String,
}

impl Passeport {
    fn check_id(&self) -> bool {
        self.id.len() == 9
    }
    fn check_birth_year(&self) -> bool {
        if self.birth_year >= 1920 && self.birth_year <= 2002 {
            true
        } else {
            false
        }
    }
    fn check_issue_year(&self) -> bool {
        if self.issue_year >= 2010 && self.issue_year <= 2020 {
            true
        } else {
            false
        }
    }
    fn check_expiration_year(&self) -> bool {
        if self.expiration_year >= 2020 && self.expiration_year <= 2030 {
            true
        } else {
            false
        }
    }
    fn check_height(&self) -> bool {
        match self.height.1 {
            Unit::Centimeter => {
                if self.height.0 >= 150 && self.height.0 <= 193 {
                    true
                } else {
                    false
                }
            }
            Unit::Inch => {
                if self.height.0 >= 59 && self.height.0 <= 76 {
                    true
                } else {
                    false
                }
            }
            Unit::None => false,
        }
    }
    fn check_hair_color(&self) -> bool {
        if self.hair_color.len() != 7 {
            return false;
        }
        let mut chars = self.hair_color.chars();
        if chars.next().unwrap() != '#' {
            return false;
        }
        for c in chars {
            match c {
                '0'..='9' | 'a'..='f' => continue,
                _ => return false,
            }
        }
        true
    }
    fn check_eye_color(&self) -> bool {
        match self.eye_color.as_str() {
            "amb" | "blu" | "brn" | "gry" | "grn" | "hzl" | "oth" => true,
            _ => false,
        }
    }
    fn check(&self) -> bool {
        self.check_id()
            && self.check_birth_year()
            && self.check_issue_year()
            && self.check_expiration_year()
            && self.check_height()
            && self.check_hair_color()
            && self.check_eye_color()
    }
}

struct Passeports<I>
where
    I: Iterator<Item = Property>,
{
    properties: I,
}

impl<I> Passeports<I>
where
    I: Iterator<Item = Property>,
{
    fn new(properties: I) -> Self {
        Self { properties }
    }
}

impl<I> std::iter::Iterator for Passeports<I>
where
    I: Iterator<Item = Property>,
{
    type Item = Passeport;
    fn next(&mut self) -> Option<Self::Item> {
        let mut builder = PasseportBuilder::default();
        for property in &mut self.properties {
            builder = match property {
                Property::Id(id) => builder.id(id),
                Property::CountryId(country_id) => builder.country_id(country_id),
                Property::BirthYear(birth_year) => builder.birth_year(birth_year),
                Property::IssueYear(issue_year) => builder.issue_year(issue_year),
                Property::ExpirationYear(expiration_year) => {
                    builder.expiration_year(expiration_year)
                }
                Property::Height(height, unit) => builder.height(height, unit),
                Property::HairColor(hair_color) => builder.hair_color(hair_color),
                Property::EyeColor(eye_color) => builder.eye_color(eye_color),
                Property::EndPasseport => {
                    if let Ok(passeport) = builder.build() {
                        return Some(passeport);
                    } else {
                        PasseportBuilder::default()
                    }
                }
            };
        }
        builder.build().ok()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let properties = Properties::new(
        reader
            .lines()
            .map(|line| line.expect("expect line to be parseable as a String")),
    );
    let passeports = Passeports::new(properties);
    let (complete, valid) = passeports.fold((0, 0), |(mut complete, mut valid), passeport| {
        complete += 1;
        if passeport.check() {
            valid += 1;
        }
        (complete, valid)
    });
    println!("Number of complete passeports is {}", complete);
    println!("Number of valid passeports is {}", valid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_passeports() {
        let passeports = r#"ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in"#;
        let properties =
            Properties::new(passeports.split('\n').map(std::borrow::ToOwned::to_owned));
        let passeports = Passeports::new(properties);
        assert_eq!(2, passeports.count());
    }

    #[test]
    fn invalid_passeports() {
        let passeports = r#"eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007"#;
        let properties =
            Properties::new(passeports.split('\n').map(std::borrow::ToOwned::to_owned));
        let passeports = Passeports::new(properties);
        assert_eq!(0, passeports.filter(Passeport::check).count());
    }

    #[test]
    fn valid_passeports() {
        let passeports = r#"pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719"#;
        let properties =
            Properties::new(passeports.split('\n').map(std::borrow::ToOwned::to_owned));
        let passeports = Passeports::new(properties);
        assert_eq!(4, passeports.filter(Passeport::check).count());
    }

    #[test]
    fn check_id() {
        for (id, expected) in vec![
            ("000000001", true),
            ("123456789", true),
            ("0123456789", false),
        ] {
            let passeport = Passeport {
                id: String::from(id),
                ..Default::default()
            };
            assert_eq!(expected, passeport.check_id());
        }
    }

    #[test]
    fn check_birth_year() {
        for (birth_year, expected) in vec![(2002, true), (2003, false)] {
            let passeport = Passeport {
                birth_year,
                ..Default::default()
            };
            assert_eq!(expected, passeport.check_birth_year());
        }
    }

    #[test]
    fn check_height() {
        use Unit::*;
        for (height, expected) in vec![
            ((60, Inch), true),
            ((190, Centimeter), true),
            ((190, Inch), false),
            ((190, None), false),
        ] {
            let passeport = Passeport {
                height,
                ..Default::default()
            };
            assert_eq!(expected, passeport.check_height());
        }
    }

    #[test]
    fn check_hair_color() {
        for (hair_color, expected) in vec![("#123abc", true), ("#123abz", false), ("123abc", false)]
        {
            let passeport = Passeport {
                hair_color: String::from(hair_color),
                ..Default::default()
            };
            assert_eq!(expected, passeport.check_hair_color());
        }
    }

    #[test]
    fn check_eye_color() {
        for (eye_color, expected) in vec![("brn", true), ("wat", false)] {
            let passeport = Passeport {
                eye_color: String::from(eye_color),
                ..Default::default()
            };
            assert_eq!(expected, passeport.check_eye_color());
        }
    }
}
