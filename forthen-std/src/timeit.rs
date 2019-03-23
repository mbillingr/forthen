use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;

use std::time::Instant;

/// Load basic operations into the dictionary
pub fn timeit(state: &mut State) -> Result<()> {
    state.new_mod("timeit".to_string())?;

    state.add_native_word("timeit", "(..a setup( -- ..b) run(..c -- ..d) -- ..a)", |state| {
        let runner = state.pop()?;
        let setup = state.pop()?;

        let mut times = vec![];

        let total = Instant::now();

        while total.elapsed().as_millis() < 100 || times.len() < 3 {
            let sub = &mut state.substate();
            setup.call(sub)?;

            let start = Instant::now();
            runner.call(sub)?;
            times.push(start.elapsed().as_nanos() as f64);
        }

        let mut mean = times.iter().sum::<f64>() / times.len() as f64;
        let var = times.iter().map(|t| (t - mean) * (t - mean)).sum::<f64>() / (times.len() as f64 - 1.0);
        let mut std = var.sqrt();

        let mut mean_units = -9;
        let mut std_units = -9;

        while mean >= 1e3 && mean_units < 0 {
            mean /= 1e3;
            mean_units += 3
        }

        while std >= 1e3 && std_units < 0 {
            std /= 1e3;
            std_units += 3
        }

        println!("{} runs, average {:.1} {} (± {:.1} {} std)", times.len(), mean, time_units(mean_units), std, time_units(std_units));


        Ok(())
    });

    state.exit_mod().unwrap();

    Ok(())
}

fn time_units(exp: i32) -> &'static str {
    match exp {
        0 => "s",
        -3 => "ms",
        -6 => "µs",
        -9 => "ns",
        _ => panic!("Invalid time exponent: {}", exp)
    }
}
