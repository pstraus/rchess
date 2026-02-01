fn main() {
    // Delegate to library code so core logic is testable in `src/lib.rs`.
    println!("{}", rchess::greet());
}
