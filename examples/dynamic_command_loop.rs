// https://github.com/mihsamusev/otus_patterns_hw3/blob/main/src/exceptionhandler.py

use std::collections::VecDeque;
use std::{cell::RefCell, rc::Rc};

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("RepeatError: repetition failed")]
    RepeatError(String),
    #[error("CommandError: execution failed")]
    CommandError(String),
}

trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
}

type CommandRef = Rc<RefCell<dyn Command>>;
type CommandQueueRef = Rc<RefCell<VecDeque<CommandRef>>>;

struct PrintNumber {
    number: f32,
}

impl Command for PrintNumber {
    fn execute(&mut self) -> Result<(), CommandError> {
        println!("Number is {}", self.number);
        Ok(())
    }
}

struct RepeatCommand {
    inner: CommandRef,
}

impl Command for RepeatCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.inner
            .borrow_mut()
            .execute()
            .map_err(|e| CommandError::RepeatError(e.to_string()))
    }
}

struct EnqueueCommand {
    inner: CommandRef,
    command_queue: CommandQueueRef,
}

impl Command for EnqueueCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.command_queue
            .borrow_mut()
            .push_back(self.inner.clone());
        Ok(())
    }
}

struct FailCommand {}

impl Command for FailCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        // import randomness, if true crash if not Ok
        Err(CommandError::CommandError("fail".to_string()))
    }
}

trait ErrorHandler {
    fn handle(&mut self, command: CommandRef, error: CommandError);
}

struct PrinterErrorHandler {}

impl ErrorHandler for PrinterErrorHandler {
    fn handle(&mut self, _command: CommandRef, error: CommandError) {
        println!("{:?}", error)
    }
}

struct PanicErrorHandler {}

impl ErrorHandler for PanicErrorHandler {
    fn handle(&mut self, _command: CommandRef, _error: CommandError) {
        panic!("I cant handle this shit")
    }
}

struct RetryThenPanicErrorHandler {
    command_queue: CommandQueueRef,
}

impl ErrorHandler for RetryThenPanicErrorHandler {
    fn handle(&mut self, command: CommandRef, error: CommandError) {
        match error {
            CommandError::CommandError(_) => EnqueueCommand {
                inner: command,
                command_queue: self.command_queue.clone(),
            }
            .execute()
            .unwrap(),
            CommandError::RepeatError(e) => {
                panic!("Retried once, still failure: {}", e)
            }
        };
    }
}

fn execute_commands<E: ErrorHandler>(command_queue: CommandQueueRef, error_handler: &mut E) {
    while command_queue.borrow().len() > 0 {
        let command = command_queue.borrow_mut().pop_front().unwrap();
        match command.borrow_mut().execute() {
            Ok(_) => {}
            Err(err) => error_handler.handle(command.clone(), err),
        };
        //command.execute();
    }
}

fn main() {
    let first_command = Rc::new(RefCell::new(PrintNumber { number: 10.0 }));
    let repeater = RepeatCommand {
        inner: first_command.clone(),
    };
    let command_queue: CommandQueueRef = Rc::new(RefCell::new(VecDeque::new()));
    let enqueuer = EnqueueCommand {
        inner: first_command.clone(),
        command_queue: command_queue.clone(),
    };

    command_queue.borrow_mut().push_back(first_command);
    command_queue
        .borrow_mut()
        .push_back(Rc::new(RefCell::new(repeater)));
    command_queue
        .borrow_mut()
        .push_back(Rc::new(RefCell::new(enqueuer)));
    command_queue
        .borrow_mut()
        .push_back(Rc::new(RefCell::new(FailCommand {})));
    command_queue
        .borrow_mut()
        .push_back(Rc::new(RefCell::new(PrintNumber { number: 20.0 })));

    //let mut error_handler = PrinterErrorHandler {}; // just print the error
    let mut error_handler = PanicErrorHandler {}; // just crash
                                                  //let mut error_handler = RetryThenPanicErrorHandler {
                                                  // try again then crash
                                                  //command_queue: command_queue.clone(),
                                                  //};
    execute_commands(command_queue, &mut error_handler);
}
