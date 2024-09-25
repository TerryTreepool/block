

fn main() {

    let mnemonic = "bar cinnamon grow  \rhungry   lens\n danger treat artist hello seminar document gasp";

    let words: Vec<&str> = mnemonic.split(' ').into_iter().map(| v | v.trim()).collect();
    if words.len() != 12 {
        panic!();
    };

    let mnemonic = words.join(" ");
    println!("{mnemonic}");


}
