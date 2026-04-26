
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

mod tests {
    use super::*;
    use ecol_interpreter::EcolMachine;
    fn run(input: &str) -> Result<String, String> {
        let mut machine = EcolMachine::new();
        let mut output = String::new();
        machine.execute_direct(input, &mut |s| output.push_str(s))?;
        machine.execute_direct("NR(1)", &mut |s| output.push_str(s))
    }
    #[allow(dead_code)]
    fn run_raw(input: &str) -> Result<String, String> {
        let mut machine = EcolMachine::new();
        let mut output = String::new();
        machine.execute_direct(input, &mut |s| output.push_str(s))

    }
    fn run_program(programma: &str) -> Result<String, String> {
        let mut machine = EcolMachine::new();
        let mut output = String::new();
        for regel in programma.lines() {
            if !regel.trim().is_empty() {
                machine.execute_direct(regel, &mut |s| output.push_str(s))?;
            }
        }
        machine.execute_direct("START", &mut |s| output.push_str(s))
    }

    //enkelvoudige expressies
    #[wasm_bindgen_test] fn optellen() {
        assert_eq!(run("SCHRIJF(5,2) := 1 + 2"), Ok("    3.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn aftrekken() {
        assert_eq!(run("SCHRIJF(5,2) := 2 - 1"), Ok("    1.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn vermenigvuldigen() {
        assert_eq!(run("SCHRIJF(5,2) := 3 * 2"), Ok("    6.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn delen() {
        assert_eq!(run("SCHRIJF(5,2) := 8 / 2"), Ok("    4.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn delen_door_nul() {
        assert!(run("SCHRIJF(5,2) := 2 / 0").is_err());
    }
    #[wasm_bindgen_test] fn machtsverheffen() {
        assert_eq!(run("SCHRIJF(5,2) := 2 M 3"), Ok("    8.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn machtsverheffen_fout1() {
        assert!(run("SCHRIJF(5,2) := -2 M 1.5").is_err());
    }
    #[wasm_bindgen_test] fn machtsverheffen_fout2() {
        assert!(run("SCHRIJF(5,2) := 2 M 1000001").is_err());
    }
    #[wasm_bindgen_test] fn machtsverheffen_fout3() {
        assert!(run("SCHRIJF(5,2) := 3 M 1000000").is_err());
    }
    #[wasm_bindgen_test] fn prioriteit_1() {
        assert_eq!(run("SCHRIJF(5,2) := 3 + 2 M 3"), Ok("   11.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn prioriteit_2() {
        assert_eq!(run("SCHRIJF(5,2) := 3 + 2 * 3"), Ok("    9.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn prioriteit_3() {
        assert_eq!(run("SCHRIJF(5,2) := 3 * 2 M 3"), Ok("   24.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn haakjes() {
        assert_eq!(run("SCHRIJF(5,2) := (3 * 2) M 3"), Ok("  216.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn links_naar_rechts() {
        assert_eq!(run("SCHRIJF(5,2) := 8 - 3 - 4"), Ok("    1.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn negatief_resultaat() {
        assert_eq!(run("SCHRIJF(5,2) := 8 - 16"), Ok("    -8.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn negatieve_invoer() {
        assert_eq!(run("SCHRIJF(5,2) := 25 + -12"), Ok("   13.00\n".to_string()));
    }

    //Functies
    #[wasm_bindgen_test] fn abs() {
        assert_eq!(run("SCHRIJF(5,2) := ABS(-5.5)"), Ok("    5.50\n".to_string()));
    }
    #[wasm_bindgen_test] fn abs_positief() {
        assert_eq!(run("SCHRIJF(5,2) := ABS(3.5)"), Ok("    3.50\n".to_string()));
    }

    // ARCTAN: nulpunt, en ARCTAN(1) = π/4 als herkenbare waarde
    #[wasm_bindgen_test] fn arctan_nul() {
        assert_eq!(run("SCHRIJF(5,2) := ARCTAN(0)"), Ok("    0.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn arctan_een() {
        assert_eq!(run("SCHRIJF(1,4) := ARCTAN(1)"), Ok("0.7854\n".to_string()));
    }
    #[wasm_bindgen_test] fn arctan_negatief() {
        // ARCTAN accepteert alle reële getallen; −1 → −π/4
        assert_eq!(run("SCHRIJF(1,4) := ARCTAN(-1)"), Ok("-0.7854\n".to_string()));
    }
    #[wasm_bindgen_test] fn bovin() {
        assert_eq!(run_program("
        10 RIJ(3,8) r
        20 SCHRIJF(5,0) := BOVIN(r)
        30 NR
        40 KLAAR"), Ok("    8\n".to_string()));
    }
    // COS: nulpunt, en COS(π/2) ≈ 0 via ARCTAN(1)*2
    #[wasm_bindgen_test] fn cos_nul() {
        assert_eq!(run("SCHRIJF(5,2) := COS(0)"), Ok("    1.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn cos_halve_pi() {
        // cos(π/2) is in f32 niet exact 0 maar ≈ −4.4e-8; formatter geeft −0.00
        assert_eq!(run("SCHRIJF(5,2) := COS(ARCTAN(1)*2)"), Ok("    -0.00\n".to_string()));
    }
    // EXP
    #[wasm_bindgen_test] fn exp_nul() {
        assert_eq!(run("SCHRIJF(5,2) := EXP(0)"), Ok("    1.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn exp_een() {
        // e ≈ 2.72
        assert_eq!(run("SCHRIJF(5,2) := EXP(1)"), Ok("    2.72\n".to_string()));
    }
    #[wasm_bindgen_test] fn g_positief() {
        assert_eq!(run("SCHRIJF(5,2) := G(5.7)"), Ok("    5.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn g_negatief() {
        assert_eq!(run("SCHRIJF(5,2) := G(-5.7)"), Ok("    -6.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn gok_resultaat() {
        assert_eq!(run_program("
        10 resultaat := 0
        20 MET 1, i:=1, 100
        30 g := GOK(1, 1000)
        40 ALS g < 1 OF g > 1000 DAN 60 ANDERS 50
        50 HERHAAL
        55 NAAR 70
        60 resultaat := 1
        70 SCHRIJF(1,0) := resultaat
        80 NR
        90 KLAAR"), Ok("0\n".to_string()));
    }
    #[wasm_bindgen_test] fn gok_grenzen_gelijk_fout() {
        assert!(run("SCHRIJF(5,2) := GOK(3, 3)").is_err());
    }
    #[wasm_bindgen_test] fn gok_grenzen_omgekeerd_fout() {
        assert!(run("SCHRIJF(5,2) := GOK(5, 3)").is_err());
    }
    #[wasm_bindgen_test] fn gokc_resultaat() {
        assert_eq!(run_program("
        10 resultaat := 0
        20 MET 1, i:=1, 100
        30 g := GOKC
        40 ALS g < 0 OF g > 1 DAN 60 ANDERS 50
        50 HERHAAL
        55 NAAR 70
        60 resultaat := 1
        70 SCHRIJF(1,0) := resultaat
        80 NR
        90 KLAAR"), Ok("0\n".to_string()));
    }
    // LN
    #[wasm_bindgen_test] fn ln_een() {
        assert_eq!(run("SCHRIJF(5,2) := LN(1)"), Ok("    0.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn ln_tien() {
        assert_eq!(run("SCHRIJF(5,2) := LN(10)"), Ok("    2.30\n".to_string()));
    }
    #[wasm_bindgen_test] fn ln_breuk() {
        // regressietest: LN accepteert niet-gehele getallen na bugfix grens_bewaking
        assert_eq!(run("SCHRIJF(5,2) := LN(2.5)"), Ok("    0.92\n".to_string()));
    }
    #[wasm_bindgen_test] fn ln_nul_fout() {
        assert!(run("SCHRIJF(5,2) := LN(0)").is_err());
    }
    #[wasm_bindgen_test] fn ln_negatief_fout() {
        assert!(run("SCHRIJF(5,2) := LN(-1)").is_err());
    }
    // LOG
    #[wasm_bindgen_test] fn log_een() {
        assert_eq!(run("SCHRIJF(5,2) := LOG(1)"), Ok("    0.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn log_tien() {
        assert_eq!(run("SCHRIJF(5,2) := LOG(10)"), Ok("    1.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn log_honderd() {
        assert_eq!(run("SCHRIJF(5,2) := LOG(100)"), Ok("    2.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn log_nul_fout() {
        assert!(run("SCHRIJF(5,2) := LOG(0)").is_err());
    }
    #[wasm_bindgen_test] fn log_negatief_fout() {
        assert!(run("SCHRIJF(5,2) := LOG(-1)").is_err());
    }
    #[wasm_bindgen_test] fn ondin() {
        assert_eq!(run_program("
        10 RIJ(3,8) r
        20 SCHRIJF(5,0) := ONDIN(r)
        30 NR
        40 KLAAR"), Ok("    3\n".to_string()));
    }
    #[wasm_bindgen_test] fn ps_na_schrijf() {
        assert_eq!(run_program("
        20 SCHRIJF(5,0) := 25
        30 ps := PS
        35 NR
        36 SCHRIJF(5,0) := ps
        37 NR
        40 KLAAR"), Ok("   25\n    5\n".to_string()));
    }
    #[wasm_bindgen_test] fn ps_na_nr() {
        assert_eq!(run_program("
        20 SCHRIJF(5,0) := 25
        30 NR
        35 ps := PS
        36 SCHRIJF(5,0) := ps
        37 NR
        40 KLAAR"), Ok("   25\n    0\n".to_string()));
    }
    #[wasm_bindgen_test] fn sin_nul() {
        assert_eq!(run("SCHRIJF(5,2) := SIN(0)"), Ok("    0.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn sin_halve_pi() {
        assert_eq!(run("SCHRIJF(5,2) := SIN(ARCTAN(1)*2)"), Ok("    1.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn wrtl() {
        assert_eq!(run("SCHRIJF(5,2) := WRTL(25)"), Ok("    5.00\n".to_string()));
    }
    #[wasm_bindgen_test] fn wrtl_breuk() {
        // regressietest: WRTL accepteert niet-gehele getallen na bugfix grens_bewaking
        assert_eq!(run("SCHRIJF(5,2) := WRTL(2.25)"), Ok("    1.50\n".to_string()));
    }
    #[wasm_bindgen_test] fn wrtl_nul_fout() {
        assert!(run("SCHRIJF(5,2) := WRTL(0)").is_err());
    }
    #[wasm_bindgen_test] fn wrtl_negatief_fout() {
        assert!(run("SCHRIJF(5,2) := WRTL(-1)").is_err());
    }
    #[wasm_bindgen_test] fn stelling_van_pythagoras() {
        assert_eq!(run("SCHRIJF(5,2) := SIN(1) M 2 + COS(1) M 2"), Ok("    1.00\n".to_string()));
    }
    // Omgekeerde: EXP(LN(x)) ≈ x
    #[wasm_bindgen_test] fn exp_ln_inverse() {
        assert_eq!(run("SCHRIJF(5,2) := EXP(LN(5))"), Ok("    5.00\n".to_string()));
    }

    //Sleutelwoorden
    //Uitvoer
    #[wasm_bindgen_test] fn uitvoer() {

    }


}
