
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
    #[wasm_bindgen_test] fn nr_grenzen_geldig() {
        assert!(run_program("10 TEKST := \"x\"\n20 NR(1)\n30 KLAAR").is_ok());
        assert!(run_program("10 TEKST := \"x\"\n20 NR(99)\n30 KLAAR").is_ok());
    }
    #[wasm_bindgen_test] fn nr_nul_is_fout() {
        assert!(run_program("10 TEKST := \"x\"\n20 NR(0)\n30 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn nr_negatief_is_fout() {
        assert!(run_program("10 TEKST := \"x\"\n20 NR(-1)\n30 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn nr_te_groot_is_fout() {
        assert!(run_program("10 TEKST := \"x\"\n20 NR(100)\n30 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn nr_uitgebreid() {
        assert_eq!(run_program("
        10 TEKST := \"abc\"
        20 NR(5)
        30 TEKST := \"def\"
        40 NR
        50 KLAAR"), Ok("abc\n\n\n\n\ndef\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_geen_format_geheel_getal() {
        assert_eq!(run("SCHRIJF := 25"), Ok("    25\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_geen_format_breuk() {
        assert_eq!(run("SCHRIJF := 25.5"), Ok("    25.5000\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_geen_format_breuk_negatief() {
        assert_eq!(run("SCHRIJF := -25.5"), Ok("    -25.5000\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_geen_format_geheel_getal_negatief() {
        assert_eq!(run("SCHRIJF := -25"), Ok("    -25\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_format_overflow_fout() {
        assert!(run("SCHRIJF(3,0) := 1000").is_err());
    }
    #[wasm_bindgen_test] fn schrijf_format_round_up() {
        assert_eq!(run("SCHRIJF(5,1) := 25.45"), Ok("   25.5\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_format_round_down() {
        assert_eq!(run("SCHRIJF(5,1) := 25.44"), Ok("   25.4\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijf_format_krankzinnig_groot() {
        // 1234567890.12... overschrijdt de f32-precisie (>0.5 afrondingsfout)
        assert!(run("SCHRIJF(10,13) := 1234567890.12345678").is_err());
    }
    #[wasm_bindgen_test] fn schrijfsym_ok() {
        assert_eq!(run("SCHRIJFSYM := 82"), Ok("*\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijfsym_negatief() {
        assert!(run("SCHRIJFSYM := -1").is_err());
    }
    #[wasm_bindgen_test] fn schrijfsym_te_hoog() {
        assert!(run("SCHRIJFSYM := 100").is_err());
    }
    #[wasm_bindgen_test] fn schrijfsym_gebroken_getal() {
        assert!(run("SCHRIJFSYM := 1.5").is_err());
    }
    #[wasm_bindgen_test] fn schrijfsym_geen_symbool() {
        assert!(run("SCHRIJFSYM := 93").is_err());
    }
    #[wasm_bindgen_test] fn schrijm_positief() {
        assert_eq!(run("SCHRIJM := 1123456.321654"), Ok("+0.11234564E+7\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijm_negatief() {
        assert_eq!(run("SCHRIJM := -1123456.321654"), Ok("-0.11234564E+7\n".to_string()));
    }
    #[wasm_bindgen_test] fn schrijm_klein() {
        assert_eq!(run("SCHRIJM := 0.000000012345"), Ok("+0.12345E-7\n".to_string()));
    }
    #[wasm_bindgen_test] fn spatie() {
        assert_eq!(run_program("
        10 TEKST := \"abc\"
        20 SPATIE
        30 TEKST := \"def\"
        40 NR
        50 KLAAR"), Ok("abc def\n".to_string()));
    }
    #[wasm_bindgen_test] fn spaties() {
        assert_eq!(run_program("
        10 TEKST := \"abc\"
        20 SPATIE (5)
        30 TEKST := \"def\"
        40 NR
        50 KLAAR"), Ok("abc     def\n".to_string()));
    }
    #[wasm_bindgen_test] fn spatie_te_klein() {
        assert!(run_program("
        10 SPATIE (0)
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn spatie_te_groot() {
        assert!(run_program("
        10 SPATIE (81)
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn spatie_gebroken_getal() {
        assert!(run_program("
        10 SPATIE (1.5)
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn tekst() {
        assert_eq!(run("TEKST := \"abcdef\""), Ok("abcdef\n".to_string()));
    }
    #[wasm_bindgen_test] fn np() {
        assert!(!run("NP").is_err());
    }

    //Stroombesturing
    //Eerst: NAAR. Als dat werkt hoeft die functionaliteit niet meer getest te worden bij ALS
    #[wasm_bindgen_test] fn naar_vooruit_geldig() {
        assert_eq!(run_program("
        10 a:=36
        20 NAAR 40
        30 a:=63
        40 SCHRIJF(2,0):=a
        50 NR
        60 KLAAR"), Ok("36\n".to_string()));
    }
    #[wasm_bindgen_test] fn naar_vooruit_ongeldig_in_range() {
        assert!(run_program("
        10 a:=36
        20 NAAR 21
        30 a:=63
        40 SCHRIJF(2,0):=a
        50 NR
        60 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn naar_vooruit_ongeldig_buiten_range() {
        assert!(run_program("
        10 a:=36
        20 NAAR 100
        30 a:=63
        40 SCHRIJF(2,0):=a
        50 NR
        60 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn naar_achteruit_geldig() {
        assert_eq!(run_program("
        10 a:=36
        20 NAAR 40
        30 a:=63
        32 SCHRIJF(2,0):=a
        34 NAAR 50
        40 SCHRIJF(2,0):=a
        42 NAAR 30
        50 NR
        60 KLAAR"), Ok("3663\n".to_string()));
    }
    #[wasm_bindgen_test] fn naar_achteruit_rij_gewist() {
        assert!(run_program("
        10 NAAR 100
        20 aap(5) = 100
        30 SCHRIJF(3,0):=aap(5)
        40 NR
        50 KLAAR
        100 RIJ (3,48) aap
        110 NAAR 20").is_err());
    }
    #[wasm_bindgen_test] fn naar_naar_functie() {
        assert!(run_program("
        10 NAAR 110
        20 aap := 100
        30 SCHRIJF(3,0):=aap(2)
        40 NR
        50 KLAAR
        100 FUN aap(b)
        110 a:=100 * b
        120 FUN := a").is_err());
    }
    #[wasm_bindgen_test] fn naar_uit_functie() {
        assert!(run_program("
        30 SCHRIJF(3,0):=aap(2)
        40 NR
        50 KLAAR
        100 FUN aap(b)
        110 a:=100 * b
        115 NAAR 30
        120 FUN := aap").is_err());
    }
    #[wasm_bindgen_test] fn naar_binnen_functie() {
        assert_eq!(run_program("
        30 SCHRIJF(3,0):=aap(1)
        40 NR
        50 KLAAR
        100 FUN aap(b)
        105 NAAR 115
        110 a:=100
        115 a:=120 * b
        120 FUN := a"), Ok("120\n".to_string()));
    }
    #[wasm_bindgen_test] fn naar_naar_sub() {
        assert!(run_program("
        10 NAAR 110
        20 aap = 100
        30 SCHRIJF(3,0):=a
        40 NR
        50 KLAAR
        100 SUB aapje
        110 a:=100
        120 END").is_err());
    }
    #[wasm_bindgen_test] fn naar_buiten_sub() {
        assert!(run_program("
        20 aapje
        30 SCHRIJF(3,0):=a
        40 NR
        50 KLAAR
        100 SUB aapje
        105 NAAR 30
        110 a:=100
        120 END").is_err());
    }
    #[wasm_bindgen_test] fn naar_binnen_sub() {
        assert_eq!(run_program("
        20 aapje
        30 SCHRIJF(3,0):=a
        40 NR
        50 KLAAR
        100 SUB aapje
        105 NAAR 115
        110 a:=100
        115 a:=120
        120 END"), Ok("120\n".to_string()));
    }
    #[wasm_bindgen_test] fn naar_lege_regel() {
        assert_eq!(run_program("
        20 NAAR 100
        30 SCHRIJF(3,0):=a
        40 NR
        50 KLAAR
        100 : ;lege regel
        110 a:=100
        120 NAAR 30"), Ok("100\n".to_string()));
    }
    //ALS
    #[wasm_bindgen_test] fn als_zonder_anders() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        30 ALS a > b DAN 60
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("10\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_met_anders() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        30 ALS a > b DAN 60 ANDERS 80
        40 SCHRIJF(2,0) := a
        50 NAAR 100
        60 SCHRIJF(2,0) := b
        70 NAAR 100
        80 TEKST := \"anders\"
        100 NR
        110 KLAAR"), Ok("anders\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_complexe_vergelijking() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        30 ALS a <> b EN ((3 < 1) OF (100 = 100)) DAN 60
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("20\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_unicode_1() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        30 ALS a ≠ b DAN 60
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("20\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_unicode_2() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        30 ALS a ≥ b DAN 60
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("10\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_unicode_3() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        30 ALS a ≤ b DAN 60
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("20\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_dan_stop() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        25 SCHRIJF(2,0) := a
        27 NR
        30 ALS a <= b DAN STOP
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("10\n".to_string()));
    }
    #[wasm_bindgen_test] fn als_dan_anders_stop() {
        assert_eq!(run_program("
        10 a := 10
        20 b := 20
        25 SCHRIJF(2,0) := a
        27 NR
        30 ALS a >= b DAN 60 ANDERS STOP
        40 SCHRIJF(2,0) := a
        50 NAAR 70
        60 SCHRIJF(2,0) := b
        70 NR
        80 KLAAR"), Ok("10\n".to_string()));
    }
    //MET
    #[wasm_bindgen_test] fn met_omhoog() {
        assert_eq!(run_program("
        10 a := 0
        20 MET 1, teller := 1, 10
        30 a := a + teller
        40 HERHAAL
        50 SCHRIJF(2,0) := a
        60 NR
        70 KLAAR"), Ok("55\n".to_string()));
    }
    #[wasm_bindgen_test] fn met_omlaag() {
        assert_eq!(run_program("
        10 a := 100
        20 MET -1, teller := 10, 1
        30 a := a - teller
        40 HERHAAL
        50 SCHRIJF(2,0) := a
        60 NR
        70 KLAAR"), Ok("45\n".to_string()));
    }
    #[wasm_bindgen_test] fn met_breuk() {
        assert_eq!(run_program("
        10 a := 0.1
        20 MET 0.15, teller := 0.5, 3.4
        30 a := a + teller
        40 HERHAAL
        50 SCHRIJF(2,1) := a
        60 NR
        70 KLAAR"), Ok("38.6\n".to_string()));
    }
    #[wasm_bindgen_test] fn met_omlaag_naar_negatief() {
        assert_eq!(run_program("
        10 a := 100
        20 MET -1, teller := 4, -5
        30 a := a + teller
        40 HERHAAL
        50 SCHRIJF(2,0) := a
        60 NR
        70 KLAAR"), Ok("95\n".to_string()));
    }
    #[wasm_bindgen_test] fn met_geen_herhaling() {
        assert_eq!(run_program("
        10 a := 100
        20 MET -1, teller := 4, 6
        30 a := a + teller
        40 HERHAAL
        50 SCHRIJF(3,0) := a
        60 NR
        70 KLAAR"), Ok("100\n".to_string()));
    }
    #[wasm_bindgen_test] fn met_genest_ok() {
        assert_eq!(run_program("
        10 MET 1, tel1 := 1, 3
        20 MET 2, tel2 := 4, 8
        30 a := tel1 * tel2
        40 HERHAAL
        45 HERHAAL
        50 SCHRIJF(2,0) := a
        60 NR
        70 KLAAR"), Ok("24\n".to_string()));
    }
    #[wasm_bindgen_test] fn met_zonder_herhaal() {
        assert!(run_program("
        20 MET 2, tel2 := 4, 8
        30 a := 3 * tel2
        50 SCHRIJF(2,0) := a
        60 NR
        70 KLAAR").is_err());
    }

    //RIJ(SYM)
    #[wasm_bindgen_test] fn rij_ok() {
        assert_eq!(run_program("
        10 RIJ(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        50 SCHRIJF(1,0) := a(5)
        60 NR
        70 KLAAR"), Ok("5\n".to_string()));
    }
    #[wasm_bindgen_test] fn rij_onderste_index() {
        assert_eq!(run_program("
        10 RIJ(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller + 8
        40 HERHAAL
        50 SCHRIJF(2,0) := a(3)
        60 NR
        70 KLAAR"), Ok("11\n".to_string()));
    }
    #[wasm_bindgen_test] fn rij_bovenste_index() {
        assert_eq!(run_program("
        10 RIJ(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller + 4
        40 HERHAAL
        50 SCHRIJF(2,0) := a(10)
        60 NR
        70 KLAAR"), Ok("14\n".to_string()));
    }
    #[wasm_bindgen_test] fn rij_beneden_index() {
        assert!(run_program("
        10 RIJ(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        50 SCHRIJF(1,0) := a(2)
        60 NR
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_boven_index() {
        assert!(run_program("
        10 RIJ(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        50 SCHRIJF(1,0) := a(11)
        60 NR
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_nul_index() {
        assert!(run_program("
        10 RIJ(0,10) a
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_ok() {
        assert_eq!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller + 7
        40 HERHAAL
        50 SCHRIJFSYM := a(5)
        60 NR
        70 KLAAR"), Ok("c\n".to_string()));
    }
    #[wasm_bindgen_test] fn rijsym_onderste_index() {
        assert_eq!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller + 8
        40 HERHAAL
        50 SCHRIJFSYM := a(3)
        60 NR
        70 KLAAR"), Ok("b\n".to_string()));
    }
    #[wasm_bindgen_test] fn rijsym_bovenste_index() {
        assert_eq!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller + 4
        40 HERHAAL
        50 SCHRIJFSYM := a(10)
        60 NR
        70 KLAAR"), Ok("e\n".to_string()));
    }
    #[wasm_bindgen_test] fn rijsym_beneden_index() {
        assert!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        50 SCHRIJFSYM := a(2)
        60 NR
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_boven_index() {
        assert!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        50 SCHRIJFSYM := a(11)
        60 NR
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_onder_nul() {
        assert!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        45 a(3) := -1
        50 SCHRIJFSYM := a(5)
        60 NR
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_boven_honderd() {
        assert!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL
        45 a(3) := 100
        50 SCHRIJFSYM := a(5)
        60 NR
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_nul_index() {
        assert!(run_program("
        10 RIJSYM(0,10) a
        70 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_begin_te_groot() {
        assert!(run_program("
        10 RIJ(10000,10001) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_einde_te_groot() {
        assert!(run_program("
        10 RIJ(1,10000) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_einde_gelijk_begin() {
        assert!(run_program("
        10 RIJ(5,5) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_einde_voor_begin() {
        assert!(run_program("
        10 RIJ(5,3) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rij_gebroken_getal() {
        assert!(run_program("
        10 RIJ(1.5,5) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_begin_te_groot() {
        assert!(run_program("
        10 RIJSYM(10000,10001) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_einde_te_groot() {
        assert!(run_program("
        10 RIJSYM(1,10000) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_einde_gelijk_begin() {
        assert!(run_program("
        10 RIJSYM(5,5) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_einde_voor_begin() {
        assert!(run_program("
        10 RIJSYM(5,3) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn rijsym_gebroken_getal() {
        assert!(run_program("
        10 RIJSYM(1.5,5) a
        20 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn commentaar_ok() {
        assert_eq!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL ;commentaar
        50 SCHRIJFSYM := a(5)
        60 NR
        70 KLAAR"), Ok("5\n".to_string()));
    }
    #[wasm_bindgen_test] fn commentaar_fout() {
        assert!(run_program("
        10 RIJSYM(3,10) a
        20 MET 1, teller := 3, 10
        30 a(teller) := teller
        40 HERHAAL  commentaar
        50 SCHRIJFSYM := a(5)
        60 NR
        70 KLAAR").is_err());
    }
    //SUB
    #[wasm_bindgen_test] fn sub_aanroep_forward() {
        assert_eq!(run_program("
        10 aapje
        20 SCHRIJF(3,0) := bel
        30 NR
        40 KLAAR
        100 SUB aapje
        110 bel := 233
        120 END"), Ok("233\n".to_string()));
    }
    #[wasm_bindgen_test] fn sub_aanroep_backward() {
        assert_eq!(run_program("
        10 SUB aapje
        20 bel := 238
        30 END
        100 aapje
        110 SCHRIJF(3,0) := bel
        120 NR
        130 KLAAR"), Ok("238\n".to_string()));
    }
    #[wasm_bindgen_test] fn sub_geneste_aanroep_zelf() {
        assert_eq!(run_program("
        10 SUB aapje
        20 bel := bel * 10
        30 ALS bel < 1000 DAN 40 ANDERS 50
        40 aapje
        50 END
        90 bel := 3
        100 aapje
        110 SCHRIJF(4,0) := bel
        120 NR
        130 KLAAR"), Ok("3000\n".to_string()));
    }
    #[wasm_bindgen_test] fn sub_geneste_aanroep_ander() {
        assert_eq!(run_program("
        10 SUB aapje
        20 bel := bel * 10
        30 ALS bel < 1000 DAN 40 ANDERS 50
        40 aapje
        50 aapje2
        60 END
        90 bel := 3
        100 aapje
        110 SCHRIJF(4,0) := bel
        115 SCHRIJF(3,0) := belletje
        120 NR
        130 KLAAR
        140 SUB aapje2
        150 belletje := 20
        160 END
        "), Ok("3000 20\n".to_string()));
    }
    #[wasm_bindgen_test] fn sub_onbekende_aanroep() {
        assert!(run_program("
        10 SUB aapje
        20 bel := bel * 10
        30 ALS bel < 1000 DAN 40 ANDERS 50
        40 aapje
        50 END
        90 bel := 3
        100 aapjes
        110 SCHRIJF(4,0) := bel
        120 NR
        130 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn sub_rij_reservering() {
        assert_eq!(run_program("
        10 SUB aapje
        20 RIJ(5,8) lijst
        30 END
        100 aapje
        110 MET 1, tel := ONDIN(lijst), BOVIN(lijst)
        120 lijst(tel) := tel * 11
        130 HERHAAL
        140 totaal := 0
        150 MET 1, tel := ONDIN(lijst), BOVIN(lijst)
        160 totaal := totaal + lijst(tel)
        170 HERHAAL
        180 SCHRIJF(3,0) := totaal
        190 NR
        200 KLAAR"), Ok("286\n".to_string()));
    }
    //FUN
    #[wasm_bindgen_test] fn fun_aanroep_forward() {
        assert_eq!(run_program("
        10 a := max(5)
        20 SCHRIJF(3,0) := a
        30 NR
        40 KLAAR
        100 FUN max(aap)
        110 res := aap * 100
        120 FUN := res"), Ok("500\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_aanroep_backward() {
        assert_eq!(run_program("
        10 FUN max(aap)
        20 res := aap * 80
        30 FUN := res
        100 a := max(6)
        110 SCHRIJF(3,0) := a
        120 NR
        130 KLAAR"), Ok("480\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_geneste_aanroep_zelf() {
        assert_eq!(run_program("
        10 FUN joop(beer)
        20 res := beer + 800
        30 ALS res < 5000 DAN 40 ANDERS 50
        40 res := joop(res)
        50 FUN := res
        100 a := joop(6)
        110 SCHRIJF(4,0) := a
        120 NR
        130 KLAAR"), Ok("5606\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_geneste_oneindig_fout() {
        assert!(run_program("
        10 FUN joop(beer)
        20 res := beer + 80
        40 res := joop(res)
        50 FUN := res
        100 a := joop(6)
        110 SCHRIJF(4,0) := a
        120 NR
        130 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geneste_aanroep_ander() {
        assert_eq!(run_program("
        10 FUN joop(beer)
        20 res := beer + 800
        30 ALS res < 5000 DAN 40 ANDERS 60
        40 res := joop(res)
        50 NAAR 70
        60 res := klaas(res)
        70 FUN := res
        100 a := joop(6)
        110 SCHRIJF(4,0) := a
        120 NR
        130 KLAAR
        200 FUN klaas(hert)
        210 res := hert - 100
        220 FUN := res"), Ok("5506\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_rij_doorgeven() {
        assert_eq!(run_program("
        10 FUN rijtest(RIJ lijst)
        20 res := 0
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 res := res + lijst(teller)
        50 HERHAAL
        60 FUN := res
        100 RIJ(3,7) lijstje
        110 met := 0
        120 MET 1, teller := ONDIN(lijstje), BOVIN(lijstje)
        130 lijstje(teller) := GOK(1, 1000)
        140 met := met + lijstje(teller)
        150 HERHAAL
        160 ALS met = rijtest(lijstje) DAN 170 ANDERS 190
        170 TEKST := \"OK\"
        180 NAAR 200
        190 TEKST := \"Fout\"
        200 NR
        210 KLAAR"), Ok("OK\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_rijsym_doorgeven() {
        assert_eq!(run_program("
        10 FUN rijtest(RIJSYM lijst)
        20 res := 0
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 res := res + lijst(teller)
        50 HERHAAL
        60 FUN := res
        100 RIJSYM(3,7) lijstje
        110 met := 0
        120 MET 1, teller := ONDIN(lijstje), BOVIN(lijstje)
        130 lijstje(teller) := GOK(1, 99)
        140 met := met + lijstje(teller)
        150 HERHAAL
        160 ALS met = rijtest(lijstje) DAN 170 ANDERS 190
        170 TEKST := \"OK\"
        180 NAAR 200
        190 TEKST := \"Fout\"
        200 NR
        210 KLAAR"), Ok("OK\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_geen_variabelen_uit_hoofdprogramma() {
        assert!(run_program("
        10 FUN vartest(a)
        20 FUN := a + b
        100 b:=10
        110 SCHRIJF(3,0) := vartest(b)
        120 NR
        130 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_variabelen_uit_functie() {
        assert!(run_program("
        10 FUN varitest(a)
        20 bof := 10
        30 FUN := a + bof
        110 SCHRIJF(3,0) := varitest(b) + bof
        120 NR
        130 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_meerdere_parameters() {
        assert_eq!(run_program("
        10 FUN multitest(a, RIJ lijst)
        20 bof := a
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 bof := bof + lijst(teller)
        50 HERHAAL
        60 FUN := bof
        100 param1 := 10
        110 RIJ (4,5) param2
        120 MET 1, teller := ONDIN(param2), BOVIN(param2)
        130 param2(teller) := teller + 4
        140 HERHAAL
        150 SCHRIJF(2,0) := multitest(param1, param2)
        160 NR
        170 KLAAR"), Ok("27\n".to_string()));
    }
    #[wasm_bindgen_test] fn fun_te_weinig_parameters() {
        assert!(run_program("
        10 FUN multitest(a, RIJ lijst)
        20 bof := a
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 bof := bof + lijst(teller)
        50 HERHAAL
        60 FUN := bof
        100 param1 := 10
        110 RIJ (4,5) param2
        120 MET 1, teller := ONDIN(param2), BOVIN(param2)
        130 param2(teller) := teller + 4
        140 HERHAAL
        150 SCHRIJF(2,0) := multitest(param1)
        160 NR
        170 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_te_veel_parameters() {
        assert!(run_program("
        10 FUN multitest(a, RIJ lijst)
        20 bof := a
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 bof := bof + lijst(teller)
        50 HERHAAL
        60 FUN := bof
        100 param1 := 10
        110 RIJ (4,5) param2
        120 MET 1, teller := ONDIN(param2), BOVIN(param2)
        130 param2(teller) := teller + 4
        140 HERHAAL
        150 SCHRIJF(2,0) := multitest(param1, param2, 100)
        160 NR
        170 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_naam_als_variabele() {
        assert!(run_program("
        10 FUN multitest(a, RIJ lijst)
        20 bof := a
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 bof := bof + lijst(teller)
        50 HERHAAL
        60 FUN := bof
        100 param1 := 10
        105 multitest := 12
        110 RIJ (4,5) param2
        120 MET 1, teller := ONDIN(param2), BOVIN(param2)
        130 param2(teller) := teller + 4
        140 HERHAAL
        150 SCHRIJF(2,0) := multitest(param1, param2)
        160 NR
        170 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_verkeerde_parameter() {
        assert!(run_program("
        10 FUN multitest(a, RIJ lijst)
        20 bof := a
        30 MET 1, teller := ONDIN(lijst), BOVIN(lijst)
        40 bof := bof + lijst(teller)
        50 HERHAAL
        60 FUN := bof
        100 param1 := 10
        110 RIJ (4,5) param2
        120 MET 1, teller := ONDIN(param2), BOVIN(param2)
        130 param2(teller) := teller + 4
        140 HERHAAL
        150 SCHRIJF(2,0) := multitest(param1, param1)
        160 NR
        170 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_schrijf() {
        assert!(run_program("
        10 FUN uitvoertest1(a)
        20 SCHRIJF := a
        30 FUN := a
        100 SCHRIJF := uitvoertest1(1)
        110 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_schrijfsym() {
        assert!(run_program("
        10 FUN uitvoertest2(a)
        20 SCHRIJFSYM := a
        30 FUN := a
        100 SCHRIJF := uitvoertest2(1)
        110 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_schrijm() {
    assert!(run_program("
        10 FUN uitvoertest3(a)
        20 SCHRIJM := a
        30 FUN := a
        100 SCHRIJF := uitvoertest3(1)
        110 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_spatie() {
        assert!(run_program("
        10 FUN uitvoertest4(a)
        20 SPATIE
        30 FUN := a
        100 SCHRIJF := uitvoertest4(1)
        110 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_tekst() {
        assert!(run_program("
        10 FUN uitvoertest7(a)
        20 TEKST := \"abcd\"
        30 FUN := a
        100 SCHRIJF := uitvoertest7(1)
        110 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_nr() {
        assert!(run_program("
        10 FUN uitvoertest5(a)
        20 NR
        30 FUN := a
        100 SCHRIJF := uitvoertest5(1)
        110 KLAAR").is_err());
    }
    #[wasm_bindgen_test] fn fun_geen_uitvoer_np() {
        assert!(run_program("
        10 FUN uitvoertest6(a)
        20 NP
        30 FUN := a
        100 SCHRIJF := uitvoertest6(1)
        110 KLAAR").is_err());
    }



}
