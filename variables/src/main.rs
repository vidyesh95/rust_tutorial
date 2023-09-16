/*
* Variables are immutable by default.
* A variable can be made mutable by adding mut in front of the variable name.
* Constants are always immutable.
* Constants are declared using the const keyword instead of the let keyword.
* Constants can be declared in any scope, including the global scope.
* Rust’s naming convention for constants is to use all uppercase with underscores between words.
* We can shadow a variable by using the same variable’s name and repeating the use of the let
* keyword.
* Shadowing is different from marking a variable as mut, because we’ll get a compile-time error
* if we accidentally try to reassign to this variable without using the let keyword.
* By using let, we can perform a few transformations on a value but have the variable be
* immutable after those transformations have been completed.
*/
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
fn main() {
    let mut x = 5;
    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");

    println!("The value of THREE_HOURS_IN_SECONDS is: {THREE_HOURS_IN_SECONDS}");

    // Shadowing
    let y = 5;
    let y = y + 1;
    {
        let y = y * 2;
        println!("The value of y in the inner scope is: {y}");
    }
    println!("The value of y is: {y}");

    /*
    * The other difference between mut and shadowing is that because we’re effectively creating a
    * new variable when we use the let keyword again, we can change the type of the value but
    * reuse the same name.
    */
    let spaces = "   ";
    let spaces = spaces.len();
    println!("The value of spaces is: {spaces}");
}
