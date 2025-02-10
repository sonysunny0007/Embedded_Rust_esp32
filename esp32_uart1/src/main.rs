use esp_idf_hal::{
    delay::{FreeRtos, NON_BLOCK},
    gpio,
    peripherals::Peripherals,
    prelude::*,
    uart::*,
};
use esp_idf_sys as _;

const STATUS_MSG: &[u8] = b"System OK";
const CR: u8 = 13;

fn main() {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let tx = peripherals.pins.gpio13;  // UART TX pin
    let rx = peripherals.pins.gpio12;  // UART RX pin

    let config = config::Config::new().baudrate(Hertz(115_200));
    let uart = UartDriver::new(
        peripherals.uart1,
        tx,
        rx,
        Option::<gpio::Gpio0>::None,
        Option::<gpio::Gpio1>::None,
        &config,
    )
    .unwrap();

    // Transmit system status message upon startup
    if let Err(e) = uart.write(STATUS_MSG) {
        eprintln!("Failed to send status message: {:?}", e);
    }

    let mut cli_buf: Vec<u8> = Vec::new();

    loop {
        let mut buf: [u8; 10] = [0; 10];
        match uart.read(&mut buf, NON_BLOCK) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    let b = buf[0];
                    cli_buf.push(b);
                    if b == CR {
                        // Send back the received data when CR is encountered
                        if let Err(e) = uart.write(&cli_buf) {
                            eprintln!("Failed to write received data: {:?}", e);
                        } else {
                            println!("{:?} written", cli_buf);
                        }
                        cli_buf.clear();
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from UART: {:?}", e);
            }
        }

        // Delay to avoid busy-waiting
        FreeRtos::delay_ms(100);
    }
}
