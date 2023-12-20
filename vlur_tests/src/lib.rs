mod run;

use std::fs;

use run::{run, StartupOption::*};

pub fn test(vimrc: &str) {
    run(vimrc, true, &[Headless, SetRtp, QuitWithCode]);
}

pub fn test_cache(vimrc: &str) {
    run(vimrc, false, &[Headless, SetRtp, QuitWithCode]);
}

pub fn bench(vimrc: &str) {
    let count = 70;
    let warmup = 30;
    let file = "__startuptime.log";
    let opts = &[Env("LANG", "C"), Headless, StartupTime(file), Quit];
    let mut logs = Vec::with_capacity(warmup + count);

    run(vimrc, true, opts);
    for _ in 0..logs.capacity() {
        run(vimrc, false, opts);
        logs.push(fs::read_to_string(file).unwrap());
        fs::remove_file(file).unwrap();
    }

    let results: Vec<f32> = logs[warmup..]
        .iter()
        .map(|log| {
            log.lines()
                .last()
                .unwrap()
                .split_once(' ')
                .unwrap()
                .0
                .parse::<f32>()
                .unwrap()
        })
        .collect();
    println!(
        "mean: {mean}\t min: {min}\t max: {max}",
        mean = results.iter().sum::<f32>() / count as f32,
        min = results.iter().fold(f32::NAN, |m, v| v.min(m)),
        max = results.iter().fold(f32::NAN, |m, v| v.max(m)),
    );
}
