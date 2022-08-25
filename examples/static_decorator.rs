use std::rc::Rc;

// A device component that can report its status
trait Device {
    fn report(&self) -> String;
}

enum DeviceType {
    Default(DefaultDevice),
    Thermometer(TemperatureDecorator),
    HumidityMeter(HumidityDecorator)
}

impl Device for DeviceType {
    fn report(&self) -> String {
        match self {
            Self::Default(device) => device.report(),
            Self::Thermometer(device) => device.report(),
            Self::HumidityMeter(device) => device.report()
        }
    }

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
    fn new(device: Rc<DeviceType>) -> Self;
}

struct TemperatureDecorator {
    device: Rc<DeviceType>
}

impl DeviceDecorator for TemperatureDecorator {
    fn new(device: Rc<DeviceType>) -> Self {
        Self {device}
    }
}

impl Device for TemperatureDecorator {
    fn report(&self) -> String {
        format!("{}\nI am device with temperature sensor, i can return temperature measurements", self.device.report())
    }
}

struct HumidityDecorator {
    device: Rc<DeviceType>
}

impl DeviceDecorator for HumidityDecorator {
    fn new(device: Rc<DeviceType>) -> Self {
        Self {device}
    }
}

impl Device for HumidityDecorator {
    fn report(&self) -> String {
        format!("{}\nI am device with humidity sensor, i can return humidity measurements", self.device.report())
    }
}


struct Client;

impl Client {
    fn print_report<T: Device>(device: &T) {
        println!("{}", device.report())
    }
}

fn main() {
    let base_device = Rc::new(DeviceType::Default(DefaultDevice{}));
    Client::print_report(base_device.as_ref());

    let decorated_device = Rc::new(DeviceType::Thermometer(TemperatureDecorator::new(base_device)));
    let decorated_device = Rc::new(DeviceType::HumidityMeter(HumidityDecorator::new(decorated_device)));
    Client::print_report(decorated_device.as_ref());
}