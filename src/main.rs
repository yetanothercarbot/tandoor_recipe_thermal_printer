mod tandoor_api;

use std::io::{stdin, Read};use indicatif::ProgressBar;
use escpos::printer::Printer;
use escpos::utils::*;
use escpos::{driver::*, errors::Result};
use std::path::Path;
use std::process::exit;
use clap::Parser;
use unidecode::unidecode;
use crate::tandoor_api::TandoorRecipe;

#[derive(clap::ValueEnum, Clone, PartialEq, Eq, Default)]
enum IngredientDisplay {
    Both,
    Summary,
    #[default]
    Step,
    None
}

#[derive(clap::ValueEnum, Clone, PartialEq, Eq, Default)]
enum CutMode {
    None,
    Pause,
    Partial,
    #[default]
    Full
}

#[derive(Parser)]
#[command(version, name = "recipe_printer")]
struct Arguments {
    /// Base link to Tandoor instance, with protocol - e.g. "https://recipes.example.com"
    instance: String,

    /// Tandoor token to authenticate with
    token: String,

    /// Recipe ID
    id: Vec<u32>,

    /// Dry run (retrieve recipe but do not print)
    #[arg(long)]
    dry_run: bool,

    /// Print more debugging information
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Printer path
    #[arg(short, long, default_value_t=String::from("/dev/usb/lp0"))]
    printer_path: String,

    /// Customise how ingredients are displayed - as an aggregated list, per-step, both or none
    #[arg(short, long, value_enum, default_value_t)]
    ingredient_display: IngredientDisplay,

    /// Add number of servings and cooking time (if relevant) to printout
    #[arg(short, long)]
    stats: bool,

    /// Print QR code link to recipe
    #[arg(long)]
    qr: bool,

    /// Select type of cut at end of recipe (no cut, pause for printers with no automatic cutter, partial cut or full cut)
    #[arg(long, value_enum, default_value_t)]
    cut_mode: CutMode,

    /// Width of paper, in characters. Used for word wrapping (use 0 to revert to the printer's built-in wrapping)
    #[arg(long, default_value_t=42)]
    columns: u32,

}

fn retrieve_recipe(instance: &str, token: &str, id: &u32) -> TandoorRecipe {
    let recipe_client = reqwest::blocking::Client::new();

    let resp = recipe_client.get(format!("{instance}/api/recipe/{id}/"))
        .header("Authorization", format!("Bearer {token}"))
        .send();

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                let response = r.text().unwrap();
                serde_json::from_str(&response).expect("Malformed recipe")
            } else {
                println!("Recipe {id}: got HTTP error ({}) - is Tandoor running and accessible?", r.status());
                exit(2);
            }
        }
        Err(e) => {
            println!("Network/Auth Error: {e}");
            exit(2);
        }
    }
}

fn format_text(input: &str, width: usize) -> String {
    if width == 0 {
        unidecode(input)
    } else {
        textwrap::fill(&unidecode(input), width)
    }
}

fn main() -> Result<()> {
    let args = Arguments::parse();
    let mut recipes: Vec<TandoorRecipe> = Vec::new();

    let bar_style = indicatif::ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap();

    let download_bar = ProgressBar::new(args.id.len() as u64);
    download_bar.set_style(bar_style.clone());
    download_bar.set_message("Retrieving recipes");
    for id in &args.id {
        recipes.push(retrieve_recipe(&args.instance, &args.token, id));
        download_bar.inc(1);
    }
    download_bar.finish_with_message("Recipes retrieved");

    if args.verbose >= 1 {
        println!("Recipes retrieved:");
        for recipe in &recipes {
            println!("Recipe {}: {} steps", recipe.name, recipe.steps.len());
        }
        println!("---");
    }

    if !args.dry_run {
        let path = Path::new(&args.printer_path);
        let driver = FileDriver::open(path)?;
        let mut printer = Printer::new(driver, Protocol::default(), None);
        printer.init()?;

        let print_bar = ProgressBar::new(args.id.len() as u64);
        print_bar.set_style(bar_style.clone());

        for recipe in &recipes {
            if args.verbose >= 1 {
                println!("Printing {}", recipe.name);
            }
            print_bar.set_message("Printing...");
            print_bar.inc(1);

            printer.justify(JustifyMode::LEFT)?;

            printer.bold(true)?
                .size(2, 2)?
                .writeln(&format_text(&recipe.name, (args.columns/2) as usize))?
                .reset_size()?
                .bold(false)?
                .feed()?;

            if args.stats {
                printer.writeln(&recipe.get_servings())?
                    .writeln(&recipe.get_duration())?;
            }


            if args.verbose >= 2 {
                println!("{}", format_text(&recipe.name, (args.columns/2) as usize));
            }

            if args.ingredient_display == IngredientDisplay::Both || args.ingredient_display == IngredientDisplay::Summary {
                printer.bold(true)?
                    .underline(UnderlineMode::Single)?
                    .writeln("Ingredients")?
                    .bold(false)?
                    .underline(UnderlineMode::None)?;

                for ingredient in recipe.get_all_ingredients() {
                    if ingredient.include() {
                        printer.writeln(&format_text(&ingredient.pretty_print(), args.columns as usize))?;
                    }
                }
                printer.feed()?;
            }

            let mut step_no: u32 = 0;
            for step in &recipe.steps {
                step_no += 1;

                let mut heading = format!("Step {step_no}");
                if !step.name.is_empty() {
                    heading.push_str(&format!(": {}", step.name));
                }

                printer.bold(true)?
                    .underline(UnderlineMode::Single)?
                    .writeln(&heading)?
                    .bold(false)?
                    .underline(UnderlineMode::None)?;

                if args.verbose >= 2 {
                    print!("{heading}");
                }

                if step.time != 0 {
                    printer.justify(JustifyMode::RIGHT)?;
                    printer.writeln(&format!("({} min)", step.time))?;
                    printer.justify(JustifyMode::LEFT)?;

                    if args.verbose >= 2 {
                        print!(" ({} min)", step.time);
                    }
                }

                if args.ingredient_display == IngredientDisplay::Both || args.ingredient_display == IngredientDisplay::Step {
                    for ingredient in &step.ingredients {
                        if ingredient.include() {
                            printer.writeln(&format_text(&ingredient.pretty_print(), args.columns as usize))?;
                        }
                    }
                }

                if args.verbose >= 2 {
                    println!(" {}", format_text(&step.instruction, args.columns as usize));
                }


                printer.writeln(&format_text(&step.instruction, args.columns as usize))?;
                printer.feed()?;
            }

            if args.qr {
                printer.justify(JustifyMode::CENTER)?;
                printer.qrcode(&format!("{}/recipe/{}", args.instance, recipe.id))?;
                printer.justify(JustifyMode::LEFT)?;
            }

            printer.feed()?;

            match args.cut_mode {
                CutMode::None => (),
                CutMode::Partial => { printer.partial_cut()?;},
                CutMode::Full => { printer.cut()?; },
                CutMode::Pause => {
                    print_bar.set_message("Paused. Tear off recipe and press ENTER to continue");
                    // println!("Paused. Tear off recipe and press ENTER to continue");
                    stdin().read(&mut [0])?;
                }
            }

            printer.print()?;

        }
        print_bar.finish_with_message("Printing complete.");
    }
    Ok(())
}
