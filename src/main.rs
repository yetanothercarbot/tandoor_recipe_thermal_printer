use escpos::printer::Printer;
use escpos::utils::*;
use escpos::{driver::*, errors::Result};
use std::path::Path;
use std::process::exit;
use serde_json::*;
use serde_json::Value::Array;
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "recipe_printer")]
struct Arguments {
    /// Base link to Tandoor instance, with protocol - e.g. "https://recipes.example.com"
    instance: String,
    
    /// Recipe ID
    id: u16,

    /// Username to authenticate with
    #[arg(short, long)]
    username: Option<String>,

    /// Password to authenticate with
    #[arg(short, long)]
    password: Option<String>,

    /// Token to authenticate with
    #[arg(short, long)]
    token: Option<String>,

    /// Printer path
    #[arg(long, default_value_t=String::from("/dev/usb/lp0"))]
    printer_path: String,

    /// Print QR code link to recipe
    #[arg(long)]
    qr: bool,
}

fn auth(args: &Arguments) -> String {
    if args.token.is_some() {
        return args.token.clone().unwrap();
    }

    if args.username.is_some() && args.password.is_some() {
        // Retrieve token
        let mut auth_deets = HashMap::new();
        auth_deets.insert("username", args.username.clone().unwrap());
        auth_deets.insert("password", args.password.clone().unwrap());

        println!("Signing in with {:?} / {:?}", auth_deets.get("username"), auth_deets.get("password"));

        println!("Requesting {}/api-token/auth/", args.instance);

        let auth_client = reqwest::blocking::Client::new();

        let resp = auth_client.post(format!("{}/api-token-auth/", args.instance))
            .json(&auth_deets)
            .send();
        
        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    let response: String = r.text().unwrap();
                    let authorisation: Value = serde_json::from_str(&response).expect("Malformed authorisation");
                    println!("Received token {}", &authorisation["token"].as_str().unwrap());
                    return authorisation["token"].as_str().unwrap().to_string();
                } else {
                    println!("Unable to authenticate!");
                    exit(2);
                }
                // println!("{}", r.json().expect("No body")["token"]);
            },
            Err(e) => {
                println!("Network/Auth Error: {}", e);
                exit(2);
            }
        }
        
    }

    if args.username.is_none() || args.password.is_none() {
        println!("Please provide username and password!");
        exit(1);
    }

    println!("Please provide at least one authentication method!");
    exit(1);
}

fn retrieve_recipe(args: &Arguments, tok: String) -> Value {
    let recipe_client = reqwest::blocking::Client::new();

    let resp = recipe_client.post(format!("{}/api/recipe/{}", args.instance, args.id))
        .header("Authorization", format!("Bearer {tok}"))
        .send();

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                let response = r.text().unwrap();
                return serde_json::from_str(&response).expect("Malformed recipe");
            } else {
                println!("Got HTTP failure - is Tandoor running?");
                exit(2);
            }
        }
        Err(e) => {
            println!("Network/Auth Error: {}", e);
            exit(2);
        }
    }
}

fn main() -> Result<()> {
    let args = Arguments::parse();
    let tok = auth(&args);

    let recipe = retrieve_recipe(&args, tok);

    println!("Recipe: {}", recipe["name"]);

    let path = Path::new(&args.printer_path);
    let driver = FileDriver::open(&path).unwrap();
    let mut printer = Printer::new(driver.clone(), Protocol::default(), None);
    printer.init().unwrap();
    printer.bold(true)?
        .size(2, 2)?
        .writeln(&format!("{}", recipe["name"].as_str().unwrap()))?
        .reset_size()?
        .bold(false)?;

    let steps;
    match &recipe["steps"] {
        Array(val) => {steps = val},
        _ => {exit(3)}
    }

    let mut step_no = 0;
    for step in steps {
        step_no += 1;
        println!("--> Step {}", step_no);

        printer.bold(true)?
            .writeln(&format!("Step {}", step_no))?
            .bold(false)?;

        if step["ingredients"].is_null() == false {
            let ingredients;
            match &step["ingredients"] {
                Array(val) => {ingredients = val},
                _ => {exit(2)}
            }
            for ingredient in ingredients {
                let mut ingredient_str = String::from("- ");
                let mut amount: f64 = 0.0;
                
                if ingredient["amount"].as_number().is_some() {
                    let amnt = ingredient["amount"].as_number().unwrap();
                    if amnt.is_f64() && amnt.as_f64().unwrap() > 0.0 {
                        amount = amnt.as_f64().unwrap();
                        ingredient_str.push_str(&format!("{} ", &amnt));
                    }
                }

                if ingredient["unit"]["plural_name"].as_str().is_some() 
                    && (amount - 1.0).abs() > f64::EPSILON {
                    let pl_unit = ingredient["unit"]["plural_name"].as_str().unwrap();
                    ingredient_str.push_str(&format!("{} ", &pl_unit));
                }

                if ingredient["unit"]["name"].as_str().is_some() && (
                    amount - 1.0 < f64::EPSILON
                    || ingredient["unit"]["plural_name"].as_str().is_none()
                    ) {
                    let unit = ingredient["unit"]["name"].as_str().unwrap();
                    ingredient_str.push_str(&format!("{} ", &unit));
                }

                if ingredient["food"]["name"].as_str().is_some() {
                    let food = ingredient["food"]["name"].as_str().unwrap();
                    ingredient_str.push_str(&format!("{} ", &food));

                    if ingredient["note"].as_str().is_some() {
                        let note = ingredient["note"].as_str().unwrap();
                        ingredient_str.push_str(&format!(" ({})", &note));
                    }

                    println!("{}", ingredient_str);
                    printer.writeln(&ingredient_str)?;
                }
            }
        }
        println!("{}", step["instruction"].as_str().unwrap());
        printer.writeln(&format!("{}", step["instruction"].as_str().unwrap()))?;
        printer.feed()?;
    }

    if args.qr {
        printer.qrcode(&format!("{}/view/recipe/{}", args.instance, args.id))?;
    }
    
    printer.feed()?
        .print_cut()?;

    Ok(())
}