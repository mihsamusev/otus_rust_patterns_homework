// https://github.com/mihsamusev/otus_patterns_hw3/blob/main/src/exceptionhandler.py

use std::{rc::Rc, cell::RefCell};
use std::collections::VecDeque;

#[derive(Debug)]
struct CommandError {}
trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
}

struct PrintNumber {
    number: f32
}

impl Command for PrintNumber {
    fn execute(&mut self) -> Result<(), CommandError>{
        println!("Number is {}", self.number);
        Ok(())
    }
}

struct RepeatImmediately {
    inner: Rc<RefCell<dyn Command>>
}

impl Command for RepeatImmediately {
    fn execute(&mut self) -> Result<(), CommandError>{
        self.inner.borrow_mut().execute();
        Ok(())
    }
}

struct RepeatLater {
    inner: Rc<RefCell<dyn Command>>,
    queue: Rc<RefCell<VecDeque<Rc<RefCell<dyn Command>>>>>
}

impl Command for RepeatLater {
    fn execute(&mut self)  -> Result<(), CommandError> {
        self.queue.borrow_mut().push_back(self.inner.clone());
        Ok(())
    }
}

struct UncertainCommand {}

impl Command for UncertainCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        // import randomness, if true crash if not Ok
        Ok(())
    }
}

enum Commands {
    EnqueueFront,
    LogException,
    RepeatOnce,
    RepeatTwice
}


// trait ErrorHandlingStrategy {
//     fn handle(&mut self, error_pair: (&Command, &Error));
// }

enum Strategies {
    Panic,
    Log,
    RetryOnce,
    RetryTwice,

}


trait ErrorHandler {
    fn handle(&mut self, error: CommandError);
}

struct PrinterErrorHandler {}

impl ErrorHandler for PrinterErrorHandler {
    fn handle(&mut self, error: CommandError) {
        println!("{:?}", error)
    }
}

fn execute_commands<E: ErrorHandler>(command_queue: Rc<RefCell<VecDeque<Rc<RefCell<dyn Command>>>>>, error_handler: &mut E) {
    while command_queue.borrow().len() > 0 {
        let command = command_queue.borrow_mut().pop_front().unwrap();
        match command.borrow_mut().execute() {
            Ok(_) => {},
            Err(err) => error_handler.handle(err)
        };
        //command.execute();
    }
}

fn main() {
    let first_command = Rc::new(RefCell::new(PrintNumber{number: 10.0}));
    let repeater = RepeatImmediately{inner: first_command.clone()};
    let command_queue: Rc<RefCell<VecDeque<Rc<RefCell<dyn Command>>>>> = Rc::new(RefCell::new(VecDeque::new()));
    let enqueuer = RepeatLater{inner: first_command.clone(), queue: command_queue.clone()};
    command_queue.borrow_mut().push_back(first_command);
    command_queue.borrow_mut().push_back(Rc::new(RefCell::new(repeater)));
    command_queue.borrow_mut().push_back(Rc::new(RefCell::new(enqueuer)));
    command_queue.borrow_mut().push_back(Rc::new(RefCell::new(PrintNumber{number: 20.0})));

    let mut error_handler = PrinterErrorHandler{};
    execute_commands(command_queue, &mut error_handler);
}
