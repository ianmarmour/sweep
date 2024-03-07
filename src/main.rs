use imap::fetch_inbox_top;
use llama_cpp::grammar::LlamaGrammar;
use llama_cpp::standard_sampler::{SamplerStage, StandardSampler};
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use std::path::PathBuf;
use std::str;
use std::str::FromStr;
mod imap;

extern crate regex;

use regex::Regex;

fn remove_html_tags(input: &str) -> String {
    let html_tag_regex = Regex::new(r"(?i)<[^>]*>").expect("Invalid regex pattern");
    html_tag_regex.replace_all(input, "").to_string()
}

fn remove_links(text: &str) -> String {
    let url_regex = Regex::new(
        r"http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+",
    )
    .expect("Failed to create regex");

    url_regex.replace_all(text, "").to_string()
}

fn remove_whitespace(input: &str) -> String {
    let whitespace_regex = Regex::new(r"\s{2,}").expect("Invalid regex pattern");
    whitespace_regex.replace_all(input, "").to_string()
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

fn truncate_in_place(s: &mut String, max_chars: usize) {
    let bytes = truncate(&s, max_chars).len();
    s.truncate(bytes);
}

fn main() {
    /*
        // a builder for `FmtSubscriber`.
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::TRACE)
            // completes the builder.
            .finish();

        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */
    let model_path = &PathBuf::from("/Users/ian/Models/mixtral-8x7b-instruct-v0.1.Q4_K_M.gguf");

    let model = LlamaModel::load_from_file(model_path, LlamaParams::default())
        .expect("Could not load model");

    for n in 1..10 {
        let mut session_params: SessionParams = SessionParams::default();
        session_params.n_ctx = 32000_u32;
        session_params.n_batch = 512;

        let message = fetch_inbox_top().unwrap().unwrap();
        let message_without_links = remove_links(message.as_str());
        let message_without_html = remove_html_tags(message_without_links.as_str());
        let mut message_without_whitespace =
            remove_whitespace(message_without_html.as_str()).to_string();
        truncate_in_place(&mut message_without_whitespace, 1000);
        //println!("{}", String::from(message_without_whitespace.clone()));

        let mut ctx = model
            .create_session(session_params)
            .expect("Failed to create session");

        let mut prompt = "".to_owned();
        let prompt_start = "[INST]";
        let system_entry = " You are a helpful and honest assistant. Your job is to read e-mails and determine if they're important enough to read. I do not care about advertisements, promotional emails or anything that can be answered without a timeframe required. ";
        let prompt_end = "[/INST]";

        prompt.push_str(prompt_start);
        prompt.push_str(system_entry);
        prompt.push_str(&message_without_whitespace.clone());
        prompt.push_str(prompt_end);

        //println!("{}", String::from(prompt.clone()));
        ctx.advance_context(prompt).unwrap();

        let gbnf_bytes = include_bytes!("json.gbnf");
        let gbnf: &str = str::from_utf8(gbnf_bytes).unwrap();
        let grammar = LlamaGrammar::from_str(gbnf).unwrap();

        let stages = vec![
            SamplerStage::RepetitionPenalty {
                repetition_penalty: 1.1,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                last_n: 64,
            },
            SamplerStage::TopK(40),
            SamplerStage::TopP(0.95),
            SamplerStage::MinP(0.05),
            SamplerStage::Temperature(0.8),
        ];

        let sampler = StandardSampler::new_softmax(stages, 1, Some(grammar));

        let completions = ctx.start_completing_with(sampler, 1024).into_string();

        println!("{}", String::from(completions.clone()));
    }
}
