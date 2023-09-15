// Standard input/output library
use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    println!("Guess the number");

    let secret_number = rand::thread_rng().gen_range(1..=100);
    println!("The secret number is: {secret_number}");

    loop {
        println!("Please input your guess.");

        /*
         * Variable to store the user input
         * In Rust, variables are immutable by default
         * To make them mutable, we need to add mut before the variable name
         * String::new() returns a new instance of a String
         * :: indicates that new is an associated function of the String type
         * Associated functions are implemented on types, rather than on instances of types
         * Some languages call this a static method
         * The :: syntax is used for both associated functions and namespaces created by modules
         */
        let mut guess = String::new();

        // Receiving the user input
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line"); // Handling Potential Failure with Result

        // Converting the user input to a number
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed: {guess}");

        // Comparing the Guess to the Secret Number
        /*
         * Rust allows us to shadow the previous value of guess with a new one
         * This feature is often used in situations in which you want to convert a value from one type to another type
         * Shadowing lets us reuse the guess variable name rather than forcing us to create two unique variables
         * Shadowing is different from marking a variable as mut, because we’ll get a compile-time error if we accidentally try to reassign to this variable without using the let keyword
         */
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
