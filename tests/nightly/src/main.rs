// === Rust Nightly Test ===

#[crabtime::function]
fn gen_positions(components: Vec<String>) {
    for (ix, name) in components.iter().enumerate() {
        let dim = ix + 1;
        let cons = components[0..dim].join(",");
        crabtime::output! {
            enum Position{{dim}} {
                {{cons}}
            }
        }
    }
}
gen_positions!(["X", "Y", "Z", "W"]);

fn main() {
    let _p1 = Position2::X;
}
