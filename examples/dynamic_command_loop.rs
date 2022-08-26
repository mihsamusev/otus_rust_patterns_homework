// https://github.com/mihsamusev/otus_patterns_hw3/blob/main/src/exceptionhandler.py

use std::collections::VecDeque;
use std::{cell::RefCell, rc::Rc};
use std::thread;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("RepeatError: repetition failed - {0}")]
    RepeatError(String),
    #[error("CommandError: execution failed - {0}")]
    CommandError(String),
}

trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
}

type CommandRef = Box<dyn Command>;
type CommandQueueRef = Rc<RefCell<VecDeque<CommandRef>>>;

struct LinearBody {
    position: (f32, f32),
    velocity: (f32, f32),
}

impl LinearBody { 
    fn new(position: (f32, f32), velocity: (f32, f32)) -> Self {
        Self {position, velocity}
    }
    fn update_state(&mut self) {
        self.position = (self.position.0 + self.velocity.0, self.position.1 + self.velocity.1);
    }

    fn set_velocity(&mut self, velocity: (f32, f32)) {
        self.velocity = velocity;
    }

    fn describe_state(&self) -> String {
        format!("Body is at position {:?}, with velocity {:?}", self.position, self.velocity)
    }
}

struct MoveStraightCommand {
    body: Rc<RefCell<LinearBody>>
}

impl Command for MoveStraightCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.body.borrow_mut().update_state();
        Ok(())
    }
}

struct ChangeVelocityCommand {
    body: Rc<RefCell<LinearBody>>,
    new_velocity: (f32, f32)
}

impl Command for ChangeVelocityCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.body.borrow_mut().set_velocity(self.new_velocity);
        Ok(())
    }
}

struct PrintNumberCommand {
    number: f32,
}

impl Command for PrintNumberCommand {
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
        self.inner.execute().map_err(|e| CommandError::RepeatError(e.to_string()))
    }
}

struct FailCommand {}

impl Command for FailCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        // import randomness, if true crash if not Ok
        Err(CommandError::CommandError("I failed".to_string()))
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
            CommandError::CommandError(e) => {
                println!("CommandError: {} -> gonna retry", e);
                self.command_queue.borrow_mut().push_back(Box::new(RepeatCommand{inner: command}))
            },
            CommandError::RepeatError(e) => {
                panic!("Retried once, still failure: {}", e)
            }
        };
    }
}

fn execute_commands<E: ErrorHandler>(command_queue: CommandQueueRef, error_handler: &mut E) {
    while command_queue.borrow().len() > 0 {
        println!("commands left {}", command_queue.borrow().len());
        let mut command = command_queue.borrow_mut().pop_front().unwrap();
        match command.execute() {
            Ok(_) => {}
            Err(err) => error_handler.handle(command, err),
        };
        thread::sleep_ms(1000);
        //command.execute();
    }
}

fn execute_body_commands<E: ErrorHandler>(body: Rc<RefCell<LinearBody>>, command_queue: CommandQueueRef, error_handler: &mut E) {
    while command_queue.borrow().len() > 0 {
        println!("Current state: {}", body.borrow().describe_state());
        println!("Commands left: {}", command_queue.borrow().len());
        let mut command = command_queue.borrow_mut().pop_front().unwrap();
        match command.execute() {
            Ok(_) => {}
            Err(err) => error_handler.handle(command, err),
        };
        thread::sleep_ms(1000);
        //command.execute();
    }
}

fn main() {
    let command_queue: CommandQueueRef = Rc::new(RefCell::new(VecDeque::new()));
    command_queue.borrow_mut().push_back(Box::new(PrintNumberCommand { number: 10.0 }));
    command_queue.borrow_mut().push_back(Box::new(PrintNumberCommand { number: 15.0 }));
    command_queue.borrow_mut().push_back(Box::new(FailCommand {}));
    command_queue.borrow_mut().push_back(Box::new(PrintNumberCommand { number: 20.0 }));   

    let mut error_handler = PrinterErrorHandler {}; // just print the error
    // let mut error_handler = PanicErrorHandler {}; // just crash
    // let mut error_handler = RetryThenPanicErrorHandler {command_queue: command_queue.clone()};
    execute_commands(command_queue, &mut error_handler);


    // example 2
    let body = Rc::new(RefCell::new(LinearBody::new((0.0, 0.0), (1.0, 1.0))));
    let command_queue: CommandQueueRef = Rc::new(RefCell::new(VecDeque::new()));
    command_queue.borrow_mut().push_back(Box::new(MoveStraightCommand {body: body.clone()}));
    command_queue.borrow_mut().push_back(Box::new(MoveStraightCommand {body: body.clone()}));
    command_queue.borrow_mut().push_back(Box::new(ChangeVelocityCommand {body: body.clone(), new_velocity: (0.5, 1.5)}));
    command_queue.borrow_mut().push_back(Box::new(MoveStraightCommand {body: body.clone()}));
    command_queue.borrow_mut().push_back(Box::new(MoveStraightCommand {body: body.clone()}));

    let mut error_handler = PrinterErrorHandler {}; // just print the error
    // let mut error_handler = PanicErrorHandler {}; // just crash
    // let mut error_handler = RetryThenPanicErrorHandler {command_queue: command_queue.clone()};
    execute_body_commands(body.clone(), command_queue, &mut error_handler);
}
