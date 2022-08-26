use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::{borrow::Cow, collections::BTreeMap, fs::File, io, path::Path};

macro_rules! regex {
    ($re:expr) => {
        ::regex::Regex::new($re).unwrap()
    };
}

lazy_static! {
    static ref NEIGHBORING_BRACKETS: regex::Regex = regex!(r"((\[[^\]]*\])\s*){2,}");
    static ref DATA_IN_BRACKETS: regex::Regex = regex!(r"\[([^\]]+)\]");
}

pub struct RegexTransforms {
    full_path_regex: Vec<(Regex, String)>,
    base_file_regex: Vec<(Regex, String)>,
    directory_regex: Vec<(Regex, String)>,
    post_processing_regex: Vec<(Regex, String)>,
}

impl RegexTransforms {
    pub fn load(file_path: &Path) -> io::Result<RegexTransforms> {
        match File::open(file_path) {
            Ok(file) => {
                let deserialized_data: Result<
                    BTreeMap<String, Vec<(String, String)>>,
                    serde_yaml::Error,
                > = serde_yaml::from_reader(&file);
                match deserialized_data {
                    Ok(data_map) => Ok(RegexTransforms {
                        full_path_regex: data_map["full_path_regex"]
                            .iter()
                            .map(|op| (regex!(&op.0), op.1.to_owned()))
                            .collect(),
                        base_file_regex: data_map["base_file_regex"]
                            .iter()
                            .map(|op| (regex!(&op.0), op.1.to_owned()))
                            .collect(),
                        directory_regex: data_map["directory_regex"]
                            .iter()
                            .map(|op| (regex!(&op.0), op.1.to_owned()))
                            .collect(),
                        post_processing_regex: data_map["post_processing_regex"]
                            .iter()
                            .map(|op| (regex!(&op.0), op.1.to_owned()))
                            .collect(),
                    }),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn transform_file_path(&self, file_path: &str) -> String {
        // Run regular expressions over the full path
        let mut full_path_ref = Cow::Borrowed(file_path);
        for (regex, replacement) in self.full_path_regex.iter() {
            full_path_ref = Cow::Owned(regex.replace_all(&full_path_ref, replacement).into_owned());
        }
        let split_path: &Vec<&str> = &full_path_ref.split('/').collect::<Vec<&str>>();
        // Run regular expressions over the base file
        let mut file_name: Cow<str> = Cow::Borrowed(*split_path.last().unwrap());
        for (regex, replacement) in self.base_file_regex.iter() {
            file_name = Cow::Owned(regex.replace_all(&file_name, replacement).into_owned());
        }
        file_name = RegexTransforms::join_bracketed_data(file_name);
        // Run regular expressions over the directories
        let directories = split_path[0..split_path.len() - 1].join("/");
        let mut directories_ref = Cow::Borrowed(&directories);
        for (regex, replacement) in self.directory_regex.iter() {
            directories_ref = Cow::Owned(
                regex
                    .replace_all(&directories_ref, replacement)
                    .into_owned(),
            );
        }
        // Run post-processing regular expressions on the full path
        let full_path = format!("{}/{}", directories_ref, file_name);
        full_path_ref = Cow::Borrowed(&full_path);
        for (regex, replacement) in self.post_processing_regex.iter() {
            full_path_ref = Cow::Owned(regex.replace_all(&full_path_ref, replacement).into_owned());
        }
        full_path_ref.into_owned()
    }

    fn join_bracketed_data(data: Cow<str>) -> Cow<str> {
        // Join bracketed information into one bracket list
        Cow::Owned(
            NEIGHBORING_BRACKETS
                .replace_all(&data, |caps: &Captures| {
                    format!(
                        "[{}]",
                        DATA_IN_BRACKETS
                            .captures_iter(&caps[0])
                            .map(|c| Cow::Borrowed(&c[1]).into_owned())
                            .fold(String::new(), |mut a, b| {
                                a.reserve(&b.len() + 1);
                                a.push_str(&b);
                                a.push(' ');
                                a
                            })
                            .trim_end()
                    )
                })
                .into_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_tv_shows() {
        let data: [(&str, &str); 8] = [
            (
                "TV Shows/11.22.63 - Stephen King 8 Part Mini Series 2016 Eng Ita Multi-Subs 1080p [H264-mp4]/Subtitles - [SRT-MicroDVD]/08 11.22.63 The Day In Question - English.srt",
                "TV Shows/11 22 63 (2016)/08 11 22 63 The Day In Question.en.srt"
            ),
            (
                "TV Shows/THE VAMPIRE DIARIES SEASON 8 COMPLETE S08 1080P HEVC DD51 BLUURY/The Vampire Diaries 08x01  Hello Brother.mkv",
                "TV Shows/THE VAMPIRE DIARIES/Season 08/The Vampire Diaries - S08E01 - Hello Brother.mkv"
            ),
            (   "TV Shows/Atlanta Season 1 Mp4 1080p/Atlanta S01E01.mp4", 
                "TV Shows/Atlanta/Season 01/Atlanta - S01E01 -.mp4"
            ),
            (
                "TV Shows/House.of.the.Dragon.S01E01.The.Heirs.of.the.Dragon.1080p.WEB-DL.DD5.1.H.264-MiU[TGx]/House.of.the.Dragon.S01E01.The.Heirs.of.the.Dragon.1080p.WEB-DL.DD5.1.H.264-MiU.mkv",
                "TV Shows/House of the Dragon/Season 01/House of the Dragon - S01E01 - The Heirs of the Dragon [1080p h264].mkv"
            ),
            (
                "TV Shows/Psych (2006) Season 1-8 S01-S08 (1080p AMZN WEB-DL x265 HEVC 10bit AAC 5.1 MONOLITH) REPACK/Season 1/Psych (2006) - S01E01 - Pilot (1080p AMZN WEB-DL x265 MONOLITH).mkv",
                "TV Shows/Psych (2006)/Season 01/Psych (2006) - S01E01 - Pilot [1080p x265].mkv"
            ),
            (
                "TV Shows/Psych (2006) Season 1-8 S01-S08 (1080p AMZN WEB-DL x265 HEVC 10bit AAC 5.1 MONOLITH) REPACK/Season 5/Psych (2006) - S05E14 - The Polarizing Express (1080p AMZN WEB-DL x265 MONOLITH).mkv",
                "TV Shows/Psych (2006)/Season 05/Psych (2006) - S05E14 - The Polarizing Express [1080p x265].mkv"
            ),
            (
                "TV Shows/Red Dwarf S01-S12 1988-2020 1080p BluRay HEVC x265 BONE/Season 1/Red Dwarf S01E01 The End 1080p BluRay HEVC x265 BONE.mkv",
                "TV Shows/Red Dwarf/Season 01/Red Dwarf - S01E01 - The End [1080p x265].mkv"
            ),
            (
                "TV Shows/Red Dwarf S01-S12 1988-2020 1080p BluRay HEVC x265 BONE/Season 3/Red Dwarf S03E02 Marooned 1080p BluRay HEVC x265 BONE.mkv",
                "TV Shows/Red Dwarf/Season 03/Red Dwarf - S03E02 - Marooned [1080p x265].mkv"
            ),
        ];

        for (input_file_path, expected) in data {
            let regex_path = Path::new("example/transforms.yaml");
            let regex_transforms = RegexTransforms::load(regex_path).unwrap();
            let result = regex_transforms.transform_file_path(input_file_path);
            assert_eq!(expected, result, "\n\tInput: {}", input_file_path);
        }
    }
}
