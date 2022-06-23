use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use crate::params::MacroParameters;

pub fn dump_maybe(args: &TokenStream, input: &TokenStream) {
    println!("--vvv------------------------------------------");
    println!("maybe:");
    println!("args:");
    println!("{}", args);
    println!("input:");
    println!("{}", input);
    println!("--^^^------------------------------------------");
    println!("");
}

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

pub fn dump_params(name: &str, params: &MacroParameters) {
    println!("--vvv------------------------------------------");
    println!("{}:", name);
    println!("{:#?}", params);
    println!("--^^^------------------------------------------");
    println!("");
}

