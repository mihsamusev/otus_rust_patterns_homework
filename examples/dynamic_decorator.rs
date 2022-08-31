use std::rc::Rc;

// A device component that can report its status
trait Device {
    fn report(&self) -> String;
}

// Base device on which all decorators can be attached
struct DefaultDevice {}

impl Device for DefaultDevice {
    fn report(&self) -> String {
        "I am default device, i can only be turned on or off".to_string()
    }
}

// define how to wrap the concrete decorators
trait DeviceDecorator: Device {
    fn new(component: Rc<dyn Device>) -> Self;
}

struct TemperatureDecorator {
    device: Rc<dyn Device>,
}

impl DeviceDecorator for TemperatureDecorator {
    fn new(device: Rc<dyn Device>) -> Self {
        Self { device }
    }
}

impl Device for TemperatureDecorator {
    fn report(&self) -> String {
        format!(
            "{}\nI am device with temperature sensor, i can return temperature measurements",
            self.device.report()
        )
    }
}

struct HumidityDecorator {
    device: Rc<dyn Device>,
}

impl DeviceDecorator for HumidityDecorator {
    fn new(device: Rc<dyn Device>) -> Self {
        Self { device }
    }
}

impl Device for HumidityDecorator {
    fn report(&self) -> String {
        format!(
            "{}\nI am device with humidity sensor, i can return humidity measurements",
            self.device.report()
        )
    }
}

struct Client;
impl Client {
    fn print_report<T: Device>(device: &T) {
        println!("{}", device.report())
    }
}
fn main() {
    let base_device = Rc::new(DefaultDevice {});
    Client::print_report(base_device.as_ref());

    let sensor_1 = TemperatureDecorator::new(base_device);
    let sensor_2 = HumidityDecorator::new(Rc::new(sensor_1));
    Client::print_report(&sensor_2);
}
