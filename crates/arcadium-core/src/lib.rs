pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}





use arcadium_sdk::GameMetadata;

pub fn run(){
    let game = GameMetadata {
        name: String::from("Test Game"),
        author: String::from("Arcadium"),
        version: String::from("0.0.1"),
    };

    println!("Arcadium core started.");
    println!("Loaded game: {}", game.name);

}