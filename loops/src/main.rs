/**
* Repetition with Loops
* Rust has three kinds of loops: loop, while, and for.
* You can use a loop to repeatedly call code while a condition evaluates to true, to break out of a
* loop, or to return a value from a loop.
* You can use a while loop when you’re unsure of how many times you’ll need to execute the loop
* body.
* You can use a for loop to execute some code for each item in a collection.
* If you want to run some code a certain number of times, it’s best to use a for loop.
* The for loop is best when you want to do something with each item in a collection in turn, like
* printing its value or modifying it in some way.
* If you want to loop over each element in a collection, you can use a for loop.
*/
fn main() {
    println!("Repeating Code with loop");
    loop {
        println!("again!");
        break;
    }

    println!("Returning Values from Loops");
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2; // break with value
        }
    };
    println!("The result is {result}");

    println!("Loops labels to disambiguate between multiple loops");
    let mut count = 0;
    'counting_up: loop {
        println!("count={count}");
        let mut remaining = 10;
        loop {
            println!("remaining={remaining}");
            if remaining == 9 {
                break;
            }
            if count == 2 {
                break 'counting_up;
            }
            remaining -= 1;
        }
        count += 1;
    }
    println!("End count= {count}");

    println!("Conditional Loops with while");
    let mut number = 3;
    while number != 0 {
        println!("{}!", number);
        number -= 1;
    }
    println!("LIFTOFF!!!");
}
