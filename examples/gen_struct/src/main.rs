#[crabtime::function]
fn gen_struct(components: Vec<String>) {
    let fields = components
        .iter()
        .map(|x| format!("{}: &'static str", x))
        .collect::<Vec<String>>()
        .join(",");

    crabtime::output! {
        #[derive(Debug)]
        struct User {
            {{fields}}
        }
    }
}

fn main() {
    gen_struct!(["name", "last_name", "birthdate"]);

    let data = User {
        name: "John",
        last_name: "Doe",
        birthdate: "01/01/1970",
    };

    println!("{data:?}")
}
