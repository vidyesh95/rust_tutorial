/**
* OwnerShip in Rust
* Ownership enables Rust to make memory safety guarantees without needing a garbage collector.
* Ownership is a set of rules that govern how a Rust program manages memory.
* The memory is managed through a system of ownership with a set of rules that the compiler checks
* at compile time. If any of the rules are violated, the program will not compile. None of the
* features of ownership will slow down your program while it’s running.
*
* Ownership Rules
* 1. Each value in Rust has a variable that’s called its owner.
* 2. There can only be one owner at a time.
* 3. When the owner goes out of scope, the value will be dropped.
*
* Variable Scope
* A scope is the range within a program for which an item is valid.
*
* The Stack and the Heap
* In many programming languages, you don’t have to think about the stack and the heap very often.
* But in a systems programming language like Rust, whether a value is on the stack or the heap has
* more of an effect on how the language behaves and why you have to make certain decisions.
* Both the stack and the heap are parts of memory available to your code to use at runtime, but they are structured in different ways.
*
* Stack
* Last In First Out (LIFO)
* All data stored on the stack must have a known, fixed size.
* Data with an unknown size at compile time or a size that might change must be stored on the heap
* instead.
* Accessing data on the stack is faster than accessing data on the heap because all heap data must
* be accessed through a pointer.
*
* Heap
* Data with an unknown size at compile time or a size that might change must be stored on the heap
* instead.
* The heap is less organized: when you put data on the heap, you request a certain amount of space.
* The memory allocator finds an empty spot in the heap that is big enough, marks it as being in use,
* and returns a pointer, which is the address of that location. This process is called allocating
* on the heap and is sometimes abbreviated as just allocating.
* Pushing values onto the stack is not considered allocating. Because the pointer is a known,
* fixed size, you can store the pointer on the stack, but when you want the actual data, you must
* follow the pointer.
* Accessing data in the heap is slower than accessing data on the stack because you have to follow
* a pointer to get there.
*/
fn main() {
    println!("Hello, world!");
}
