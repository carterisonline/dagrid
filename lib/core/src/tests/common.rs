use crate::control::ControlGraph;
use glicol::Engine;
use owo_colors::OwoColorize;

#[macro_export]
macro_rules! assert_glicol_ref_eq {
    (within epsilon * $ex: literal: &mut $cg: ident * $n: literal == $src: expr) => {{
        let synthesized = $crate::tests::common::cg_samples::<$n>(&mut $cg);
        let reference = $crate::tests::common::glicol_ref::<$n>($src);
        let matches = $crate::tests::common::eq_matches::<$n>(&synthesized, &reference, $ex);

        if !matches.iter().all(|b| *b) {
            panic!(
                "{}",
                $crate::tests::common::nonmatching_report::<$n>(&synthesized, &reference, &matches)
            );
        }
    }};
}

pub fn nonmatching_report<const N: usize>(
    synthesized: &[f32],
    reference: &[f32],
    matches: &[bool],
) -> String {
    const STRING_COPY: String = String::new();
    let mut report_buf = [STRING_COPY; N];

    report_buf.iter_mut().enumerate().for_each(|(i, s)| {
        if matches[i] {
            *s = format!("  {}", synthesized[i].to_string().bright_blue());
        } else {
            *s = format!(
                "  {} {} {}",
                synthesized[i].bright_red(),
                "!=".bright_black(),
                reference[i].blue()
            );
        }
    });

    let max_deviation = synthesized
        .iter()
        .zip(reference.iter())
        .fold(0f32, |acc, (s, r)| acc.max((s - r).abs()));

    format!(
        "{}:\n\n[\n{}\n]\n\n{}{}\n    ({} * {})",
        "Synthesized result does not match glicol reference"
            .red()
            .bold(),
        report_buf.join("\n"),
        "Max deviation was ±".blue(),
        max_deviation.to_string().blue(),
        "EPSILON".blue(),
        (max_deviation as f64 / f32::EPSILON as f64)
            .to_string()
            .blue()
    )
}

pub fn eq_matches<const N: usize>(
    synthesized: &[f32],
    reference: &[f32],
    epsilon_range: usize,
) -> [bool; N] {
    let mut matches = [false; N];

    matches.iter_mut().enumerate().for_each(|(i, m)| {
        *m = (synthesized[i] - reference[i]).abs() < f32::EPSILON * epsilon_range as f32
    });

    matches
}

pub fn cg_samples<const N: usize>(cg: &mut ControlGraph) -> [f32; N] {
    let mut v = [0.0; N];
    for x in v.iter_mut() {
        *x = *(cg.next_sample()) as f32;
    }

    v
}

pub fn glicol_ref<const N: usize>(src: &str) -> Vec<f32> {
    let mut engine = Engine::<N>::new();

    engine.update_with_code(src);
    engine.next_block(vec![]).0[0].to_vec()
}
