use anyhow::{bail, Result};
use std::fs::File;

use vibrato::{Dictionary, Tokenizer};

pub struct RawToken {
    pub surface: String,
    pub feature: String,
}

impl From<vibrato::token::Token<'_, '_>> for RawToken {
    fn from(value: vibrato::token::Token) -> Self {
        Self {
            surface: value.surface().into(),
            feature: value.feature().into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreparedToken {
    literal: String,
    pos: POS,
    pos2: POS,
    pos3: POS,
    pos4: POS,
    inflection_type: POS,
    inflection_form: POS,
    lemma: String,
    reading: String,
    hatsuon: String,
}

#[derive(PartialEq, Clone, Debug)]
enum POS {
    Meishi,
    KoyuuMeishi,
    DaiMeishi,
    JoDoushi,
    Kazu,
    Joshi,
    Settoushi,
    Doushi,
    Kigou,
    Firaa,
    Sonota,
    Kandoushi,
    Rentaishi,
    Setsuzokushi,
    Fukushi,
    Setsuzokujoshi,
    Keiyoushi,
    Hijiritsu,
    Fukushikanou,
    Sahensetsuzoku,
    Keiyoudoushigokan,
    Naikeiyoushigokan,
    Jodoushigokan,
    Fukushika,
    Taigensetsuzoku,
    Rentaika,
    Tokushu,
    Setsubi,
    Setsuzokushiteki,
    Doushihijiritsuteki,
    SahenSuru,
    TokushuTa,
    TokushuNai,
    TokushuTai,
    TokushuDesu,
    TokushuDa,
    TokushuMasu,
    TokushuNu,
    Fuhenkagata,
    Jinmei,
    MeireiI,
    Kakarijoshi,

    Unset,
    Unknown,
}

impl From<&str> for POS {
    fn from(value: &str) -> Self {
        match value {
            "名詞" => Self::Meishi,
            "固有名詞" => Self::KoyuuMeishi,
            "代名詞" => Self::DaiMeishi,
            "助動詞" => Self::JoDoushi,
            "数" => Self::Kazu,
            "助詞" => Self::Joshi,
            "接頭詞" => Self::Settoushi,
            "動詞" => Self::Doushi,
            "記号" => Self::Kigou,
            "フィラー" => Self::Firaa,
            "その他" => Self::Sonota,
            "感動詞" => Self::Kandoushi,
            "連体詞" => Self::Rentaishi,
            "接続詞" => Self::Setsuzokushi,
            "副詞" => Self::Fukushi,
            "接続助詞" => Self::Setsuzokujoshi,
            "形容詞" => Self::Keiyoushi,
            "非自立" => Self::Hijiritsu,
            "副詞可能" => Self::Fukushikanou,
            "サ変接続" => Self::Sahensetsuzoku,
            "形容動詞語幹" => Self::Keiyoudoushigokan,
            "ナイ形容詞語幹" => Self::Naikeiyoushigokan,
            "助動詞語幹" => Self::Jodoushigokan,
            "副詞化" => Self::Fukushika,
            "体言接続" => Self::Taigensetsuzoku,
            "連体化" => Self::Rentaika,
            "特殊" => Self::Tokushu,
            "接尾" => Self::Setsubi,
            "接続詞的" => Self::Setsuzokushiteki,
            "動詞非自立的" => Self::Doushihijiritsuteki,
            "サ変・スル" => Self::SahenSuru,
            "特殊・タ" => Self::TokushuTa,
            "特殊・ナイ" => Self::TokushuNai,
            "特殊・タイ" => Self::TokushuTai,
            "特殊・デス" => Self::TokushuDesu,
            "特殊・ダ" => Self::TokushuDa,
            "特殊・マス" => Self::TokushuMasu,
            "特殊・ヌ" => Self::TokushuNu,
            "不変化型" => Self::Fuhenkagata,
            "人名" => Self::Jinmei,
            "命令ｉ" => Self::MeireiI,
            "係助詞" => Self::Kakarijoshi,
            "*" => Self::Unset,
            _ => Self::Unknown,
        }
    }
}

const NA: &str = "な";
const NI: &str = "に";
const TE: &str = "て";
const DE: &str = "で";
const BA: &str = "ば";
const NN: &str = "ん";
const SA: &str = "さ";

#[derive(Debug)]

pub struct Word {
    pub word: String,
    pub lemma: String, // dictionary form
    pub part_of_speech: PartOfSpeech,
    pub tokens: Vec<PreparedToken>,
    pub extra: WordExtra,
}

#[derive(Debug)]
pub struct WordExtra {
    pub reading: String,
    pub transcription: String,
    pub grammar: Option<Grammar>,
}

#[derive(PartialEq, Debug)]
pub enum PartOfSpeech {
    Noun,
    ProperNoun,
    Pronoun,
    Adjective,
    Adverb,
    Determiner,
    Preposition,
    Postposition,
    Verb,
    Suffix,
    Prefix,
    Conjunction,
    Interjection,
    Number,
    Unknown,
    Symbol,
    Other,
}

#[derive(Debug)]
pub enum Grammar {
    Auxillary,
    Nominal,
}

pub fn prepare_tokens(raw_tokens: Vec<RawToken>) -> Result<Vec<PreparedToken>> {
    raw_tokens.into_iter().map(|raw_token| {
        let features: Vec<&str> = raw_token.feature.split(',').collect();

        let [pos, pos2, pos3, pos4, inflection_type, inflection_form] = features[..6] else {
            bail!("Couldn't read all features from token. Make sure you're using an IPADIC dictionary")
        };

        let lemma: &str = features.get(7).unwrap_or(&"");
        let reading: &str = features.get(8).unwrap_or(&"");
        let hatsuon: &str = features.get(9).unwrap_or(&"");

        let parsed_pos = POS::from(pos);
        let parsed_pos2 = POS::from(pos2);
        let parsed_pos3 = POS::from(pos3);
        let parsed_pos4 = POS::from(pos4);
        let parsed_inf_type = POS::from(inflection_type);
        let parsed_inf_form = POS::from(inflection_form);

        if(parsed_pos == POS::Unset) { bail!("The main POS of token '{}' couldn't be identified", raw_token.surface);}
        // if(parsed_pos2 == POS::Unknown) { bail!("The POS 2nd level of token '{}' couldn't be identified", raw_token.surface);}
        // if(parsed_pos3 == POS::Unknown) { bail!("The POS 3rd level of token '{}' couldn't be identified", raw_token.surface);}
        // if(parsed_pos4 == POS::Unknown) { bail!("The POS 4th level of token '{}' couldn't be identified", raw_token.surface);}
        // if(parsed_inf_type == POS::Unknown) { bail!("The inflection type of token '{}' couldn't be identified", raw_token.surface);}
        // if(parsed_inf_form == POS::Unknown) { bail!("The inflection form of token '{}' couldn't be identified", raw_token.surface);}

        Ok(PreparedToken {
            literal: raw_token.surface,
            pos: parsed_pos,
            pos2: parsed_pos2,
            pos3: parsed_pos3,
            pos4: parsed_pos4,
            inflection_type: parsed_inf_type,
            inflection_form: parsed_inf_form,
            lemma: lemma.into(),
            reading:  reading.into(),
            hatsuon: hatsuon.into(),
        })

    }).collect()
}

pub fn parse_into_words(tokens: Vec<PreparedToken>) -> Result<Vec<Word>> {
    let mut words: Vec<Word> = Vec::new();
    let mut iter = tokens.iter().peekable();
    let mut previous: Option<PreparedToken> = None;

    while let Some(token) = iter.next() {
        let mut pos: Option<PartOfSpeech> = None;
        let mut grammar: Option<Grammar> = None;
        let mut eat_next = false;
        let mut eat_lemma = false;
        let mut attach_to_previous = false;
        let mut also_attach_to_lemma = false;
        let mut update_pos = false;

        match token.pos {
            POS::Meishi => {
                pos = Some(PartOfSpeech::Noun);
                match token.pos2 {
                    POS::KoyuuMeishi => {
                        pos = Some(PartOfSpeech::ProperNoun);
                    }
                    POS::DaiMeishi => {
                        pos = Some(PartOfSpeech::Pronoun);
                    }
                    POS::Fukushikanou
                    | POS::Sahensetsuzoku
                    | POS::Keiyoudoushigokan
                    | POS::Naikeiyoushigokan => {
                        if let Some(following) = iter.peek() {
                            if following.inflection_type == POS::SahenSuru {
                                pos = Some(PartOfSpeech::Verb);
                                eat_next = true;
                            } else if following.inflection_type == POS::TokushuDa {
                                pos = Some(PartOfSpeech::Adjective);
                                if following.inflection_form == POS::Taigensetsuzoku {
                                    eat_next = true;
                                    eat_lemma = false;
                                }
                            } else if following.inflection_type == POS::TokushuNai {
                                pos = Some(PartOfSpeech::Adjective);
                                eat_next = true;
                            } else if following.pos == POS::Joshi && following.literal == NI {
                                pos = Some(PartOfSpeech::Adverb);
                                eat_next = false;
                            }
                        }
                    }
                    POS::Hijiritsu | POS::Tokushu => {
                        if let Some(following) = iter.peek() {
                            match token.pos3 {
                                POS::Fukushikanou => {
                                    if following.pos == POS::Joshi && following.literal == NI {
                                        pos = Some(PartOfSpeech::Adverb);
                                        eat_next = true;
                                    }
                                }
                                POS::Jodoushigokan => {
                                    if following.inflection_type == POS::TokushuDa {
                                        pos = Some(PartOfSpeech::Verb);
                                        grammar = Some(Grammar::Auxillary);

                                        if following.inflection_form == POS::Taigensetsuzoku {
                                            eat_next = true;
                                        }
                                    } else if following.pos == POS::Joshi
                                        && following.pos2 == POS::Fukushika
                                    {
                                        pos = Some(PartOfSpeech::Adverb);
                                        eat_next = true;
                                    }
                                }
                                POS::Keiyoudoushigokan => {
                                    pos = Some(PartOfSpeech::Adjective);
                                    if (following.inflection_type == POS::TokushuDa
                                        && following.inflection_form == POS::Taigensetsuzoku)
                                        || following.pos2 == POS::Rentaika
                                    {
                                        eat_next = true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    POS::Kazu => {
                        pos = Some(PartOfSpeech::Number);
                        if words.len() > 0
                            && words
                                .last()
                                .is_some_and(|w| w.part_of_speech == PartOfSpeech::Number)
                        {
                            attach_to_previous = true;
                            also_attach_to_lemma = true;
                        }
                    }
                    POS::Setsubi => {
                        if token.pos3 == POS::Jinmei {
                            pos = Some(PartOfSpeech::Suffix);
                        } else {
                            if token.pos3 == POS::Tokushu && token.lemma == SA {
                                update_pos = true;
                                pos = Some(PartOfSpeech::Noun);
                            } else {
                                also_attach_to_lemma = true;
                            }
                            attach_to_previous = true;
                        }
                    }
                    POS::Setsuzokushiteki => {
                        pos = Some(PartOfSpeech::Conjunction);
                    }
                    POS::Doushihijiritsuteki => {
                        pos = Some(PartOfSpeech::Verb);
                        grammar = Some(Grammar::Nominal)
                    }
                    _ => (),
                }
            }
            POS::Settoushi => {
                pos = Some(PartOfSpeech::Prefix);
            }
            POS::JoDoushi => {
                pos = Some(PartOfSpeech::Postposition);

                if (previous.is_none() || (previous.is_some_and(|p| p.pos2 != POS::Kakarijoshi)))
                    && [
                        POS::TokushuTa,
                        POS::TokushuNai,
                        POS::TokushuTai,
                        POS::TokushuMasu,
                        POS::TokushuNu,
                    ]
                    .contains(&token.inflection_type)
                {
                    attach_to_previous = true;
                } else if token.inflection_type == POS::Fuhenkagata && token.lemma == NN {
                    attach_to_previous = true;
                } else if [POS::TokushuDa, POS::TokushuDesu].contains(&token.inflection_type)
                    && token.literal != NA
                {
                    pos = Some(PartOfSpeech::Verb)
                }
            }
            POS::Doushi => {
                pos = Some(PartOfSpeech::Verb);
                if token.pos2 == POS::Setsubi {
                    attach_to_previous = true;
                } else if token.pos2 == POS::Hijiritsu && token.inflection_form != POS::MeireiI {
                    attach_to_previous = true;
                }
            }
            POS::Keiyoushi => {
                pos = Some(PartOfSpeech::Adjective);
            }
            POS::Joshi => {
                pos = Some(PartOfSpeech::Postposition);
                if token.pos2 == POS::Setsuzokujoshi
                    && [TE, DE, BA].contains(&token.literal.as_str())
                {
                    attach_to_previous = true;
                }
            }
            POS::Rentaishi => {
                pos = Some(PartOfSpeech::Determiner);
            }
            POS::Setsuzokushi => {
                pos = Some(PartOfSpeech::Conjunction);
            }
            POS::Fukushi => {
                pos = Some(PartOfSpeech::Adverb);
            }
            POS::Kigou => {
                pos = Some(PartOfSpeech::Symbol);
            }
            POS::Firaa | POS::Kandoushi => {
                pos = Some(PartOfSpeech::Interjection);
            }
            POS::Sonota => pos = Some(PartOfSpeech::Other),
            _ => (),
        }

        // let's make sure we found *some* part of speech here
        if pos.is_none() {
            bail!(
                "Part of speech couldn't be recognized for token {}",
                token.literal
            );
        }
        let pos = pos.unwrap();

        if attach_to_previous && words.len() > 0 {
            let last = words.last_mut().unwrap();

            let token = token.clone();

            last.word.push_str(&token.literal);
            last.extra.reading.push_str(&token.reading);
            last.extra.transcription.push_str(&token.hatsuon);
            if also_attach_to_lemma {
                last.lemma.push_str(&token.lemma);
            }
            if update_pos {
                last.part_of_speech = pos
            }

            last.tokens.push(token);
        } else {
            let token = token.clone();
            let token2 = token.clone();

            let mut word = Word {
                word: token.literal,
                lemma: token.lemma,
                part_of_speech: pos,
                tokens: vec![token2],
                extra: WordExtra {
                    reading: token.reading,
                    transcription: token.hatsuon,
                    grammar,
                },
            };

            if eat_next {
                let Some(following) = iter.next() else {
                    bail!("eat_next was set despite there being no following token")
                };

                let following = following.clone();
                word.word.push_str(&following.literal);
                word.extra.reading.push_str(&following.reading);
                word.extra.transcription.push_str(&following.hatsuon);
                if eat_lemma {
                    word.lemma.push_str(&following.lemma)
                }
                word.tokens.push(following);
            }

            words.push(word);
        }
        previous = Some(token.clone());
    }

    Ok(words)
}
