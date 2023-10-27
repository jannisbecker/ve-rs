mod tokenizer;

fn main() {
    let excerpt = r#"
    
    ハハ、お前が俺の相手だって？笑わせるつもりか？お前、いくつ？７歳？みたいだな。
    俺に汗をかく暗いのことが出来るかもしれないが、打撃を与えることは絶対できないな。

    "#;

    //lindera::tokenize(&excerpt).unwrap();
    tokenizer::vibrato_tokenize(&excerpt).unwrap();
}
