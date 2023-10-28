use std::fs::File;

use anyhow::Result;
use lib::RawToken;
use vibrato::{Dictionary, Tokenizer};

mod lib;

pub fn vibrato_tokenize(sentence: &str) -> Result<Vec<RawToken>> {
    let reader = zstd::Decoder::new(File::open("system.dic.zst")?)?;
    let mut dict = Dictionary::read(reader)?;

    let tokenizer = Tokenizer::new(dict)
        .ignore_space(true)?
        .max_grouping_len(24);
    let mut worker = tokenizer.new_worker();

    worker.reset_sentence(&sentence);
    worker.tokenize();

    let tokens: Vec<RawToken> = worker.token_iter().map(|t| t.into()).collect();

    Ok(tokens)
}

fn main() {
    let excerpt = r#"
    イスラエル軍は27日夜、イスラム組織ハマスが実効支配するガザ地区にこれまでにない激しい空爆を行うとともに、地上での軍事行動を拡大していると発表しました。
    一方、ガザ地区では空爆による死者が増え続け、地区の保健当局はこれまでに3000人を超える子どもが死亡したと発表しました。
    最新の動きを随時更新でお伝えしています
    "#;

    let raw_tokens = vibrato_tokenize(&excerpt).unwrap();

    let debug_str = raw_tokens
        .iter()
        .map(|t| format!("{}\t [{}]", t.surface, t.feature))
        .collect::<Vec<String>>()
        .join("\n");

    println!("Raw Tokens");
    println!("{}", debug_str);

    let prepared_tokens = lib::prepare_tokens(raw_tokens).unwrap();

    println!("Prepared Tokens");
    println!("{:#?}", prepared_tokens);

    let words = lib::parse_into_words(prepared_tokens).unwrap();

    println!("Words");
    println!("{:#?}", words);

    println!("Final sentence splitting");
    let sentence = words
        .into_iter()
        .map(|w| w.word)
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", sentence);
}
