use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub fn dump_tokens(name: &str, ts: &TokenStream) {
    println!("--vvv------------------------------------------");
    println!("{}:", name);
    println!("{}", ts);
    println!("--^^^------------------------------------------");
    println!("");
}

pub fn dump_tokens2(name: &str, ts: &TokenStream2) {
    println!("--vvv------------------------------------------");
    println!("{}:", name);
    println!("{}", ts);
    println!("--^^^------------------------------------------");
    println!("");
}