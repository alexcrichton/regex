// The Computer Language Benchmarks Game
// http://benchmarksgame.alioth.debian.org/
//
// contributed by the Rust Project Developers
// contributed by TeXitoi
// contributed by BurntSushi

// This technically solves the problem posed in the `regex-dna` benchmark, but
// it cheats by combining all of the replacements into a single regex and
// replacing them with a single linear scan. i.e., it re-implements
// `replace_all`. As a result, this is around 25% faster. ---AG

extern crate regex;

use std::io::{self, Read};
use std::sync::Arc;
use std::thread;

macro_rules! regex { ($re:expr) => { ::regex::Regex::new($re).unwrap() } }

fn main() {
    let mut seq = String::with_capacity(50 * (1 << 20));
    io::stdin().read_to_string(&mut seq).unwrap();
    let ilen = seq.len();

    seq = regex!(">[^\n]*\n|\n").replace_all(&seq, "");
    let clen = seq.len();
    let seq_arc = Arc::new(seq.clone());

    let variants = vec![
        regex!("agggtaaa|tttaccct"),
        regex!("[cgt]gggtaaa|tttaccc[acg]"),
        regex!("a[act]ggtaaa|tttacc[agt]t"),
        regex!("ag[act]gtaaa|tttac[agt]ct"),
        regex!("agg[act]taaa|ttta[agt]cct"),
        regex!("aggg[acg]aaa|ttt[cgt]ccct"),
        regex!("agggt[cgt]aa|tt[acg]accct"),
        regex!("agggta[cgt]a|t[acg]taccct"),
        regex!("agggtaa[cgt]|[acg]ttaccct"),
    ];
    let mut count_promises = vec![];
    for i in 0..3 {
        let seq = seq_arc.clone();
        let res = variants[i * 3..i * 3 + 3].to_vec();
        count_promises.push(thread::spawn(move || {
            res.into_iter()
               .map(|re| (re.to_string(), re.find_iter(&seq).count()))
               .collect::<Vec<_>>()
        }));
    }

    let substs = vec![
        (b'B', "(c|g|t)"),
        (b'D', "(a|g|t)"),
        (b'H', "(a|c|t)"),
        (b'K', "(g|t)"),
        (b'M', "(a|c)"),
        (b'N', "(a|c|g|t)"),
        (b'R', "(a|g)"),
        (b'S', "(c|g)"),
        (b'V', "(a|c|g)"),
        (b'W', "(a|t)"),
        (b'Y', "(c|t)"),
    ]; // combined into one regex in `replace_all`
    let seq = replace_all(&seq, substs);

    for promise in count_promises {
        for (re, count) in promise.join().unwrap() {
            println!("{} {}", re, count);
        }
    }
    println!("\n{}\n{}\n{}", ilen, clen, seq.len());
}

fn replace_all(text: &str, substs: Vec<(u8, &str)>) -> String {
    let mut replacements = vec![""; 256];
    let mut alternates = vec![];
    for (re, replacement) in substs {
        replacements[re as usize] = replacement;
        alternates.push((re as char).to_string());
    }

    let re = regex!(&alternates.connect("|"));
    let mut new = String::with_capacity(text.len());
    let mut last_match = 0;
    for (s, e) in re.find_iter(text) {
        new.push_str(&text[last_match..s]);
        new.push_str(replacements[text.as_bytes()[s] as usize]);
        last_match = e;
    }
    new.push_str(&text[last_match..]);
    new
}
