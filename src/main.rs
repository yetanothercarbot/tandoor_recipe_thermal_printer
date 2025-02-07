use escpos::printer::Printer;
use escpos::utils::*;
use escpos::{driver::*, errors::Result};
use std::path::Path;
use std::fs;
use std::process::exit;
// use std::env;
use json::*;
use json::JsonValue::Array;

fn retrieve_recipe() -> JsonValue {
    let file_path = Path::new("./test.json");
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    // println!("{contents}");
    json::parse(&contents).unwrap()
}

fn main() -> Result<()> {
    let recipe = retrieve_recipe();

    println!("Recipe: {}", recipe["name"]);

    let path = Path::new("/dev/usb/lp0");
    let driver = FileDriver::open(&path).unwrap();
    let mut printer = Printer::new(driver.clone(), Protocol::default(), None);
    printer.init().unwrap();
    printer.bold(true)?
        .size(2, 2)?
        .writeln(&format!("{}", recipe["name"]))?
        .reset_size()?
        .bold(false)?;

    let steps;
    match &recipe["steps"] {
        Array(val) => {steps = val},
        _ => {exit(1)}
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
                _ => {exit(1)}
            }
            for ingredient in ingredients {
                println!("- {}{} {}", 
                    ingredient["conversions"][0]["amount"], 
                    ingredient["conversions"][0]["unit"], 
                    ingredient["conversions"][0]["food"]);
                printer.writeln(&format!("- {}{} {}", 
                    ingredient["conversions"][0]["amount"], 
                    ingredient["conversions"][0]["unit"], 
                    ingredient["conversions"][0]["food"]))?;
            }
        }
        println!("{}", step["instruction"]);
        printer.writeln(&format!("{}", step["instruction"]))?;
        printer.feed()?;
    }

    env_logger::init();
    
    printer.feed()?
        .print_cut()?;

    Ok(())
}