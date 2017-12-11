
extern crate piccolo;

fn main() {
    let code =
        //"32 + -4.5 - 3 == 72 * 3 && 4 != 5";
        //"false == false";
        //"\"strang\" == \"string\"";
        //"\"string\" == \"string\"";
        //"me 32 + 32\n\nme true\n\n\nme \"it is wednesday, my dudes\"\n";
        //"a = 0.1\n\nb=0.2\nme a + b == 0.3\na = 9\nme a + b";
        //"a = 2 b = 3 a b = 4";
        //"me a = 2";
        "x = \"yes\"\nx or or or or";

    println!("program:\n{}\n", code);
    let s = piccolo::scanner::Scanner::new(code.into()).scan_tokens();

    if s.is_err() {
        println!("scan err!\n{}", s.err().unwrap());
    } else {
        println!("tokens:");
        for tok in s.clone().unwrap() {
            println!("{:?}", tok);
        }
        println!();

        let p = piccolo::parser::Parser::new(s.unwrap()).parse();

        if p.is_err() {
            println!("parse err!\n{}", p.err().unwrap());
        } else {
            println!("statements:");
            for stmt in p.clone().unwrap() {
                println!("{:?}", stmt);
            }

            let mut interp = piccolo::interp::Interpreter::new();
            println!("\noutput:");
            let i = interp.interpret(&p.unwrap());

            println!();

            if i.is_err() {
                println!("runtime err!\n{}", i.err().unwrap());
            } else {
                println!("huzzah!\n{:?}", interp.env);
            }
        }
    }
}

