extern crate error_chain;
extern crate fixture_setup;

use error_chain::ChainedError;
use fixture_setup::load::load;

fn main() {
    if let Err(ref err) = load() {
        println!("{}", err.display_chain());
    }
}
