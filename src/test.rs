mod lib;

fn main() {
    let excerpt = r#"イスラエル軍は27日夜、イスラム組織ハマスが実効支配するガザ地区にこれまでにない激しい空爆を行うとともに、地上での軍事行動を拡大していると発表しました。

    一方、ガザ地区では空爆による死者が増え続け、地区の保健当局はこれまでに3000人を超える子どもが死亡したと発表しました。
    
    最新の動きを随時更新でお伝えしています"#;

    //lindera::tokenize(&excerpt).unwrap();
    let raw_tokens = lib::vibrato_tokenize(&excerpt).unwrap();

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
