use anyhow::{bail, Result};
use std::{fs::File, any};

use vibrato::{Dictionary, Tokenizer};

struct RawToken {
    surface: String,
    feature: String,
}

impl From<vibrato::token::Token<'_, '_>> for RawToken {
    fn from(value: vibrato::token::Token) -> Self {
        Self {
            surface: value.surface().into(),
            feature: value.feature().into(),
        }
    }
}

struct PreparedToken {
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

#[derive(PartialEq)]
enum POS {
    MEISHI,
    KOYUUMEISHI,
    DAIMEISHI,
    JODOUSHI,
    KAZU,
    JOSHI,
    SETTOUSHI,
    DOUSHI,
    KIGOU,
    FIRAA,
    SONOTA,
    KANDOUSHI,
    RENTAISHI,
    SETSUZOKUSHI,
    FUKUSHI,
    SETSUZOKUJOSHI,
    KEIYOUSHI,
    HIJIRITSU,
    FUKUSHIKANOU,
    SAHENSETSUZOKU,
    KEIYOUDOUSHIGOKAN,
    NAIKEIYOUSHIGOKAN,
    JODOUSHIGOKAN,
    FUKUSHIKA,
    TAIGENSETSUZOKU,
    RENTAIKA,
    TOKUSHU,
    SETSUBI,
    SETSUZOKUSHITEKI,
    DOUSHIHIJIRITSUTEKI,
    SAHEN_SURU,
    TOKUSHU_TA,
    TOKUSHU_NAI,
    TOKUSHU_TAI,
    TOKUSHU_DESU,
    TOKUSHU_DA,
    TOKUSHU_MASU,
    TOKUSHU_NU,
    FUHENKAGATA,
    JINMEI,
    MEIREI_I,
    KAKARIJOSHI,
    Unknown,
}

impl From<&str> for POS {
    fn from(value: &str) -> Self {
        match value {
            "名詞" => Self::MEISHI,
            "固有名詞" => Self::KOYUUMEISHI,
            "代名詞" => Self::DAIMEISHI,
            "助動詞" => Self::JODOUSHI,
            "数" => Self::KAZU,
            "助詞" => Self::JOSHI,
            "接頭詞" => Self::SETTOUSHI,
            "動詞" => Self::DOUSHI,
            "記号" => Self::KIGOU,
            "フィラー" => Self::FIRAA,
            "その他" => Self::SONOTA,
            "感動詞" => Self::KANDOUSHI,
            "連体詞" => Self::RENTAISHI,
            "接続詞" => Self::SETSUZOKUSHI,
            "副詞" => Self::FUKUSHI,
            "接続助詞" => Self::SETSUZOKUJOSHI,
            "形容詞" => Self::KEIYOUSHI,
            "非自立" => Self::HIJIRITSU,
            "副詞可能" => Self::FUKUSHIKANOU,
            "サ変接続" => Self::SAHENSETSUZOKU,
            "形容動詞語幹" => Self::KEIYOUDOUSHIGOKAN,
            "ナイ形容詞語幹" => Self::NAIKEIYOUSHIGOKAN,
            "助動詞語幹" => Self::JODOUSHIGOKAN,
            "副詞化" => Self::FUKUSHIKA,
            "体言接続" => Self::TAIGENSETSUZOKU,
            "連体化" => Self::RENTAIKA,
            "特殊" => Self::TOKUSHU,
            "接尾" => Self::SETSUBI,
            "接続詞的" => Self::SETSUZOKUSHITEKI,
            "動詞非自立的" => Self::DOUSHIHIJIRITSUTEKI,
            "サ変・スル" => Self::SAHEN_SURU,
            "特殊・タ" => Self::TOKUSHU_TA,
            "特殊・ナイ" => Self::TOKUSHU_NAI,
            "特殊・タイ" => Self::TOKUSHU_TAI,
            "特殊・デス" => Self::TOKUSHU_DESU,
            "特殊・ダ" => Self::TOKUSHU_DA,
            "特殊・マス" => Self::TOKUSHU_MASU,
            "特殊・ヌ" => Self::TOKUSHU_NU,
            "不変化型" => Self::FUHENKAGATA,
            "人名" => Self::JINMEI,
            "命令ｉ" => Self::MEIREI_I,
            "係助詞" => Self::KAKARIJOSHI,
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

pub struct Word {
    word: String,
    lemma: String, // dictionary form
    part_of_speech: PartOfSpeech,
    tokens: Vec<PreparedToken>,
    extra: WordExtra
}

pub struct WordExtra {
    reading: String,
    transcription: String,
    grammar: Option<Grammar>
}

#[derive(PartialEq)]
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

pub enum Grammar {
    Auxillary,
    Nominal
}

fn vibrato_tokenize(sentence: &str) -> Result<Vec<RawToken>> {
    let reader = zstd::Decoder::new(File::open("system.dic.zst")?)?;
    let mut dict = Dictionary::read(reader)?;

    let tokenizer = Tokenizer::new(dict).ignore_space(true)?.max_grouping_len(0);
    let mut worker = tokenizer.new_worker();

    worker.reset_sentence(&sentence);
    worker.tokenize();

    let tokens: Vec<RawToken> = worker.token_iter().map(|t| t.into()).collect();

    Ok(tokens)
}

pub fn prepare_tokens(raw_tokens: Vec<RawToken>) -> Result<Vec<PreparedToken>> {
    raw_tokens.into_iter().map(|raw_token| {
        let features: Vec<&str> = raw_token.feature.split(',').collect();
        let [pos, pos2, pos3, pos4, inflection_type, inflection_form, lemma, reading, hatsuon, ..] = features[..9] else {
            bail!("Couldn't read all features from token. Make sure you're using an IPADIC dictionary")
        };

        let parsed_pos = [
            POS::from(pos), 
            POS::from(pos2),
            POS::from(pos3),
            POS::from(pos4),
            POS::from(inflection_type),
            POS::from(inflection_form)
            ];

        if parsed_pos.into_iter().any(|p| p == POS::Unknown) {
            bail!("Some of the tokens POS or inflection info couldn't be parsed");
        }

        Ok(PreparedToken {
            literal: raw_token.surface,
            pos: parsed_pos[0],
            pos2: parsed_pos[1],
            pos3: parsed_pos[2],
            pos4: parsed_pos[3],
            inflection_type: parsed_pos[4],
            inflection_form: parsed_pos[5],
            lemma: lemma.into(),
            reading:  reading.into(),
            hatsuon: hatsuon.into(),
        })

    }).collect()
}

pub fn parse_into_words(tokens: Vec<PreparedToken>) -> Result<Vec<Word>> {
    let mut words: Vec<Word> = Vec::new();
    let mut iter = tokens.into_iter().peekable();
    let mut previous: Option<PreparedToken>;

    for token in iter {
        let mut pos: Option<PartOfSpeech> = None;
        let mut grammar: Option<Grammar> = None;
        let mut eat_next = false;
        let mut eat_lemma = false;
        let mut attach_to_previous = false;
        let mut also_attach_to_lemma = false;
        let mut update_pos = false;

        match token.pos {
            POS::MEISHI => {
                pos = Some(PartOfSpeech::Noun);
                match token.pos2 {
                    POS::KOYUUMEISHI => {
                        pos = Some(PartOfSpeech::ProperNoun);
                    },
                    POS::DAIMEISHI => {
                        pos = Some(PartOfSpeech::Pronoun);
                    },
                    POS::FUKUSHIKANOU |POS::SAHENSETSUZOKU | POS::KEIYOUDOUSHIGOKAN | POS::NAIKEIYOUSHIGOKAN => {
                        if let Some(following) = iter.peek() {
                            if following.inflection_type == POS::SAHEN_SURU {
                                pos = Some(PartOfSpeech::Verb);
                                eat_next = true;
                            } else if following.inflection_type == POS::TOKUSHU_DA {
                                pos = Some(PartOfSpeech::Adjective);
                                if following.inflection_form == POS::TAIGENSETSUZOKU {
                                    eat_next = true;
                                    eat_lemma = false;
                                }
                            } else if following.inflection_type == POS::TOKUSHU_NAI {
                                pos = Some(PartOfSpeech::Adjective);
                                eat_next = true;
                            } else if following.pos == POS::JOSHI && following.literal == NI {
                                pos = Some(PartOfSpeech::Adverb);
                                eat_next = false;
                            }
                        }
                    },
                    POS::HIJIRITSU | POS::TOKUSHU => {
                        if let Some(following) = iter.peek() {
                            match token.pos3 {
                                POS::FUKUSHIKANOU => {
                                    if following.pos == POS::JOSHI && following.literal == NI {
                                    pos = Some(PartOfSpeech::Adverb);
                                    eat_next = true;
                                    }
                                },
                                POS::JODOUSHIGOKAN => {
                                    if following.inflection_type == POS::TOKUSHU_DA {
                                        pos = Some(PartOfSpeech::Verb);
                                        grammar = Some(Grammar::Auxillary);

                                        if following.inflection_form == POS::TAIGENSETSUZOKU {
                                            eat_next = true;
                                        }
                                    } else if following.pos == POS::JOSHI && following.pos2 == POS::FUKUSHIKA {
                                        pos = Some(PartOfSpeech::Adverb);
                                        eat_next = true;
                                    }
                                },
                                POS::KEIYOUDOUSHIGOKAN => {
                                    pos = Some(PartOfSpeech::Adjective);
                                    if (following.inflection_type == POS::TOKUSHU_DA && following.inflection_form == POS::TAIGENSETSUZOKU) 
                                        || following.pos2 == POS::RENTAIKA {
                                            eat_next = true;
                                        }
                                },
                                _ => ()
                            }
                        }
                    }
                    POS::KAZU => {
                        pos = Some(PartOfSpeech::Number);
                        if words.len() > 0 && words.last().is_some_and(|w| w.part_of_speech == PartOfSpeech::Number) {
                            attach_to_previous = true;
                            also_attach_to_lemma = true;
                        }
                    },
                    POS::SETSUBI => {
                        if token.pos3 == POS::JINMEI {
                            pos = Some(PartOfSpeech::Suffix);
                        } else {
                            if token.pos3 == POS::TOKUSHU && token.lemma == SA {
                                update_pos = true;
                                pos = Some(PartOfSpeech::Noun);
                            } else {
                                also_attach_to_lemma = true;
                            }
                            attach_to_previous = true;
                        }
                    },
                    POS::SETSUZOKUSHITEKI => {
                        pos = Some(PartOfSpeech::Conjunction);
                    },
                    POS::DOUSHIHIJIRITSUTEKI => {
                        pos = Some(PartOfSpeech::Verb);
                        grammar = Some(Grammar::Nominal)
                    },
                    _ => ()
                }    
            },
            POS::SETTOUSHI => {
                pos = Some(PartOfSpeech::Prefix);
            },
            POS::JODOUSHI => {
                pos = Some(PartOfSpeech::Postposition);

                if (previous.is_none() || (previous.is_some_and(|p| p.pos2 != POS::KAKARIJOSHI))) &&
                [POS::TOKUSHU_TA, POS::TOKUSHU_NAI, POS::TOKUSHU_TAI, POS::TOKUSHU_MASU, POS::TOKUSHU_NU].contains(&token.inflection_type) {
                    attach_to_previous = true;
                } else if token.inflection_type == POS::FUHENKAGATA && token.lemma == NN {
                    attach_to_previous = true;
                } else if [POS::TOKUSHU_DA, POS::TOKUSHU_DESU].contains(&token.inflection_type) && token.literal != NA {
                    pos = Some(PartOfSpeech::Verb)
                }
            },
            POS::DOUSHI => {
                pos = Some(PartOfSpeech::Verb);
                if token.pos2 == POS::SETSUBI {
                    attach_to_previous = true;
                } else if token.pos2 == POS::HIJIRITSU && token.inflection_form != POS::MEIREI_I {
                    attach_to_previous = true;
                }
            },
            POS::KEIYOUSHI => {
                pos = Some(PartOfSpeech::Adjective);
            },
            POS::JOSHI => {
                pos = Some(PartOfSpeech::Postposition);
                if token.pos2 == POS::SETSUZOKUJOSHI && [TE, DE, BA].contains(&token.literal.as_str()) {
                    attach_to_previous = true;
                }
            },
            POS::RENTAISHI => {
                pos = Some(PartOfSpeech::Determiner);
            },
            POS::SETSUZOKUSHI => {
                pos = Some(PartOfSpeech::Conjunction);
            },
            POS::FUKUSHI => {
                pos = Some(PartOfSpeech::Adverb);
            },
            POS::KIGOU => {
                pos = Some(PartOfSpeech::Symbol);
            },
            POS::FIRAA | POS::KANDOUSHI => {
                pos = Some(PartOfSpeech::Interjection);
            },
            POS::SONOTA => {
                pos = Some(PartOfSpeech::Other)
            },
            _ => ()
        }
    
        // let's make sure we found *some* part of speech here
        if pos.is_none() {
            bail!("Part of speech couldn't be recognized for token {}", token.literal);
        }
        let pos = pos.unwrap();

        if attach_to_previous && words.len() > 0 {
            let mut last = words.last().unwrap();

            last.tokens.push(token);
            last.word.push_str(&token.literal);
            last.extra.reading.push_str(&token.reading);
            last.extra.transcription.push_str(&token.hatsuon);
            if also_attach_to_lemma {
                last.lemma.push_str(&token.lemma);
            }
            if update_pos {
                last.part_of_speech = pos
            }
        } else {

            let mut word = Word {
                word: token.literal,
                lemma: token.lemma,
                part_of_speech: pos,
                tokens: vec![token],
                extra: WordExtra { 
                    reading: token.reading, 
                    transcription: token.hatsuon, 
                    grammar 
                }
            };

            if eat_next {
                let Some(following) = iter.next() else {
                    bail!("eat_next was set despite there being no following token")
                };

                word.tokens.push(following);
                word.word.push_str(&following.literal);
                word.extra.reading.push_str(&following.reading);
                word.extra.transcription.push_str(&following.hatsuon);
                if eat_lemma {
                    word.lemma.push_str(&following.lemma)
                }
            }

            words.push(word);
        }
        previous = Some(token);
    }

    Ok(words)
}
