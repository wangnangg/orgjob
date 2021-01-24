extern crate clap;

use clap::{App, Arg};
use std::fs::File;
use std::io::{BufReader, BufWriter, Result, Write};
use std::process::exit;
use std::vec::Vec;

mod code_doc;
mod org_parser;
#[cfg(test)]
mod tests;

use code_doc::*;
use org_parser::*;
use std::process::{Command, ExitStatus, Stdio};

fn run_code(interpreter: &str, code: &str) -> Result<ExitStatus> {
    let mut shell = Command::new(interpreter).stdin(Stdio::piped()).spawn()?;

    {
        let mut shell_in = shell.stdin.take().unwrap();
        let mut writer = BufWriter::new(&mut shell_in);
        writer.write_all(code.as_bytes())?;
        writer.flush()?;
    }

    return shell.wait();
}

fn main() -> Result<()> {
    let matches = App::new("Run code in org doc by hierarchy")
        .version("1.0")
        .arg(
            Arg::with_name("org_file")
                .short("f")
                .long("org-file")
                .value_name("ORG_FILE")
                .help("org file")
                .default_value("jobs.org")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("action")
                .help("action to do with the job")
                .index(1)
                .required(true)
                .possible_values(&["run", "list", "show"]),
        )
        .arg(
            Arg::with_name("job")
                .help("job to run")
                .required(false)
                .index(2),
        )
        .arg(
            Arg::with_name("lang")
                .help("specify script language in case of ambiguity")
                .required(false)
                .index(3),
        )
        .get_matches();

    let action = matches.value_of("action").unwrap();
    let org_file = matches.value_of("org_file").unwrap();
    let mut reader = BufReader::new(match File::open(org_file) {
        Ok(f) => f,
        Err(_) => {
            println!("failed to open: {}", org_file);
            exit(1);
        }
    });
    let doc = match parse_org_doc(&mut reader, "doc".to_string(), "bash") {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            exit(1);
        }
    };

    match matches.value_of("job") {
        Some(job) => {
            let query: Vec<&str> = job.split("::").collect();
            let nodes = doc.lookup_nodes(DOC_NODE_ROOT_ID, &query);
            match nodes.len() {
                0 => {
                    println!("no matches for: {:?}", query);
                    exit(1);
                }
                1 => {
                    let n = nodes[0];
                    let node = doc.get_node(n);
                    let code = doc.get_runnable_code(n);
                    let mut selected_code = None;
                    match code.len() {
                        0 => {
                            println!("no code avaiable in {} for {:?}", node.name(), query);
                            exit(1);
                        }
                        1 => {
                            selected_code = Some(&code[0]);
                        }
                        _ => {
                            if let Some(lang) = matches.value_of("lang") {
                                for c in code.iter() {
                                    if c.interpreter == lang {
                                        selected_code = Some(c);
                                        break;
                                    }
                                }
                                if let None = selected_code {
                                    println!("no match for: {:?} with lang: {}", query, lang);
                                    exit(1);
                                }
                            } else {
                                println!("mutliple languages in the matched block; use -l/--lang");
                                exit(1);
                            }
                        }
                    };
                    let c = selected_code.unwrap();
                    if action == "run" {
                        match run_code(&c.interpreter, &c.code.join("\n"))?.code() {
                            Some(code) => exit(code),
                            None => {
                                println!("subprocess killed");
                                exit(1);
                            }
                        }
                    } else {
                        //show
                        println!("#!/usr/bin/env {}", c.interpreter);
                        println!("{}", c.code.join("\n"));
                        exit(0);
                    }
                }
                _ => {
                    println!("multiple matches for: {:?}", query);
                    for n in nodes {
                        println!("{}", doc.get_node(n).name());
                    }
                    exit(1);
                }
            }
        }
        None => {
            if action != "list" {
                println!("job name not provided");
                exit(1);
            }

            for node in (DOC_NODE_ROOT_ID + 1)..=doc.len() {
                for _ in 0..doc.get_node(node).level() {
                    print!("*");
                }
                println!(" {}", doc.get_node(node).name());
            }
        }
    };

    return Ok(());
}
