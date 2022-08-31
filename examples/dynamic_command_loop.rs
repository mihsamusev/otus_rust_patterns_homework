use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

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
        Self { position, velocity }
    }
    fn update_state(&mut self) {
        self.position = (
            self.position.0 + self.velocity.0,
            self.position.1 + self.velocity.1,
        );
    }

    fn set_velocity(&mut self, velocity: (f32, f32)) {
        self.velocity = velocity;
    }

    fn describe_state(&self) -> String {
        format!(
            "Body is at position {:?}, with velocity {:?}",
            self.position, self.velocity
        )
    }
}

struct MoveStraightCommand {
    body: Rc<RefCell<LinearBody>>,
}

impl Command for MoveStraightCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.body.borrow_mut().update_state();
        Ok(())
    }
}

struct ChangeVelocityCommand {
    body: Rc<RefCell<LinearBody>>,
    new_velocity: (f32, f32),
}

impl Command for ChangeVelocityCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.body.borrow_mut().set_velocity(self.new_velocity);
        Ok(())
    }
}

struct RepeatCommand {
    inner: CommandRef,
}

impl Command for RepeatCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.inner
            .execute()
            .map_err(|e| CommandError::RepeatError(e.to_string()))
    }
}

struct FailCommand {}

impl Command for FailCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        Err(CommandError::CommandError("I failed".to_string()))
    }
}

// error handling strategies
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
        panic!("I'm dead")
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
                self.command_queue
                    .borrow_mut()
                    .push_front(Box::new(RepeatCommand { inner: command }))
            }
            CommandError::RepeatError(e) => {
                panic!("Retried once, still failure: {}", e)
            }
        };
    }
}

fn execute_body_commands<E: ErrorHandler>(
    body: Rc<RefCell<LinearBody>>,
    command_queue: CommandQueueRef,
    error_handler: &mut E,
) {
    while command_queue.borrow().len() > 0 {
        println!("Current state: {}", body.borrow().describe_state());
        println!("Commands left: {}", command_queue.borrow().len());

        let mut command = command_queue.borrow_mut().pop_front().unwrap();
        match command.execute() {
            Ok(_) => {}
            Err(err) => error_handler.handle(command, err),
        };
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    let body = Rc::new(RefCell::new(LinearBody::new((0.0, 0.0), (1.0, 1.0))));
    let command_queue: CommandQueueRef = Rc::new(RefCell::new(VecDeque::new()));
    command_queue
        .borrow_mut()
        .push_back(Box::new(MoveStraightCommand { body: body.clone() }));
    command_queue
        .borrow_mut()
        .push_back(Box::new(MoveStraightCommand { body: body.clone() }));
    command_queue
        .borrow_mut()
        .push_back(Box::new(FailCommand {}));
    command_queue
        .borrow_mut()
        .push_back(Box::new(ChangeVelocityCommand {
            body: body.clone(),
            new_velocity: (0.5, 1.5),
        }));
    command_queue
        .borrow_mut()
        .push_back(Box::new(MoveStraightCommand { body: body.clone() }));
    command_queue
        .borrow_mut()
        .push_back(Box::new(MoveStraightCommand { body: body.clone() }));

    // select error handlins strategy
    // let mut error_handler = PanicErrorHandler {}; // just crash
    // let mut error_handler = RetryThenPanicErrorHandler {command_queue: command_queue.clone()}; // try again then crash
    let mut error_handler = PrinterErrorHandler {}; // just print the error

    // execute command loop
    execute_body_commands(body, command_queue, &mut error_handler);
}
